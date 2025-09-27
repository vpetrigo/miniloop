use miniloop::executor::Executor;
use miniloop::helpers::yield_me;
use miniloop::task::Task;

use core::time::Duration;

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

fn main() {
    let mut executor = Executor::<2>::new();
    executor.set_pending_callback(pending_print);
    let mut task1 = Task::new("foo", foo());
    let mut handle1 = task1.create_handle();
    let mut task2 = Task::new("bar", async { bar().await });
    let mut handle2 = task2.create_handle();

    let _ = executor.spawn(&mut task1, &mut handle1);
    let _ = executor.spawn(&mut task2, &mut handle2);
    executor.run();

    assert!(handle1.value.is_some_and(|v| v.is_ok_and(|s| s == "Hello")));
    assert!(handle2.value.is_some_and(|v| v == 300u32));
}
