use argh::FromArgs;
use json::JsonValue;
use std::io::Write;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

static LIMINE_CFG: &str = include_str!("../limine.cfg");

const RUN_ARGS: &[&str] = &["-s", "-serial", "stdio", "-vga", "std", "-m", "1024"];

const TEST_ARGS: &[&str] = &[
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial",
    "stdio",
];

const QEMU_EXIT_SUCCESS_CODE: i32 = 0x10;

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
    let target_directory = target_directory(&metadata).unwrap();
    let limine_dir = PathBuf::from(std::env::var("LIMINE_BIN_DIR").unwrap());

    let image_dir = tempdir::TempDir::new("tmp-runner").unwrap();

    let img = target_directory.join("kernel.img");
    build_image(image_dir.path(), kernel_binary_path, &limine_dir, &img);

    img
}

fn build_image(img_dir: &Path, kernel_binary_path: &Path, limine_bin_dir: &Path, out: &Path) {
    let efi_boot = img_dir.join("EFI").join("BOOT");
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&efi_boot)
        .unwrap();
    std::fs::copy(
        limine_bin_dir.join("BOOTX64.EFI"),
        efi_boot.join("BOOTX64.EFI"),
    )
    .unwrap();
    std::fs::copy(
        limine_bin_dir.join("limine-cd-efi.bin"),
        img_dir.join("limine-cd-efi.bin"),
    )
    .unwrap();
    std::fs::copy(
        limine_bin_dir.join("limine-cd.bin"),
        img_dir.join("limine-cd.bin"),
    )
    .unwrap();
    std::fs::copy(
        limine_bin_dir.join("limine.sys"),
        img_dir.join("limine.sys"),
    )
    .unwrap();

    let mut limine_cfg = std::fs::File::create(img_dir.join("limine.cfg")).unwrap();
    write!(limine_cfg, "{}", LIMINE_CFG).unwrap();

    std::fs::copy(kernel_binary_path, img_dir.join("kernel.elf")).unwrap();

    let initrd = make_initrd();
    std::fs::copy(initrd, img_dir.join("initrd")).unwrap();

    std::process::Command::new("mkisofs")
        .args(["-b", "limine-cd.bin"])
        .args(["-e", "limine-cd-efi.bin"])
        .args(["-o", out.to_str().unwrap()])
        .arg(img_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn make_initrd() -> PathBuf {
    make_libc();
    let initrd_dir = PathBuf::from(std::env::var("INITRD_DIR").unwrap());
    std::process::Command::new("make")
        .current_dir(&initrd_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    initrd_dir.join("initrd.tar")
}

fn make_libc() {
    let libc_dir = PathBuf::from(std::env::var("LIBC_DIR").unwrap());
    std::process::Command::new("make")
        .current_dir(&libc_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
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

fn target_directory(metadata: &JsonValue) -> Option<PathBuf> {
    metadata["target_directory"].as_str().map(PathBuf::from)
}
