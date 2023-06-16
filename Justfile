set dotenv-load

target_dir := `cargo metadata --format-version 1 | jq .target_directory`

kernel_target := "platforms/x86_64-none.json"
kernel_profile := env_var_or_default("PROFILE", env_var_or_default("KERNEL_PROFILE", "dev"))
kernel_target_dir := target_dir / file_stem(kernel_target) / (if kernel_profile == "dev" { "debug" } else { kernel_profile })

user_target := "platforms/x86_64-pc_os.json"
user_profile := env_var_or_default("PROFILE", env_var_or_default("USER_PROFILE", "dev"))
user_target_dir := target_dir / file_stem(user_target) / (if user_profile == "dev" { "debug" } else { user_profile })

check := "false"

cargo_build := if check == "true" { "cargo clippy" } else { "cargo build" }
cargo_build_test := if check == "true" { "cargo clippy" } else { "cargo test --no-run" }
cargo_guest_flags := "-Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem"

default:
    just --list

clean:
    cargo clean

_cargo_build package target profile: 
    {{cargo_build}} \
        --package {{package}} \
        --target {{target}} \
        --profile {{profile}} \
        {{cargo_guest_flags}}

_cargo_build_test package target test profile: 
    {{cargo_build_test}} \
        --package {{package}} \
        --target {{target}} \
        --test {{test}} \
        --profile {{profile}} \
        {{cargo_guest_flags}}

kernel_path := kernel_target_dir / "main"
kernel: (_cargo_build "kernel" kernel_target kernel_profile)

init_path := user_target_dir / "init"
init: (_cargo_build "init" user_target user_profile)

initrd_includes := "initrd/hello.txt "
initrd_path := target_dir / "initrd.tar"
initrd_dir := target_dir / "initrd"
initrd +files=initrd_includes: init
    mkdir -p {{initrd_dir}}
    cp {{init_path}} {{files}} {{initrd_dir}}
    cd {{initrd_dir}} && tar -cf {{initrd_path}} *
    rm -r {{initrd_dir}}

img_path := target_dir / "pc_os.img"
img_build_dir := target_dir / "img"
limine_prefix := env_var_or_default("LIMINE_PREFIX", "/usr/local")
_make_img kernel initrd: 
    mkdir -p {{img_build_dir}}
    cp image/limine.cfg \
       {{limine_prefix}}/share/limine/BOOTX64.EFI \
       {{limine_prefix}}/share/limine/limine-cd-efi.bin \
       {{limine_prefix}}/share/limine/limine-cd.bin \
       {{limine_prefix}}/share/limine/limine.sys \
       {{img_build_dir}}
    cp {{kernel}} {{img_build_dir}}/kernel.elf
    cp {{initrd}} {{img_build_dir}}/initrd
    xorriso -as mkisofs -b limine-cd.bin -e limine-cd-efi.bin -o {{img_path}} {{img_build_dir}}
    {{limine_prefix}}/bin/limine-deploy {{img_path}}
    rm -r {{img_build_dir}}

img: kernel initrd (_make_img kernel_path initrd_path)

ovmf_path := env_var_or_default("OVMF_PATH", "/usr/share/ovmf/OVMF.fd")
qemu := env_var_or_default("QEMU", "qemu-system-x86_64")
qemu_args := env_var_or_default("QEMU_ARGS", "")
qemu_run_args := "-s -serial stdio -vga std -m 256M -machine q35 -cpu qemu64 -d int -D qemu.log -display none -device rtl8139" + qemu_args
qemu_test_args := "-device isa-debug-exit,iobase=0xf4,iosize=0x04 -s -serial stdio -vga std -m 256M -machine q35 -cpu qemu64 -d int -D qemu.log -display none -device rtl8139" + qemu_args
run disk_image *args: img 
    {{qemu}} \
        -drive file={{disk_image}},format=raw \
        -drive file={{img_path}},format=raw \
        -bios {{ovmf_path}} \
        {{qemu_run_args}} {{args}}

gdb disk_image *args: (run disk_image args "-S")

# Used by `cargo run`
_kernel_runner kernel_path *args: initrd (_make_img (kernel_path) initrd_path)
    @ if [ -z $DISK_IMAGE ]; then echo "Set environment variable DISK_IMAGE"; exit -1; else true; fi
    {{qemu}} \
    -drive file={{img_path}},format=raw \
    -drive file=$DISK_IMAGE,format=raw \
    -bios {{ovmf_path}} \
    {{qemu_test_args}} {{args}} \
    ; if [ $? -eq 33 ]; then true; else exit $?; fi
