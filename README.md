# im very bad at naming things

# prerequisites

0. install rust, a C compiler, and `make`.
1. install [limine](https://github.com/limine-bootloader/limine). instructions are in limine's README. make sure you complete the "Installing Limine binaries" section.
    - if you change the prefix limine was installed in, make sure to set the `LIMINE_PREFIX` variable in `.cargo/config.toml`.
2. install ovmf.
    - on ubuntu, you can install it from the `ovmf` package, which will place the `OVMF.fd` file at `/usr/share/ovmf/OVMF.fd`.
    - if you have `OVMF.fd` placed somewhere other than `/usr/share/ovmf/OVMF.fd`, make sure to change the `OVMF_FD` variable in `.cargo/config.toml`.

# building

use `cargo krun` or `cargo kbuild` as you would do with normal rust projects.

to make the kernel in release mode without running it, use `cargo kimage`.

# testing

use `cargo ktest`.

# acknowledgements

cool projects that i either use directly or took inspiration from. check them out

-   https://os.phil-opp.com/
    -   wouldn't have known where to start without this guide. really cool
-   https://github.com/limine-bootloader/limine
    -   very cool bootloader
