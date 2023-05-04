use std::process;

fn main() {
    process::Command::new("make")
        .current_dir("./initrd")
        .spawn()
        .unwrap();
}