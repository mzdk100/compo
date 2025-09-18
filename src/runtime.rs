use {
    futures_util::{FutureExt, future::LocalBoxFuture},
    std::{
        cell::{Cell, RefCell},
        task::{Context, Waker},
    },
};

//noinspection SpellCheckingInspection
pub struct Runtime<'a, R> {
    context: RefCell<Context<'a>>,
    pendings: RefCell<Vec<LocalBoxFuture<'a, R>>>,
    // 用于在 poll_all 过程中收集新的 futures
    new_pendings: RefCell<Vec<LocalBoxFuture<'a, R>>>,
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

    pub fn spawn<Fut>(&self, fut: Fut)
    where
        Fut: Future<Output=R> + 'a,
    {
        let fut = fut.boxed_local();

        // 如果当前正在执行 poll_all，将新的 future 添加到 new_pendings
        // 否则直接添加到 pendings
        if self.polling.get() {
            self.new_pendings.borrow_mut().push(fut);
        } else {
            self.pendings.borrow_mut().push(fut);
        }
    }

    pub fn count(&self) -> usize {
        self.pendings.borrow().len() + self.new_pendings.borrow().len()
    }

    pub fn poll_all(&self) {
        // 标记开始执行 poll_all
        self.polling.set(true);

        // 处理当前的 pendings
        {
            let mut pendings = self.pendings.borrow_mut();
            pendings.retain_mut(|i| i.as_mut().poll(&mut self.context.borrow_mut()).is_pending());

            // 将 new_pendings 中的 futures 移动到 pendings
            let mut new_pendings = self.new_pendings.borrow_mut();
            pendings.append(&mut new_pendings);
        }

        // 标记结束执行 poll_all
        self.polling.set(false);
    }
}
