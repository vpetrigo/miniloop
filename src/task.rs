use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;

/// A `Task` represents a named asynchronous operation.
///
/// # Lifetimes
///
/// - `'a`: The lifetime of the references within the `Task`.
///
/// # Fields
///
/// - `name`: A string slice that holds the name of the task.
/// - `future`: A future that is boxed on the stack, representing the
///    asynchronous operation associated with the task.
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
    pub name: &'a str,
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

pub struct StackBox<'a, T: ?Sized> {
    pub value: RefCell<Pin<&'a mut T>>,
}

impl<'a, T: ?Sized> StackBox<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        StackBox {
            value: RefCell::new(unsafe { Pin::new_unchecked(value) }),
        }
    }
}

pub type StackBoxFuture<'a> = StackBox<'a, dyn Future<Output = ()> + 'a>;
