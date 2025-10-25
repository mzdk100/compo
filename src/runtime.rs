use {
    futures_util::{FutureExt, future::LocalBoxFuture},
    std::{
        cell::{Cell, RefCell},
        rc::Rc,
        task::{Context, Waker},
    },
};

/// A cancellable handle that can be used to cancel a spawned future
///
/// This struct provides two main functionalities:
/// 1. `cancel()` - Explicitly cancels the associated future
/// 2. `is_cancelled()` - Checks if the future has been cancelled
///
/// The cancellation is cooperative - the future must explicitly check
/// the cancellation status during execution
#[derive(Clone)]
pub struct Cancellable {
    _cancelled: Rc<Cell<bool>>,
}

impl Cancellable {
    fn new() -> Self {
        Self {
            _cancelled: Default::default(),
        }
    }

    pub fn cancel(&self) {
        self._cancelled.set(true);
    }

    pub fn is_cancelled(&self) -> bool {
        self._cancelled.get()
    }
}

impl Default for Cancellable {
    fn default() -> Self {
        Self::new()
    }
}

struct Task<'a, R> {
    cancellable: Cancellable,
    future: LocalBoxFuture<'a, R>,
}

//noinspection SpellCheckingInspection
pub struct Runtime<'a, R> {
    context: RefCell<Context<'a>>,
    pendings: RefCell<Vec<Task<'a, R>>>,
    // 用于在 poll_all 过程中收集新的 futures
    new_pendings: RefCell<Vec<Task<'a, R>>>,
    // 标记是否正在执行 poll_all
    polling: Cell<bool>,
}

impl<'a, R> Runtime<'a, R> {
    pub fn new() -> Self {
        Self {
            context: Context::from_waker(Waker::noop()).into(),
            pendings: Vec::default().into(),
            new_pendings: Vec::default().into(),
            polling: Cell::new(false),
        }
    }

    /// Spawn a future and return a JoinHandle to await its completion
    pub fn spawn<Fut>(&self, fut: Fut) -> Cancellable
    where
        Fut: Future<Output=R> + 'a,
    {
        let handle = Cancellable::new();
        let task = Task {
            cancellable: handle.clone(),
            future: fut.boxed_local(),
        };

        // 如果当前正在执行 poll_all，将新的 future 添加到 new_pendings
        // 否则直接添加到 pendings
        if self.polling.get() {
            self.new_pendings.borrow_mut().push(task);
        } else {
            self.pendings.borrow_mut().push(task);
        }

        handle
    }

    pub fn count(&self) -> usize {
        self.pendings.borrow().len() + self.new_pendings.borrow().len()
    }

    //noinspection SpellCheckingInspection
    pub fn poll_all(&self) {
        // 标记开始执行 poll_all
        self.polling.set(true);

        // 处理当前的 pendings
        {
            let mut pendings = self.pendings.borrow_mut();
            pendings.retain_mut(|task| {
                task.future
                    .as_mut()
                    .poll(&mut self.context.borrow_mut())
                    .is_pending()
                    && !task.cancellable.is_cancelled()
            });

            // 将 new_pendings 中的 futures 移动到 pendings
            let mut new_pendings = self.new_pendings.borrow_mut();
            pendings.append(&mut new_pendings);
        }

        // 标记结束执行 poll_all
        self.polling.set(false);
    }
}
