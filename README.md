[![build](https://github.com/vpetrigo/miniloop/actions/workflows/ci.yml/badge.svg)](https://github.com/vpetrigo/miniloop/actions/workflows/ci.yml)
![Crates.io Version](https://img.shields.io/crates/v/miniloop)


# miniloop - simple asynchronous executor

-----------------------------------------

This repository is created as an attempt to clarify some more low-level details about how things work
in Rust asynchronous world.

Create your tasks on the stack, add them to the executor and enjoy!

```rust
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

    let _ = executor.spawn("hello", &mut binding1);
    let _ = executor.spawn("world", &mut binding2);

    executor.run();
    println!("Done!");
}
```

# License

---------

<sup>
This project is licensed under <a href="LICENSE.md">Apache License, Version 2.0</a>
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as
defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or
conditions.
</sub>
