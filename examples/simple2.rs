use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};
use miniloop::executor::Executor;
use miniloop::helpers::yield_me;
use std::time::Duration;

fn is_expired(start: u64, delay_s: u64) -> bool {
    get_timestamp_sec() - start < delay_s
}

fn sleep(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

async fn foo() -> Result<String, ()> {
    let now = get_timestamp_sec();

    while is_expired(now, 10) {
        yield_me().await;
        sleep(100);
    }

    Ok("Hello".to_string())
}

async fn bar() -> u32 {
    let now = get_timestamp_sec();

    while is_expired(now, 2) {
        yield_me().await;
        sleep(100);
    }

    100u32 + 200u32
}

fn get_timestamp_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn pending_print(task_name: &str) {
    let now = get_timestamp_sec();
    println!("{now}: Task {task_name} is pending. Waiting for the next tick...");
}

struct Runner<'a, T: Future> {
    future: T,
    handle: Option<&'a mut Handle<T::Output>>,
}

impl<'a, T: Future> Runner<'a, T> {
    fn new(future: T) -> (Self, Handle<T::Output>) {
        let handle = Handle { val: None };
        let runner = Self {
            future,
            handle: None,
        };

        (runner, handle)
    }

    fn link_handle(&mut self, handle: &'a mut Handle<T::Output>) {
        self.handle = Some(handle);
    }
}

impl<T: Future> Future for Runner<'_, T>
where
    T::Output: Debug,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        // SAFETY:
        // 1. `this.future` is never moved out of `Runner` after this line.
        // 2. `this.future` is not used to create a `Pin<&mut T>` anywhere else.
        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        let res = ready!(future.poll(cx));

        if let Some(handle) = this.handle.as_mut() {
            handle.val = Some(res);
        }

        Poll::Ready(())
    }
}

struct Handle<T> {
    val: Option<T>,
}

macro_rules! bind {
    ($e:expr) => {{
        let _binding = async { $e().await };
        _binding
    }};
}

fn main() {
    let mut executor = Executor::new();
    executor.set_pending_callback(pending_print);

    let (mut runner1, mut handle1) = Runner::new(async { foo().await });
    runner1.link_handle(&mut handle1);
    let _ = executor.spawn("foo", &mut runner1);
    let (mut runner2, mut handle2) = Runner::new(async move { bar().await });
    runner2.link_handle(&mut handle2);
    let _ = executor.spawn("bar", &mut runner2);

    executor.run();
    println!("Foo result: {:?}", handle1.val);
    println!("Bar result: {:?}", handle2.val);
}
