use parking_lot::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    hash::Hash,
    pin::Pin,
    sync::Arc,
};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

struct State<K> {
    max_concurrency: usize,
    current_running: usize,
    pending_queue: VecDeque<(CancellationToken, BoxedFuture)>,
    tasks: HashMap<K, (u64, CancellationToken)>,
    next_tag: u64,
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
        };
        let wrapped_fut = {
            let weak_state = Arc::downgrade(&self.state);
            let weak_notify = Arc::downgrade(&self.idle_notify);
            async move {
                let (Some(state), Some(idle_notify)) =
                    (weak_state.upgrade(), weak_notify.upgrade())
                else {
                    return;
                };
                let this = TaskSet { state, idle_notify };
                let _guard = TaskGuard { this, id, tag };
                fut.await;
            }
        };
        if state.current_running < state.max_concurrency {
            state.current_running += 1;
            tokio::spawn(wrapped_fut);
        } else {
            state
                .pending_queue
                .push_back((cancel_token, Box::pin(wrapped_fut)));
        }
    }

    /// 取消指定任务
    pub fn cancel_task(&self, id: &K) {
        let mut state = self.state.lock();
        if let Some(entry) = state.tasks.remove(id) {
            entry.1.cancel();
        }
        self.try_spawn_next(&mut state);
    }

    /// 取消全部任务
    pub fn cancel_all(&self) {
        let mut state = self.state.lock();
        for (_, (_, token)) in state.tasks.drain() {
            token.cancel();
        }
        state.pending_queue.clear();
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
                    if state.current_running == 0 && state.tasks.is_empty() {
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
        if state.current_running > 0 {
            state.current_running -= 1;
        }
        self.try_spawn_next(&mut state);
    }

    fn try_spawn_next(&self, state: &mut State<K>) {
        while state.current_running < state.max_concurrency
            && let Some((token, fut)) = state.pending_queue.pop_front()
        {
            if token.is_cancelled() {
                continue;
            }
            state.current_running += 1;
            tokio::spawn(fut);
        }
        if state.current_running == 0 && state.tasks.is_empty() {
            self.idle_notify.notify_waiters();
        }
    }
}
