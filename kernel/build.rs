fn main() {
    std::process::Command::new("make")
        .current_dir("../initrd")
        .spawn()
        .unwrap();
}
