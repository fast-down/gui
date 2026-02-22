use parking_lot::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    hash::Hash,
    sync::Arc,
};
use tokio::sync::Notify;
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
    idle_notify: Arc<Notify>,
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
        Self {
            state: Arc::new(Mutex::new(State {
                max_concurrency,
                current_running: 0,
                pending_queue: VecDeque::new(),
                tasks: HashMap::new(),
                next_tag: 0,
            })),
            idle_notify: Arc::new(Notify::new()),
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
            let mut i = 0;
            while i < state.pending_queue.len() {
                if state.pending_queue[i].id == id {
                    let queued = state.pending_queue.remove(i).unwrap();
                    state.current_running += 1;
                    (queued.task)();
                    break;
                } else {
                    i += 1;
                }
            }
        };

        let wrapped_fn = {
            let weak_state = Arc::downgrade(&self.state);
            let weak_notify = Arc::downgrade(&self.idle_notify);
            let id = id.clone();
            move || match (weak_state.upgrade(), weak_notify.upgrade()) {
                (Some(state), Some(idle_notify)) => {
                    let this = TaskSet { state, idle_notify };
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

        let mut i = 0;
        while i < state.pending_queue.len() {
            if state.pending_queue[i].id == *id {
                let queued = state.pending_queue.remove(i).unwrap();
                state.current_running += 1;
                (queued.task)();
                break;
            } else {
                i += 1;
            }
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

    /// 等待所有任务完成
    pub fn join(&self) -> impl Future<Output = ()> {
        let state = self.state.clone();
        let notify = self.idle_notify.clone();
        async move {
            loop {
                {
                    let state = state.lock();
                    if state.current_running == 0 && state.pending_queue.is_empty() {
                        return;
                    }
                    notify.notified()
                }
                .await;
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
            self.idle_notify.notify_waiters();
        }
    }
}
