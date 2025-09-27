//! Helper module
//!
//! Contains a set of helper functions/structs that helps with executor control:
//!   - `yield_me` - yield current task execution and let the executor switches to another task
//!
//! # Example
//!
//! ```no_run
//! # use miniloop::executor::Executor;
//! # use miniloop::task::Task;
//! # use core::future::Future;
//! use miniloop::helpers::yield_me;
//! const TASK_ARRAY_SIZE: usize = 4;
//! // Assume `some_future` is a mutable future reference
//! let mut executor = Executor::<TASK_ARRAY_SIZE>::new();
//! let mut task1 = Task::new("task1", async {
//!     loop {
//!         // computation
//!         yield_me().await; // let to switch to another task
//!     }
//! });
//! let mut handle1 = task1.create_handle();
//! let mut task2 = Task::new("task2", async {
//!     loop {
//!         // computation
//!         yield_me().await; // let to switch to another task
//!     }
//! });
//! let mut handle2 = task2.create_handle();
//! executor.spawn(&mut task1, &mut handle1).expect("Failed to spawn task");
//! executor.spawn(&mut task2, &mut handle2).expect("Failed to spawn task");
//! executor.run();
//! ```
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
///
/// # Example
/// ```no_run
/// # use miniloop::helpers::yield_me;
/// async fn task() {
///     // some work here
///     yield_me().await; // explicitly let executor to switch to something else
///     // some work here
/// }
/// ```
pub async fn yield_me() {
    Yield::default().await;
}
