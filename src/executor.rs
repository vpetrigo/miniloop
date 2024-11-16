//! # Executor implementation
//!
//! This sub-module provides the core components and functionality needed to manage and execute
//! asynchronous tasks within the `miniloop` crate. It includes the `Executor` struct, responsible
//! for handling task management, and related utilities for polling tasks.
//!
//! ## Examples
//!
//! ### Creating a New Executor
//! ```rust
//! # use miniloop::executor::Executor;
//! let executor = Executor::new();
//! ```
//!
//! ### Setting a Pending Callback
//! ```rust
//! # use miniloop::executor::Executor;
//! let mut executor = Executor::new();
//! executor.set_pending_callback(|task_name| {
//!     println!("Task {} is pending", task_name);
//! });
//! ```
//!
//! ### Spawning a Task
//! ```no_run
//! # use miniloop::executor::Executor;
//! # use core::future::Future;
//! // Assume `some_future` is a mutable future reference
//! let mut executor = Executor::new();
//! # let mut some_future = async {};
//! executor.spawn("task1", &mut some_future).expect("Failed to spawn task");
//! ```
//!
//! ### Running the Executor
//! ```no_run
//! # use miniloop::executor::Executor;
//! # use core::future::Future;
//! // Assume `some_future` is a mutable future reference
//! let mut executor = Executor::new();
//! # let mut some_future = async {};
//! executor.spawn("task1", &mut some_future).expect("Failed to spawn task");
//! executor.run();
//! ```
//!
//! ## Usage Notes
//! - The `Executor` is designed to work with a fixed task slot size of 4. Trying to add more than 4 tasks will result in an error (`NoFreeSlots`).
//! - Ensure that tasks added to the executor are correctly managed and polled to avoid resource leaks or incomplete executions.
use crate::task::Task;
use core::future::Future;
use core::ptr;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// An enumeration representing different types of errors that can occur.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Indicates that there are no free slots available.
    NoFreeSlots,
}

/// The `Executor` struct is responsible for managing and running tasks.
pub struct Executor<'a> {
    /// An array of optional tasks that the executor can manage. The array size is fixed at 4 elements.
    tasks: [Option<Task<'a>>; 4],

    /// An index indicating the current position in the tasks array.
    index: usize,

    /// An optional callback function that takes a `&str` argument and is pending execution.
    pending_callback: Option<fn(&str)>,
}

impl<'a> Default for Executor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Executor<'a> {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// # use miniloop::executor::Executor;
    ///
    /// let executor = Executor::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: [const { None }; 4],
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
    pub fn spawn(
        &mut self,
        name: &'a str,
        future: &'a mut impl Future<Output = ()>,
    ) -> Result<(), Error> {
        if self.index >= self.tasks.len() {
            return Err(Error::NoFreeSlots);
        }

        let index = self.index;
        self.index += 1;
        self.tasks[index] = Some(Task::new(name, future));

        Ok(())
    }

    /// Executes tasks in the executor until all tasks are completed.
    ///
    /// The method repeatedly polls each task in the tasks array. If a task completes, it is removed from the array.
    /// The function keeps running until all tasks are either completed or removed from the tasks array.
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
fn poll_task(task: &mut Task, cb: Option<fn(&str)>) -> bool {
    if let Some(future) = task.future.value.get_mut() {
        let waker = create_waker();
        let context = &mut Context::from_waker(&waker);

        if matches!(future.as_mut().poll(context), Poll::Pending) {
            if let Some(cb) = cb {
                cb(task.name);
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
