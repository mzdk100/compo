use {
    crate::{component::Component, prelude::Runtime},
    std::{
        cell::UnsafeCell,
        mem::transmute,
        rc::{Rc, Weak},
    },
};

/// The main event loop structure for managing component execution and event handling.
pub struct Loop {
    quit_flag: UnsafeCell<bool>,
    handlers: UnsafeCell<Vec<Box<dyn Fn(&Self)>>>,
}

impl Loop {
    /// Creates a new instance of the event loop.
    pub fn new() -> Self {
        Self {
            quit_flag: false.into(),
            handlers: Default::default(),
        }
    }

    /// Signals the event loop to stop running.
    /// Once called, the loop will exit after completing the current iteration.
    pub fn quit(&self) {
        unsafe { *self.quit_flag.get() = true };
    }

    /// Registers a handler function to be called on each iteration of the event loop.
    /// The handler will be invoked before checking the quit flag.
    pub fn register_poll_handler<F>(self, handler: F) -> Self
    where
        F: Fn(&Self) + 'static,
    {
        let handlers: &mut Vec<Box<dyn Fn(&Self)>> = unsafe { transmute(self.handlers.get()) };
        handlers.push(Box::new(handler) as _);
        self
    }

    /// Starts the event loop with the provided entry point.
    /// The entry point is an async function that will be executed in the context of the loop.
    pub fn run<'a, F, C>(self, entry: F)
    where
        C: Component<'a> + 'a,
        F: AsyncFn(Weak<C>) + 'a,
    {
        let rt = Rc::new(Runtime::new());
        let rt_weak = Rc::downgrade(&rt);
        let c = Rc::new(C::new(rt_weak.clone()));
        let c_weak = Rc::downgrade(&c);
        rt.spawn(async move { entry(c_weak).await });
        loop {
            rt.poll_all();
            if unsafe { *self.quit_flag.get() } {
                break;
            }

            let handlers: &mut Vec<Box<dyn Fn(&Self)>> = unsafe { transmute(self.handlers.get()) };
            for h in handlers.iter() {
                h(&self);
            }
        }
    }
}

/// Convenience function that creates a new event loop and runs it with the provided entry point.
/// This is equivalent to `Loop::new().run(entry)`.
pub fn run<'a, F, C>(entry: F)
where
    C: Component<'a> + 'a,
    F: AsyncFn(Weak<C>) + 'a,
{
    Loop::new().run(entry)
}
