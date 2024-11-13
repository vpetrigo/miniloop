use miniloop::executor::Executor;
use miniloop::helpers::yield_me;

fn sleep(s: u64) {
    std::thread::sleep(std::time::Duration::from_secs(s));
}

async fn dummy_func(data: &str) {
    let mut counter = 0usize;

    while counter != 4 {
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
    let mut executor = Executor::new();
    executor.set_pending_callback(pending_print);

    let mut binding1 = async {
        dummy_func("hello").await;
    };
    let mut binding2 = async {
        dummy_func("world").await;
    };
    let mut binding3 = async {
        dummy_func("hi").await;
    };
    let mut binding4 = async {
        dummy_func("rust").await;
    };

    let _ = executor.spawn("hello", &mut binding1);
    let _ = executor.spawn("world", &mut binding2);
    let _ = executor.spawn("hi", &mut binding3);
    let _ = executor.spawn("rust", &mut binding4);

    executor.run();
    println!("Done!");
}
