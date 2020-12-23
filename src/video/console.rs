pub static CONSOLE: spin::Mutex<Option<Console>> = spin::Mutex::new(None);

pub type Console = super::vesa::console::Console<'static>;

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(console) = CONSOLE.lock().as_mut() {
            use core::fmt::Write;
            console.write_fmt(args).unwrap();
        } else {
            crate::serial_println!("ERROR: Console not available, using serial as backup");
            crate::serial::_print(args);
        }
    });
}

#[macro_export]
#[deprecated = "Moving on to VESA-based graphics"]
macro_rules! vga_print {
    ($($arg:tt)*) => ($crate::video::console::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
#[deprecated = "Moving on to VESA-based graphics"]
macro_rules! vga_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::video::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}