# one of those "OS" things

## prerequisites

- rust nightly
- [`just`](https://github.com/casey/just).
- qemu
- [limine](https://github.com/limine-bootloader/limine).
- ovmf

or you could use nix and install the dev environment using `nix develop` or `nix-shell`

## building

everything is available as a `just` target. 
- `just run <DISK_IMAGE> [EXTRA_QEMU_ARGS]`: run the kernel and initrd in qemu
- `just kernel`: build the kernel
- `just initrd [EXTRA_FILES]`: builds the initrd, with optional extra files bundled

everything builds in debug mode by default. to change the build profile, set the environment variable `{KERNEL,USER,}PROFILE=release`

if the justfile cannot find limine or ovmf, set the environment variables `LIMINE_PREFIX` and `OVMF_PATH`. default values are in the `Justfile`.

## testing

TODO

## acknowledgements

-   https://os.phil-opp.com/
-   https://github.com/limine-bootloader/limine
