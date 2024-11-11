use crate::task::Task;
use core::future::Future;
use core::ptr;
use core::task::{Context, RawWaker, RawWakerVTable, Waker};

#[derive(Debug, PartialEq)]
pub enum Error {
    NoFreeSlots,
}

pub struct Executor<'a> {
    tasks: [Option<Task<'a>>; 4],
    index: usize,
    pending_callback: Option<fn(&str)>,
}

impl<'a> Default for Executor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Executor<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: [const { None }; 4],
            index: 0,
            pending_callback: None,
        }
    }

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

    pub fn run(&mut self) {
        loop {
            for i in 0..self.tasks.len() {
                let mut should_remove = false;

                if let Some(task) = self.tasks[i].as_ref() {
                    let mut future = task.future.value.borrow_mut();
                    let waker = create_waker();
                    let context = &mut Context::from_waker(&waker);

                    if future.as_mut().poll(context).is_pending() {
                        if let Some(cb) = self.pending_callback {
                            cb(task.name);
                        }
                    } else {
                        should_remove = true;
                    }
                }

                if should_remove {
                    self.tasks[i] = None;
                }
            }

            if self.tasks.iter().all(Option::is_none) {
                return;
            }
        }
    }
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
