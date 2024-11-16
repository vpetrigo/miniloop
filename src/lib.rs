//! # Miniloop
//!
//! Miniloop is an educational Rust crate designed to teach the basics of building executors for asynchronous tasks.
//! It provides a simple and comprehensive executor that helps in understanding how futures and task scheduling work under the hood.
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
    use super::executor::{Error, Executor};
    use core::cell::{Cell, RefCell};

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
        let called = RefCell::new([const { Cell::new(false) }; 4]);
        let mut executor = Executor::new();
        let mut task1 = async {
            *called.borrow_mut()[0].get_mut() = true;
        };
        let mut task2 = async {
            *called.borrow_mut()[1].get_mut() = true;
        };
        let mut task3 = async {
            *called.borrow_mut()[2].get_mut() = true;
        };
        let mut task4 = async {
            *called.borrow_mut()[3].get_mut() = true;
        };

        let _ = executor.spawn("task1", &mut task1);
        let _ = executor.spawn("task2", &mut task2);
        let _ = executor.spawn("task3", &mut task3);
        let _ = executor.spawn("task4", &mut task4);

        executor.run();

        assert!(called.borrow().iter().all(Cell::get));
    }

    #[test]
    fn test_schedule_too_many_tasks() {
        let mut task1 = async {};
        let mut task2 = async {};
        let mut task3 = async {};
        let mut task4 = async {};
        let mut task5 = async {};
        let mut executor = Executor::new();

        let result = executor.spawn("task1", &mut task1);
        assert!(result.is_ok());
        let result = executor.spawn("task2", &mut task2);
        assert!(result.is_ok());
        let result = executor.spawn("task3", &mut task3);
        assert!(result.is_ok());
        let result = executor.spawn("task4", &mut task4);
        assert!(result.is_ok());
        let result = executor.spawn("task5", &mut task5);
        assert!(result.is_err());
        assert_eq!(Error::NoFreeSlots, result.unwrap_err());
    }
}
