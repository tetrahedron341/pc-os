#![feature(start)]
extern crate std;

macro_rules! print {
    ($($arg:tt)*) => (let _ = ::core::fmt::write(&mut $crate::Printer, format_args!($($arg)*)););
}

pub fn main() {
    for _ in 0..3 {
        kernel_uapi::syscall::ping(0, None);
    }

    print!("Hello from userland rust!\n");

    for seconds in 0..=5 {
        print!("{} seconds\n", seconds);
        kernel_uapi::syscall::sleep_ms(1000, None);
    }
}

struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.as_bytes() {
            kernel_uapi::syscall::put_char(*c, None);
        }
        Ok(())
    }
}
