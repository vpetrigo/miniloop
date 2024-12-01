use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let task_array_size = env::var("MINILOOP_TASK_ARRAY_SIZE").unwrap_or(String::from("1"));
    let dest_path = Path::new(&out_dir).join("task_array_size.inc");

    fs::write(
        &dest_path,
        format!("const TASK_ARRAY_SIZE: usize = {task_array_size};\n"),
    )
    .unwrap();
}
