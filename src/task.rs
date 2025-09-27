//! # `Task` implementation
//!
//! This module provides utilities for creating and managing named asynchronous operations.
//! It includes the `Task` struct, the `StackBox` struct, and the `StackBoxFuture` type alias.
//! These components help in organizing asynchronous tasks and executing them on the stack.
//!
//! ## Overview
//!
//! The core functionalities provided in this module facilitate the creation and management of
//! asynchronous operations by associating them with names and allowing them to be stored and
//! run on the stack. The major components are:
//!
//! 1. [`Task`]: Represents a named asynchronous operation.
//!
//! ## Examples
//!
//! ### Creating a `Task`
//!
//! ```rust
//! use miniloop::task::Task;
//!
//! let task_name = "example_task";
//! // Example future, replace `()` with actual future logic
//! let task = Task::new(task_name, async { () });
//! ```

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, ready};

pub struct Handle<T> {
    pub value: Option<T>,
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

/// A `Task` represents a named asynchronous operation.
///
/// # Examples
///
/// ```
/// use miniloop::task::Task;
///
/// let task_name = "example_task";
/// // Example future, replace `()` with actual future logic
/// let task = Task::new(task_name, async { () });
/// ```
pub struct Task<'a, F: Future> {
    /// A string that holds the name of the task.
    pub name: Option<&'a str>,
    /// A future representing the asynchronous operation associated with the task.
    pub future: F,
    handle: Option<&'a mut Handle<F::Output>>,
}

impl<'a, F: Future> Task<'a, F> {
    const fn new_impl(name: Option<&'a str>, future: F) -> Self {
        Self {
            name,
            future,
            handle: None,
        }
    }
    /// Creates a new `Task` with the specified name and future.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the task.
    /// * `future` - A mutable reference to an object that implements the
    ///   `Future` trait with an output type of `()`.
    ///
    /// # Returns
    ///
    /// A new instance of `Task`.
    ///
    /// # Examples
    ///
    /// ```
    /// use miniloop::task::Task;
    /// let task = Task::new("example_task", async {});
    /// ```
    pub const fn new(name: &'a str, future: F) -> Self {
        Self::new_impl(Some(name), future)
    }

    /// Creates a new instance of the struct without a name and initializes it with the given future.
    ///
    /// # Arguments
    ///
    /// - `future`: A mutable reference to an implementation of [`Future`] with an output of `()`.
    ///
    /// # Returns
    ///
    /// A new instance of the struct with:
    /// - `name` set to `None`.
    /// - `future` initialized using the provided `future` wrapped in a `StackBox`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use miniloop::task::Task;
    /// let instance = Task::new_nameless(async {});
    /// ```
    pub const fn new_nameless(future: F) -> Self {
        Self::new_impl(None, future)
    }

    /// Creates a default handle for the task's output.
    ///
    /// # Returns
    ///
    /// A new instance of [`Handle`] with its value set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use miniloop::task::{Task, Handle};
    ///
    /// let task = Task::new("example_task", async { 42 });
    /// let handle = task.create_handle();
    /// assert!(handle.value.is_none());
    /// ```
    #[must_use]
    pub fn create_handle(&self) -> Handle<F::Output> {
        Handle::default()
    }

    /// Links a mutable reference to a [`Handle`] with the task.
    ///
    /// # Arguments
    ///
    /// * `handle` - A mutable reference to a [`Handle`] that stores the output of the task's future.
    ///
    /// # Examples
    ///
    /// ```
    /// use miniloop::executor::Executor;
    /// use miniloop::task::{Task, Handle};
    ///
    /// let mut task = Task::new("example_task", async { 42 });
    /// let mut handle = task.create_handle();
    /// // run executor
    /// # const TASK_ARRAY_SIZE: usize = 1;
    /// # let mut executor = Executor::<TASK_ARRAY_SIZE>::new();
    /// # let _ = executor.spawn(&mut task, &mut handle);
    /// # executor.run();
    ///
    /// assert!(handle.value.is_some_and(|v| v == 42));
    /// ```
    pub(crate) fn link_handle(&mut self, handle: &'a mut Handle<F::Output>) {
        self.handle = Some(handle);
    }
}

impl<T: Future> Future for Task<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        // SAFETY:
        // 1. `this.future` is never moved out of `Runner` after this line.
        // 2. `this.future` is not used to create a `Pin<&mut T>` anywhere else.
        let mut future = unsafe { Pin::new_unchecked(&mut this.future) };
        let res = ready!(future.as_mut().poll(cx));

        if let Some(handle) = this.handle.as_mut() {
            handle.value = Some(res);
        }

        Poll::Ready(())
    }
}

pub(crate) trait TaskName {
    fn name(&self) -> Option<&str>;
}

impl<T: Future> TaskName for Task<'_, T> {
    fn name(&self) -> Option<&str> {
        self.name
    }
}

pub(crate) trait TaskFuture: Future<Output = ()> + TaskName {}

impl<T: Future> TaskFuture for Task<'_, T> {}
