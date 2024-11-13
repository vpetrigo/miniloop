use core::default::Default;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A struct that implements the `Future` trait to create a single-yield future.
#[derive(Default)]
struct Yield {
    /// A flag indicating whether the future has yielded once.
    flag: bool,
}

impl Future for Yield {
    type Output = ();

    /// Polls the future to determine if it is ready.
    ///
    /// # Parameters
    ///
    /// * `cx`:
    ///   A mutable reference to the task's context, used to wake up the task when it is ready to make progress.
    ///
    /// # Returns
    ///
    /// * `Poll::Ready(())` if the future has already yielded once and is now ready.
    /// * `Poll::Pending` if the future needs to yield.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.flag {
            return Poll::Ready(());
        }

        self.get_mut().flag = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

/// Asynchronously yields execution back to the executor.
///
/// This function creates an instance of the `Yield` future and awaits its completion,
/// effectively yielding execution back to the executor once.
pub async fn yield_me() {
    Yield::default().await;
}
