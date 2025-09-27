//! # Executor implementation
//!
//! This submodule provides the core components and functionality needed to manage and execute
//! asynchronous tasks within the `miniloop` crate. It includes the `Executor` struct, responsible
//! for handling task management, and related utilities for polling tasks.
//!
//! An executor contains a statically allocated list of tasks. The size of that list is defined by
//! the constant generic parameter.
//!
//! ## Examples
//!
//! ### Running the Executor
//! ```no_run
//! # use miniloop::executor::Executor;
//! # use miniloop::task::Task;
//! const TASK_ARRAY_SIZE: usize = 4;
//! let mut executor: Executor<TASK_ARRAY_SIZE> = Executor::new();
//! let mut task = Task::new("task1", async { println!("Task executed"); });
//! let mut handle = task.create_handle();
//! executor.spawn(&mut task, &mut handle).expect("Failed to spawn task");
//! executor.run();
//! ```
//!
//! ## Usage Notes
//! - The `Executor` is designed to work with a fixed task slot size. Trying to add more than 4 tasks will result in an error (`NoFreeSlots`).
//! - Ensure that tasks added to the executor are correctly managed and polled to avoid resource leaks or incomplete executions.
use crate::sbox::{StackBox, StackBoxFuture};
use crate::task::{Handle, Task};

use core::future::Future;
use core::pin::pin;
use core::ptr;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// An enumeration representing different types of errors that can occur.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Indicates that there are no free slots available.
    NoFreeSlots,
}

/// The `Executor` struct is responsible for managing and running tasks.
pub struct Executor<'a, const TASK_ARRAY_SIZE: usize> {
    /// An array of optional tasks that the executor can manage. The array size is fixed at 4 elements.
    tasks: [Option<StackBoxFuture<'a>>; TASK_ARRAY_SIZE],

    /// An index indicating the current position in the tasks array.
    index: usize,

    /// An optional callback function that takes a `&str` argument and is pending execution.
    pending_callback: Option<fn(&str)>,
}

impl<const TASK_ARRAY_SIZE: usize> Default for Executor<'_, TASK_ARRAY_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const TASK_ARRAY_SIZE: usize> Executor<'a, TASK_ARRAY_SIZE> {
    /// Creates a new instance of the `Executor` struct.
    ///
    /// This function initializes the `Executor` with:
    /// - an array of `None` tasks with a fixed size of 4,
    /// - the index set to 0,
    /// - and no pending callback function.
    ///
    /// # Returns
    ///
    /// A new `Executor` instance.
    ///
    /// # Must Use
    ///
    /// The `#[must_use]` attribute indicates that the returned `Executor` instance should not
    /// be discarded.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tasks: [const { None }; TASK_ARRAY_SIZE],
            index: 0,
            pending_callback: None,
        }
    }

    /// Sets the callback function to be invoked when a task is pending.
    ///
    /// # Parameters
    ///
    /// * `cb`:
    ///   A function pointer to a callback that takes a `&str` argument.
    ///   This callback will be called with the task's name when the task is pending.
    pub fn set_pending_callback(&mut self, cb: fn(&str)) {
        self.pending_callback = Some(cb);
    }

    /// # Errors
    ///
    /// * `NoFreeSlots` - if there is no free slots in the executor
    pub fn spawn<F>(
        &mut self,
        task: &'a mut Task<'a, F>,
        handle: &'a mut Handle<F::Output>,
    ) -> Result<(), Error>
    where
        F: Future + 'a,
    {
        if self.index >= self.tasks.len() {
            return Err(Error::NoFreeSlots);
        }

        task.link_handle(handle);
        let index = self.index;
        self.index += 1;
        self.tasks[index] = Some(StackBox::new(task));

        Ok(())
    }
    /// Blocks on the provided future until it is completed.
    ///
    /// This method will drive the given future to completion, blocking the
    /// current thread during the process. It is useful for running a single
    /// future to completion in a synchronous context.
    ///
    /// # Parameters
    ///
    /// * `future` - The future to be executed until completion. The future
    ///   must implement [`Future`], and its output must match the type `T`.
    ///
    /// # Returns
    ///
    /// This function will return the output of the provided future once it
    /// is resolved.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use miniloop::executor::Executor;
    /// const TASK_ARRAY_SIZE: usize = 1;
    /// let mut executor = Executor::<TASK_ARRAY_SIZE>::new();
    /// let result = executor.block_on(async { 42 });
    /// assert_eq!(result, 42);
    /// ```
    pub fn block_on<F, T>(&mut self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        let waker = create_waker();
        let mut future = pin!(future);
        let mut ctx = Context::from_waker(&waker);

        loop {
            if let Poll::Ready(val) = future.as_mut().poll(&mut ctx) {
                return val;
            }
        }
    }

    /// Executes tasks in the executor until all tasks are completed.
    ///
    /// The method repeatedly polls each task in the tasks array. If a task completes, it is removed from the array.
    /// The function keeps running until all tasks are either completed or removed from the tasks array.
    ///
    /// <div class="warning">
    /// That call does not return till all tasks are finished theirs execution.
    /// </div>
    ///
    /// # Behavior
    ///
    /// - Iterates over all tasks and attempts to poll each one.
    /// - If a task is completed, it is removed from the tasks array.
    /// - If all tasks have been removed (i.e., all tasks are `None`), the function returns.
    pub fn run(&mut self) {
        loop {
            for i in 0..self.tasks.len() {
                let should_remove = match self.tasks[i].as_mut() {
                    Some(task) => poll_task(task, self.pending_callback),
                    None => false,
                };

                if should_remove {
                    self.tasks[i].take();
                }
            }

            if self.tasks.iter().all(Option::is_none) {
                return;
            }
        }
    }
}

/// Polls a given task and optionally calls a callback function if the task is pending.
///
/// # Parameters
///
/// * `task`:
///   A mutable reference to the task being polled.
/// * `cb`:
///   An optional callback function that takes a `&str` argument. This callback is invoked with the task's name if the task is pending.
///
/// # Returns
///
/// * `true` if the task has completed.
/// * `false` if the task is still pending.
fn poll_task(task: &mut StackBoxFuture, cb: Option<fn(&str)>) -> bool {
    if let Some(future) = task.value.get_mut() {
        let waker = create_waker();
        let context = &mut Context::from_waker(&waker);

        if matches!(future.as_mut().poll(context), Poll::Pending) {
            if let Some(cb) = cb {
                cb(future.name().unwrap_or(""));
            }
        } else {
            return true;
        }
    }

    false
}

fn create_raw_waker() -> RawWaker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        create_raw_waker()
    }

    unsafe fn wake(_: *const ()) {}

    unsafe fn wake_by_ref(_: *const ()) {}

    unsafe fn drop(_: *const ()) {}

    RawWaker::new(
        ptr::null(),
        &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
    )
}

fn create_waker() -> Waker {
    let raw_waker = create_raw_waker();

    unsafe { Waker::from_raw(raw_waker) }
}
