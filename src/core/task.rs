use parking_lot::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    hash::Hash,
    sync::Arc,
};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

struct QueuedTask<K> {
    id: K,
    task: Box<dyn FnOnce() + Send>,
}

struct State<K> {
    max_concurrency: usize,
    current_running: usize,
    pending_queue: VecDeque<QueuedTask<K>>,
    tasks: HashMap<K, (u64, CancellationToken)>,
    next_tag: u64,
}

impl<K> Drop for State<K> {
    fn drop(&mut self) {
        for queued in self.pending_queue.drain(..) {
            (queued.task)();
        }
    }
}

#[derive(Clone)]
pub struct TaskSet<K> {
    state: Arc<Mutex<State<K>>>,
    idle_tx: Arc<watch::Sender<usize>>,
}

struct TaskGuard<K: Clone + Eq + Hash + Send + 'static> {
    this: TaskSet<K>,
    id: K,
    tag: u64,
}

impl<K: Clone + Eq + Hash + Send + 'static> Drop for TaskGuard<K> {
    fn drop(&mut self) {
        self.this.on_task_finished(&self.id, self.tag);
    }
}

impl<K: Clone + Eq + Hash + Send + 'static> TaskSet<K> {
    pub fn new(max_concurrency: usize) -> Self {
        let (tx, _rx) = watch::channel(0);
        Self {
            state: Arc::new(Mutex::new(State {
                max_concurrency,
                current_running: 0,
                pending_queue: VecDeque::new(),
                tasks: HashMap::new(),
                next_tag: 0,
            })),
            idle_tx: Arc::new(tx),
        }
    }

    /// 添加任务
    pub fn add_task<F>(&self, id: K, cancel_token: CancellationToken, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut state = self.state.lock();
        let tag = {
            state.next_tag += 1;
            state.next_tag - 1
        };

        if let Some((_, old_token)) = state.tasks.insert(id.clone(), (tag, cancel_token.clone())) {
            old_token.cancel();
            if let Some(pos) = state.pending_queue.iter().position(|q| q.id == id) {
                let queued = state.pending_queue.remove(pos).unwrap();
                state.current_running += 1;
                (queued.task)();
            }
        };

        let wrapped_fn = {
            let weak_state = Arc::downgrade(&self.state);
            let weak_tx = Arc::downgrade(&self.idle_tx);
            let id = id.clone();
            move || match (weak_state.upgrade(), weak_tx.upgrade()) {
                (Some(state), Some(idle_tx)) => {
                    let this = TaskSet { state, idle_tx };
                    tokio::spawn(async move {
                        let _guard = TaskGuard { this, id, tag };
                        fut.await;
                    });
                }
                _ => {
                    tokio::spawn(fut);
                }
            }
        };

        if state.current_running < state.max_concurrency {
            state.current_running += 1;
            wrapped_fn();
        } else {
            state.pending_queue.push_back(QueuedTask {
                id,
                task: Box::new(wrapped_fn),
            });
        }
    }

    /// 取消指定任务
    pub fn cancel_task(&self, id: &K) {
        let mut state = self.state.lock();
        if let Some(entry) = state.tasks.remove(id) {
            entry.1.cancel();
        }
        if let Some(pos) = state.pending_queue.iter().position(|q| q.id == *id) {
            let queued = state.pending_queue.remove(pos).unwrap();
            state.current_running += 1;
            (queued.task)();
        }
        self.try_spawn_next(&mut state);
    }

    /// 取消全部任务
    pub fn cancel_all(&self) {
        let mut state = self.state.lock();
        for (_, (_, token)) in state.tasks.drain() {
            token.cancel();
        }
        while let Some(queued) = state.pending_queue.pop_front() {
            state.current_running += 1;
            (queued.task)();
        }
        self.try_spawn_next(&mut state);
    }

    /// 等待所有任务完成，无任务时立刻返回
    pub fn join(&self) -> impl Future<Output = ()> {
        let state = self.state.clone();
        let mut rx = self.idle_tx.subscribe();
        async move {
            loop {
                {
                    let s = state.lock();
                    if s.current_running == 0 && s.pending_queue.is_empty() {
                        return;
                    }
                }
                let _ = rx.changed().await;
            }
        }
    }

    /// 等待所有任务完成，无任务时等待
    pub fn wait_last(&self) -> impl Future<Output = ()> {
        let state = self.state.clone();
        let mut rx = self.idle_tx.subscribe();
        async move {
            let baseline = {
                let s = state.lock();
                if s.current_running == 0 && s.pending_queue.is_empty() {
                    Some(s.next_tag)
                } else {
                    None
                }
            };
            loop {
                {
                    let s = state.lock();
                    if s.current_running == 0 && s.pending_queue.is_empty() {
                        match baseline {
                            Some(tag) => {
                                if s.next_tag > tag {
                                    return;
                                }
                            }
                            None => return,
                        }
                    }
                }
                let _ = rx.changed().await;
            }
        }
    }

    /// 调整并发数
    pub fn set_concurrency(&self, new_max: usize) {
        let mut state = self.state.lock();
        state.max_concurrency = new_max;
        self.try_spawn_next(&mut state);
    }

    /// 状态统计
    pub fn stats(&self) -> (usize, usize) {
        let state = self.state.lock();
        (state.current_running, state.pending_queue.len())
    }

    fn on_task_finished(&self, id: &K, task_tag: u64) {
        let mut state = self.state.lock();
        if let Some((existing_tag, _)) = state.tasks.get(id)
            && *existing_tag == task_tag
        {
            state.tasks.remove(id);
        }
        state.current_running = state.current_running.saturating_sub(1);
        self.try_spawn_next(&mut state);
    }

    fn try_spawn_next(&self, state: &mut State<K>) {
        while state.current_running < state.max_concurrency
            && let Some(queued) = state.pending_queue.pop_front()
        {
            state.current_running += 1;
            (queued.task)();
        }
        if state.current_running == 0 && state.pending_queue.is_empty() {
            self.idle_tx.send_modify(|v| *v = v.wrapping_add(1));
        }
    }
}
