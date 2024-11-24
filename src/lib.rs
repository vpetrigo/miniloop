//! # Miniloop
//!
//! Miniloop is an educational Rust crate designed to teach the basics of building executors for asynchronous tasks.
//! It provides a simple and comprehensive executor that helps in understanding how futures and task scheduling work under the hood.
//!
//! ## Build variables
//!
//! The `miniloop` executor creates a statically allocated list of tasks. That number should be available upon a crate
//! build:
//!
//! - `MINILOOP_TASK_ARRAY_SIZE`: default value is `1` which means you can schedule a single task within the executor. To
//!   override that just define an environment variable with the number of tasks you plan to use in your application.
//!
//! ## Features
//!
//! - **No Standard Library**: This crate is `#![no_std]`, making it suitable for embedded and
//!   other constrained environments.
//! - **Simple API**: Easy to use API to spawn and run tasks.
//! - **Educational Purpose**: Designed with learning in mind, this crate breaks down the concepts
//!   of executors to their simplest form.
//!
//! ## Modules
//!
//! - [`executor`]: Contains the core executor implementation.
//! - [`helpers`]: Utility functions and types to assist with task management.
//! - [`task`]: Definitions and management of tasks.
//!
//! ## Examples
//!
//! ### Spawning and Running a Single Task
//!
//! ```rust,no_run
//! use miniloop::executor::Executor;
//!
//! let mut executor = Executor::new();
//!
//! let mut task = async {
//!     println!("Hello, world!");
//! };
//!
//! executor.spawn("task", &mut task).expect("Failed to spawn task");
//! executor.run();
//! ```
//!
//! ### Handling Multiple Tasks
//!
//! ```rust,no_run
//! use miniloop::executor::Executor;
//!
//! let mut executor = Executor::new();
//!
//! let mut task1 = async {
//!     println!("Task 1 executed");
//! };
//! let mut task2 = async {
//!     println!("Task 2 executed");
//! };
//!
//! executor.spawn("task1", &mut task1).expect("Failed to spawn task 1");
//! executor.spawn("task2", &mut task2).expect("Failed to spawn task 2");
//!
//! executor.run();
//! ```
//!
//! ## Testing
//!
//! This crate includes several tests demonstrating its usage and ensuring its correctness. The tests cover:
//! - Running a single future
//! - Running multiple futures
//! - Handling the scheduling of too many tasks
//!
//! To run the tests, use the following command:
//! ```sh
//! cargo test
//! ```
//!
//! I hope Miniloop helps you understand the fundamentals of asynchronous programming and task scheduling in Rust.
//! Happy learning!
//!
#![no_std]
pub mod executor;
pub mod helpers;
pub mod task;

#[cfg(test)]
mod test {
    use super::executor::Executor;
    use core::cell::Cell;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};

    include!(concat!(env!("OUT_DIR"), "/task_array_size.inc"));

    struct MyTestFuture(bool);

    impl MyTestFuture {
        const fn default() -> Self {
            Self(false)
        }
    }

    impl Future for MyTestFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            self.get_mut().0 = true;
            Poll::Ready(())
        }
    }

    #[test]
    fn test_one_future() {
        let mut called = Cell::new(false);
        let mut executor = Executor::new();
        let mut task = async {
            *called.get_mut() = true;
        };

        let _ = executor.spawn("task", &mut task);

        executor.run();

        assert!(called.take());
    }

    #[test]
    fn test_multiple_futures() {
        let mut task_array = [const { MyTestFuture::default() }; TASK_ARRAY_SIZE];
        let mut executor = Executor::new();

        for task in &mut task_array {
            let _ = executor.spawn("", task);
        }

        executor.run();
        assert!(task_array.iter().all(|task| task.0));
    }

    #[test]
    fn test_schedule_too_many_tasks() {
        let mut array = [const { MyTestFuture::default() }; TASK_ARRAY_SIZE + 1];
        let mut executor = Executor::new();

        for (i, element) in &mut array.iter_mut().enumerate() {
            let result = executor.spawn("", element);

            if i < TASK_ARRAY_SIZE {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }
}
