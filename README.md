[![build](https://github.com/vpetrigo/miniloop/actions/workflows/ci.yml/badge.svg)](https://github.com/vpetrigo/miniloop/actions/workflows/ci.yml)
[![Crates.io Version](https://img.shields.io/crates/v/miniloop)](https://crates.io/crates/miniloop)

# miniloop - simple asynchronous executor

This repository is created as an attempt to clarify some more low-level details about how things work
in Rust asynchronous world.

The `miniloop` executor creates a statically allocated list of tasks. That number should be available upon a crate
build:

- `MINILOOP_TASK_ARRAY_SIZE`: default value is `1` which means you can schedule a single task within the executor. To
  override that just define an environment variable with the number of tasks you plan to use in your application.

## Configuration

You can set up the environment variable in a shell prior to running the `cargo build` command:

- Linux
  ```shell
  export MINILOOP_TASK_ARRAY_SIZE=10
  ```
- Windows
  ```powershell
  $env:MINILOOP_TASK_ARRAY_SIZE = 10
  ```

Or you can use [configurable environment](https://doc.rust-lang.org/nightly/cargo/reference/config.html#env) feature by
creating a `.cargo/config.toml` file with the following content:

```toml
[env]
MINILOOP_TASK_ARRAY_SIZE = "10"
```

# miniloop in action

Create your tasks on the stack, add them to the executor and enjoy!

```rust
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
    let mut executor = Executor::new();
    executor.set_pending_callback(pending_print);

    let mut task1 = Task::new("hello", async {
        dummy_func("hello").await;
    });
    let mut handle1 = task1.create_handle();
    let mut task2 = Task::new("world", async {
        dummy_func("world").await;
    });
    let mut handle2 = task2.create_handle();

    let _ = executor.spawn(&mut task1, &mut handle1);
    let _ = executor.spawn(&mut task2, &mut handle2);

    executor.run();
    println!("Done!");
    assert!(handle1.value.is_some());
    assert!(handle2.value.is_some());
}
```

# License

<sup>
This project is licensed under <a href="LICENSE.md">Apache License, Version 2.0</a>
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as
defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or
conditions.
</sub>
