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
