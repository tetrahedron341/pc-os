use argh::FromArgs;
use json::JsonValue;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const RUN_ARGS: &[&str] = &[
    "-no-shutdown",
    "-no-reboot",
    "-s",
    "-serial",
    "stdio",
    "-vga",
    "std",
];

const TEST_ARGS: &[&str] = &[
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial",
    "stdio",
];

const QEMU_EXIT_SUCCESS_CODE: i32 = 0x10;

const KERNEL_CRATE_NAME: &str = "kernel";

#[derive(FromArgs)]
/// Builds the kernel.
struct Args {
    #[argh(positional)]
    /// path to the kernel binary file
    kernel_binary_path: PathBuf,

    #[argh(switch)]
    /// do not run the kernel in QEMU
    no_boot: bool,
}

fn main() {
    let Args {
        kernel_binary_path,
        no_boot,
    } = argh::from_env();

    let kernel_binary_path = kernel_binary_path.canonicalize().unwrap();
    let kernel_parent = kernel_binary_path.parent().unwrap();
    let is_doctest = kernel_parent
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("rustdoctest");
    let is_test = is_doctest || kernel_parent.ends_with("deps");

    let uefi = create_disk_images(&kernel_binary_path);

    if no_boot {
        println!("Created disk image at `{}`", uefi.display());
        return;
    }

    let ovmf_path: PathBuf = std::env::var("OVMF_FD").unwrap().into();
    let ovmf_path = ovmf_path.canonicalize().unwrap();

    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-drive")
        .arg(format!("format=raw,file={}", uefi.display()))
        .args(["-bios", ovmf_path.to_str().unwrap()]);

    let exit_status = if is_test {
        run_cmd.args(TEST_ARGS);
        if let Some(code) = run_cmd.status().unwrap().code() {
            if code == (QEMU_EXIT_SUCCESS_CODE << 1) | 1 {
                println!("Tests passed successfully");
                0
            } else {
                println!("Tests failed: status code {}", code);
                code
            }
        } else {
            println!("Process was killed");
            -1
        }
    } else {
        run_cmd.args(RUN_ARGS);
        run_cmd.status().unwrap().code().unwrap()
    };

    std::process::exit(exit_status)
}

fn create_disk_images(kernel_binary_path: &Path) -> PathBuf {
    let metadata = cargo_metadata();
    assert!(metadata.is_object());
    let bootloader_manifest_path = locate_bootloader(&metadata, "bootloader").unwrap();
    let kernel_manifest_path = locate_kernel_manifest(&metadata).unwrap();
    let target_directory = target_directory(&metadata).unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest_path.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest_path);
    build_cmd.arg("--kernel-binary").arg(&kernel_binary_path);
    build_cmd.arg("--target-dir").arg(target_directory);
    build_cmd
        .arg("--out-dir")
        .arg(kernel_binary_path.parent().unwrap());

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }

    let kernel_binary_name = kernel_binary_path.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_binary_path
        .parent()
        .unwrap()
        .join(format!("boot-uefi-{}.img", kernel_binary_name));
    if !disk_image.exists() {
        panic!(
            "Disk image does not exist at {} after bootloader build",
            disk_image.display()
        );
    }
    disk_image
}

fn cargo_metadata() -> JsonValue {
    let metadata = Command::new(env!("CARGO"))
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .output()
        .unwrap()
        .stdout;
    json::parse(std::str::from_utf8(&metadata).unwrap()).unwrap()
}

fn locate_bootloader(metadata: &JsonValue, bootloader_crate_name: &str) -> Option<PathBuf> {
    let kernel_package = metadata["packages"]
        .members()
        .find(|pkg| pkg["name"] == KERNEL_CRATE_NAME)?;
    let kernel_id = kernel_package["id"].as_str()?;
    let kernel_resolve = metadata["resolve"]["nodes"]
        .members()
        .find(|r| r["id"] == kernel_id)?;
    let dependency = kernel_resolve["deps"]
        .members()
        .find(|d| d["name"] == bootloader_crate_name)?;
    let bootloader_id = dependency["pkg"].as_str()?;
    let bootloader_package = metadata["packages"]
        .members()
        .find(|pkg| pkg["id"] == bootloader_id)?;
    bootloader_package["manifest_path"]
        .as_str()
        .map(PathBuf::from)
}

fn locate_kernel_manifest(metadata: &JsonValue) -> Option<PathBuf> {
    let kernel_package = metadata["packages"]
        .members()
        .find(|pkg| pkg["name"] == KERNEL_CRATE_NAME)?;
    kernel_package["manifest_path"].as_str().map(PathBuf::from)
}

fn target_directory(metadata: &JsonValue) -> Option<PathBuf> {
    metadata["target_directory"].as_str().map(PathBuf::from)
}