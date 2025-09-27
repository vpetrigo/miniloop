use miniloop::executor::Executor;
use miniloop::helpers::yield_me;
use miniloop::task::Task;

fn sleep(s: u64) {
    std::thread::sleep(std::time::Duration::from_secs(s));
}

async fn dummy_func(data: &str) {
    const TICKS: usize = 4;
    let mut counter = 0usize;

    while counter != TICKS {
        sleep(2);
        let now = get_timestamp_sec();
        println!("{now}: {data}");
        yield_me().await;
        counter += 1;
    }
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
    let mut executor = Executor::<4>::new();
    executor.set_pending_callback(pending_print);

    let mut task1 = Task::new("hello", async {
        dummy_func("hello").await;
    });
    let mut handle1 = task1.create_handle();
    let mut task2 = Task::new("world", async {
        dummy_func("world").await;
    });
    let mut handle2 = task2.create_handle();
    let mut task3 = Task::new("hi", async {
        dummy_func("hi").await;
    });
    let mut handle3 = task3.create_handle();
    let mut task4 = Task::new("rust", async {
        dummy_func("rust").await;
    });
    let mut handle4 = task4.create_handle();

    let _ = executor.spawn(&mut task1, &mut handle1);
    let _ = executor.spawn(&mut task2, &mut handle2);
    let _ = executor.spawn(&mut task3, &mut handle3);
    let _ = executor.spawn(&mut task4, &mut handle4);

    executor.run();
    println!("Done!");
    assert!(handle1.value.is_some());
    assert!(handle2.value.is_some());
    assert!(handle3.value.is_some());
    assert!(handle4.value.is_some());
}
