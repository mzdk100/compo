pub use std::time::Duration;

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

/// A future that completes after a specified duration has elapsed.
pub struct Sleep {
    /// The time at which the sleep should complete
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Check if the deadline has been reached
        if Instant::now() >= this.deadline {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Creates a new `Sleep` that will complete after the specified duration.
///
/// # Examples
///
/// ```
/// use compo::prelude::*;
///
/// async fn example() {
///     // Sleep for 100 milliseconds
///     sleep(Duration::from_millis(100)).await;
/// }
/// ```
pub fn sleep(duration: Duration) -> Sleep {
    Sleep {
        deadline: Instant::now() + duration,
    }
}
