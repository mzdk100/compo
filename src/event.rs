use std::{
    borrow::Cow,
    cell::UnsafeCell,
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    mem::transmute,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

/// Error returned when an event emission fails.
///
/// This error occurs in two scenarios:
/// 1. When no listener has called `listen()` before `emit()` is called
/// 2. When all listeners have been destroyed
#[derive(Debug)]
pub struct EventEmitError<T>(T);

impl<T> Error for EventEmitError<T> where T: Debug + Display {}

impl<T> Display for EventEmitError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "EventEmitError: {}", self.0)
    }
}

/// Event emitter for sending events to listeners.
///
/// The emitter is used to broadcast events to all registered listeners.
/// It holds a weak reference to the receivers to avoid memory leaks.
pub struct EventEmitter<'a, T>
where
    T: Clone,
{
    receivers: Weak<UnsafeCell<Vec<&'a mut (Option<Cow<'a, T>>, Option<Waker>)>>>,
}

impl<'a, T> Default for EventEmitter<'a, T>
where
    T: Clone,
{
    fn default() -> Self {
        Self {
            receivers: Default::default(),
        }
    }
}

impl<'a, T> EventEmitter<'a, T>
where
    T: Clone + Debug,
{
    /// Emits an event to all registered listeners.
    ///
    /// # Errors
    ///
    /// This method will return an `EventEmitError` in the following cases:
    /// - No listener has called `listen()` before this method is called
    /// - All listeners have been destroyed
    ///
    /// # Arguments
    ///
    /// * `value` - The event value to emit
    pub fn emit(&self, value: T) -> Result<(), EventEmitError<T>> {
        let Some(receivers) = self.receivers.upgrade() else {
            return Err(EventEmitError(value));
        };

        let receivers: &mut Vec<&'a mut (Option<Cow<'a, T>>, Option<Waker>)> =
            unsafe { transmute(receivers.get()) };
        if receivers.is_empty() {
            return Err(EventEmitError(value));
        }

        let value2 = Cow::<'a, T>::Owned(value.clone());
        // 唤醒所有等待的接收者
        for (v, w) in receivers.drain(..) {
            if v.is_none() {
                *v = Some(value2.clone());
                if let Some(w) = w {
                    w.wake_by_ref();
                }
            } else {
                return Err(EventEmitError(value));
            }
        }

        Ok(())
    }
}

/// Event listener for receiving events.
///
/// The listener is used to register interest in events and create futures
/// that will be resolved when events are emitted.
pub struct EventListener<'a, T>
where
    T: Clone,
{
    receivers: Rc<UnsafeCell<Vec<&'a mut (Option<Cow<'a, T>>, Option<Waker>)>>>,
    value: UnsafeCell<(Option<Cow<'a, T>>, Option<Waker>)>,
}

impl<'a, T> Clone for EventListener<'a, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            receivers: self.receivers.clone(),
            value: (Default::default(), Default::default()).into(),
        }
    }
}

impl<'a, T> Default for EventListener<'a, T>
where
    T: Clone,
{
    fn default() -> Self {
        Self {
            receivers: UnsafeCell::new(Default::default()).into(),
            value: UnsafeCell::new(Default::default()),
        }
    }
}

impl<'a, T> EventListener<'a, T>
where
    T: Clone,
{
    /// Creates a new event emitter associated with this listener.
    ///
    /// The created emitter can be used to broadcast events to all listeners
    /// created from the same `EventListener` instance.
    pub fn new_emitter(&self) -> EventEmitter<'a, T> {
        EventEmitter {
            receivers: Rc::downgrade(&self.receivers),
        }
    }

    /// Registers interest in receiving events and returns a future that will be resolved
    /// when an event is emitted.
    ///
    /// This method must be called before any `emit()` calls for the emitter to successfully
    /// broadcast events. If all listeners are destroyed, subsequent `emit()` calls will fail.
    pub fn listen(&self) -> RecvFuture<'a, T> {
        let receivers: &mut Vec<&'a mut (Option<Cow<'a, T>>, Option<Waker>)> =
            unsafe { transmute(self.receivers.get()) };
        let value = unsafe { transmute(self.value.get()) };
        let value2 = unsafe { transmute(self.value.get()) };
        receivers.push(value2);
        RecvFuture { value }
    }
}

/// A future that resolves when an event is received.
///
/// This future is created by calling `listen()` on an `EventListener` and will
/// be resolved when an event is emitted through the associated `EventEmitter`.
pub struct RecvFuture<'a, T>
where
    T: Clone,
{
    value: &'a mut (Option<Cow<'a, T>>, Option<Waker>),
}

/// Implementation of the `Future` trait for `RecvFuture`.
impl<'a, T> Future for RecvFuture<'a, T>
where
    T: Clone,
{
    type Output = Cow<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(value) = this.value.0.take() {
            // 有新消息可用
            Poll::Ready(value)
        } else {
            this.value.1.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event::EventListener;

    #[tokio::test]
    async fn test_event_listener_basic() -> anyhow::Result<()> {
        let listener = EventListener::default();
        let emitter = listener.new_emitter();
        let fut = listener.listen();

        // 广播一个值
        emitter.emit(42)?;

        // 监听值并验证
        let result = fut.await;
        assert_eq!(*result, 42);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_multiple_listeners() -> anyhow::Result<()> {
        let listener1 = EventListener::default();
        let listener2 = listener1.clone();
        let emitter = listener1.new_emitter();
        let fut1 = listener1.listen();
        let fut2 = listener2.listen();

        // 广播一个值
        emitter.emit("hello".to_string())?;

        // 监听值并验证
        let result1 = fut1.await;
        let result2 = fut2.await;

        assert_eq!(*result1, "hello".to_string());
        assert_eq!(*result2, "hello".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_event_multiple_messages() -> anyhow::Result<()> {
        let listener = EventListener::default();
        let emitter = listener.new_emitter();
        let fut1 = listener.listen();

        // 广播第一个值
        emitter.emit(1)?;

        // 监听第一个值并验证
        let result1 = fut1.await;
        assert_eq!(*result1, 1);

        let fut2 = listener.listen();

        // 广播第二个值
        emitter.emit(2)?;

        // 监听第二个值并验证
        let result2 = fut2.await;
        assert_eq!(*result2, 2);

        Ok(())
    }
}
