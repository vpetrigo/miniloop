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
//! use miniloop::task::Task;
//!
//! const TASK_ARRAY_SIZE: usize = 1;
//! let mut executor = Executor::<TASK_ARRAY_SIZE>::new();
//!
//! let mut task = Task::new("task", async {
//!     println!("Hello, world!");
//! });
//! let mut handle = task.create_handle();
//!
//! executor.spawn(&mut task, &mut handle).expect("Failed to spawn task");
//! executor.run();
//! ```
//!
//! ### Handling Multiple Tasks
//!
//! ```rust,no_run
//! use miniloop::executor::Executor;
//! use miniloop::task::Task;
//!
//! const TASK_ARRAY_SIZE: usize = 2;
//! let mut executor = Executor::<TASK_ARRAY_SIZE>::new();
//!
//! let mut task1 = Task::new("task1", async {
//!     println!("Task 1 executed");
//! });
//! let mut handle1 = task1.create_handle();
//!
//! let mut task2 = Task::new("task2", async {
//!     println!("Task 2 executed");
//! });
//! let mut handle2 = task2.create_handle();
//!
//! executor.spawn(&mut task1, &mut handle1).expect("Failed to spawn task 1");
//! executor.spawn(&mut task2, &mut handle2).expect("Failed to spawn task 2");
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

pub(crate) mod sbox;

#[cfg(test)]
mod test {
    use super::executor::Executor;
    use super::task::Task;

    use core::future::Future;
    use core::iter::zip;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    const TASK_ARRAY_SIZE: usize = 256;

    struct MyTestFuture(bool);

    impl MyTestFuture {
        const fn default() -> Self {
            Self(false)
        }
    }

    impl Future for MyTestFuture {
        type Output = u8;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            self.get_mut().0 = true;
            Poll::Ready(42u8)
        }
    }

    #[test]
    fn test_one_future() {
        let mut executor = Executor::<TASK_ARRAY_SIZE>::new();
        let mut task = Task::new("my_test_task", MyTestFuture::default());
        let mut handle = task.create_handle();
        let result = executor.spawn(&mut task, &mut handle);
        assert!(result.is_ok());
        executor.run();
        assert!(handle.value.is_some_and(|v| v == 42u8));
    }

    #[test]
    fn test_multiple_futures() {
        let mut task_array =
            [const { Task::new_nameless(MyTestFuture::default()) }; TASK_ARRAY_SIZE];
        let mut handles = [(); TASK_ARRAY_SIZE].map(|()| task_array[0].create_handle());
        let mut executor = Executor::<TASK_ARRAY_SIZE>::new();

        for (task, handle) in zip(&mut task_array, &mut handles) {
            let result = executor.spawn(task, handle);
            assert!(result.is_ok(), "Failed to spawn task");
        }

        // Run the executor
        executor.run();

        // Validate that all tasks completed with the expected return value
        for handle in &handles {
            assert!(
                handle.value.is_some_and(|v| v == 42),
                "Task did not complete with expected value"
            );
        }
    }

    #[test]
    fn test_schedule_too_many_tasks() {
        let mut task_array =
            [const { Task::new_nameless(MyTestFuture::default()) }; TASK_ARRAY_SIZE + 1];
        let mut handles = [(); TASK_ARRAY_SIZE].map(|()| task_array[0].create_handle());
        let mut executor = Executor::<TASK_ARRAY_SIZE>::new();

        for (i, (task, handle)) in zip(&mut task_array, &mut handles).enumerate() {
            let result = executor.spawn(task, handle);

            if i < TASK_ARRAY_SIZE {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_different_return_type_tasks() {
        let mut task1 = Task::new("task1", async { 1u32 });
        let mut handle1 = task1.create_handle();
        let mut task2 = Task::new("task1", async {
            if false {
                return Err(());
            }

            Ok(2u32)
        });
        let mut handle2 = task2.create_handle();
        let mut executor = Executor::<TASK_ARRAY_SIZE>::new();

        let result = executor.spawn(&mut task1, &mut handle1);
        assert!(result.is_ok());
        let result = executor.spawn(&mut task2, &mut handle2);
        assert!(result.is_ok());
        executor.run();

        assert_eq!(handle1.value, Some(1u32));
        assert_eq!(handle2.value, Some(Ok(2u32)));
    }
}
