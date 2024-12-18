//! # Task Module
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
//! 2. [`StackBox`]: A container for safely pinning a value in place on the stack.
//! 3. [`StackBoxFuture`]: A type alias for a `StackBox` containing a `Future` trait object.
//!
//! ## Examples
//!
//! ### Creating a `Task`
//!
//! ```rust
//! use miniloop::task::Task;
//!
//! let task_name = "example_task";
//! let mut some_future = async { () }; // Example future, replace `()` with actual future logic
//! let task = Task::new(task_name, &mut some_future);
//! ```
//!
//! ### Creating a `StackBox`
//!
//! ```rust
//! use miniloop::task::StackBox;
//!
//! let mut my_value = 42;
//! let stack_box = StackBox::new(&mut my_value);
//! ```
//!
//! ### Creating a `StackBoxFuture`
//!
//! ```rust
//! use miniloop::task::{StackBox, StackBoxFuture};
//!
//! async fn my_async_fn() {
//!     // Your async code here
//! }
//!
//! let mut my_future = async { my_async_fn().await };
//! let stack_box_future: StackBoxFuture = StackBox::new(&mut my_future);
//! ```
use core::cell::OnceCell;
use core::future::Future;
use core::pin::Pin;

/// A `Task` represents a named asynchronous operation.
///
/// # Examples
///
/// ```
/// use miniloop::task::Task;
///
/// let task_name = "example_task";
/// let mut some_future = async { () }; // Example future, replace `()` with actual future logic
/// let task = Task::new(task_name, &mut some_future);
/// ```
pub struct Task<'a> {
    /// A string that holds the name of the task.
    pub name: &'a str,
    /// A future that is boxed on the stack, representing the asynchronous operation associated
    /// with the task.
    pub future: StackBoxFuture<'a>,
}

impl<'a> Task<'a> {
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
    /// use core::future::Future;
    ///
    /// let name = "example_task";
    /// let mut future = async { () };
    /// let task = Task::new(name, &mut future);
    /// ```
    pub fn new(name: &'a str, future: &'a mut impl Future<Output = ()>) -> Self {
        Self {
            name,
            future: StackBox::new(future),
        }
    }

    /// Creates a new `Task` with the specified name and boxed future.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the task.
    /// * `future` - A `StackBoxFuture` holding the future to be executed.
    ///
    /// # Returns
    ///
    /// A new instance of `Task`.
    ///
    /// # Examples
    ///
    /// ```
    /// use miniloop::task::{StackBox, StackBoxFuture, Task};
    /// let name = "example_task";
    /// let mut future = async { () };
    /// let stack_box: StackBoxFuture = StackBox::new(&mut future);
    /// let task = Task::new_box(name, stack_box);
    /// ```
    pub fn new_box(name: &'a str, future: StackBoxFuture<'a>) -> Self {
        Self { name, future }
    }
}

/// A container for holding a pinned reference to a value on the stack.
///
/// The `StackBox` struct provides a way to safely pin a value in place on the stack.
/// A pinned reference means that the value pointed to by the reference cannot be moved.
/// This is important for certain types that rely on stable addresses, such as generators or futures.
///
/// # Type Parameters
/// - `'a`: The lifetime of the reference to the stored value.
/// - `T`: The type of the value to be stored. The type may be dynamically sized (`?Sized`).
///
/// # Example
/// ```
/// use miniloop::task::StackBox;
///
/// // Create a mutable value on the stack
/// let mut my_value = 42;
///
/// // Wrap the value in a StackBox
/// let stack_box = StackBox::new(&mut my_value);
/// ```
pub struct StackBox<'a, T: ?Sized> {
    /// A `OnceCell` containing a pinned mutable reference to the stored value.
    pub value: OnceCell<Pin<&'a mut T>>,
}

impl<T: ?Sized> Default for StackBox<'_, T> {
    fn default() -> Self {
        StackBox {
            value: OnceCell::new(),
        }
    }
}

impl<'a, T: ?Sized> StackBox<'a, T> {
    /// Creates a new `StackBox` containing a pinned reference to the provided value.
    ///
    /// # Arguments
    /// - `value`: A mutable reference to the value to be stored. The reference must have the
    ///   appropriate lifetime `'a`.
    ///
    /// # Returns
    /// A `StackBox` containing a pinned mutable reference to the provided value.
    ///
    /// # Safety
    /// This function uses `Pin::new_unchecked`, which is unsafe because it assumes
    /// that the value being pinned will not move for the duration of the pin.
    /// Ensure that the value cannot be moved out of the `StackBox`.
    ///
    /// # Example
    /// ```
    /// use miniloop::task::StackBox;
    /// let mut my_value = 42;
    /// let stack_box = StackBox::new(&mut my_value);
    /// ```
    pub fn new(value: &'a mut T) -> Self {
        let new_box = StackBox::default();
        new_box
            .value
            .get_or_init(|| unsafe { Pin::new_unchecked(value) });

        new_box
    }
}

/// A type alias for a `StackBox` containing a `Future` trait object.
///
/// The `StackBoxFuture` type is a convenient way to create a stack-based pinned
/// future. This allows futures to be stored and run on the stack rather than
/// being allocated on the heap, which can be useful in certain performance-sensitive
/// scenarios.
///
/// # Type Parameters
/// - `'a`: The lifetime of the reference to the stored future.
///
/// # Example
/// ```
/// use miniloop::task::{StackBox, StackBoxFuture};
///
/// async fn my_async_fn() {
///     // Your async code here
/// }
///
/// // Create a mutable future
/// let mut my_future = async { my_async_fn().await };
///
/// // Wrap the future in a StackBoxFuture
/// let stack_box_future: StackBoxFuture = StackBox::new(&mut my_future);
/// ```
pub type StackBoxFuture<'a> = StackBox<'a, dyn Future<Output = ()> + 'a>;
