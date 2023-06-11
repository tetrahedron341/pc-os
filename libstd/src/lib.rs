#![no_std]
#![feature(c_size_t)]
#![feature(lang_items)]

pub use core::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        kernel_uapi::syscall::exit(-1, None);
    }
}

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    let exit_code = main() as i8;
    loop {
        kernel_uapi::syscall::exit(exit_code, None);
    }
}

extern "C" {
    fn main() -> i32;
}

#[lang = "start"]
fn lang_start<T>(main: fn() -> T, _argc: isize, _argv: *const *const u8, _: u8) -> isize {
    main();
    0
}

pub mod process {
    pub fn abort() -> ! {
        loop {
            kernel_uapi::syscall::exit(0, None);
        }
    }
}
