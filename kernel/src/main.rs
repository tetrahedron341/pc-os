#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use ::log::*;
use bootloader::BootInfo;
use core::panic::PanicInfo;

use kernel::*;

const SERIAL_LOG_MIN: LevelFilter = LevelFilter::Info;
const CONSOLE_LOG_MIN: LevelFilter = LevelFilter::Warn;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    // The kernel will never return from a panic anyways, and printing panic
    // information takes priority over being in a usable state afterwards.
    // FIXME: When we implement multiprocessing, we need to signal all other threads to stop execution first.
    unsafe {
        video::console::CONSOLE.force_unlock();
        serial::SERIAL1.force_unlock();
    }

    serial_println!("{}", info);
    println!("{}", info);

    hlt_loop();
}

#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

bootloader::entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let init::InitServices {
        idt_service: _idt_service,
        gdt_service: _gdt_service,
        paging_service,
    } = init::init(boot_info);

    log::init(SERIAL_LOG_MIN, CONSOLE_LOG_MIN, 128);

    #[cfg(test)]
    test_main();

    {
        let mut screen = video::vesa::lock_screen().unwrap();
        let width = screen.width();
        let height = screen.height();
        screen.draw_rect_with(0, 0, width, height, |x, y, _| {
            let u = x as f64 / width as f64;
            let v = y as f64 / height as f64;
            (((1.0 - u) * 255.9) as u8, 0, ((1.0 - v) * 255.9) as u8)
            // let n = x+y;
            // (
            //     if n % 8 < 4 {255} else {0},
            //     if n % 16 < 8 {255} else {0},
            //     if n % 32 < 16 {255} else {0},
            // )
        });
    }

    {
        if let Some(console) = video::console::CONSOLE.lock().as_mut() {
            for r in 0..16 {
                for c in 0..32 {
                    console.write_glyph(r * 32 + c);
                }
                console.newline();
            }
        }
    }

    let (width, height) = {
        let screen = video::vesa::lock_screen().unwrap();
        (screen.width(), screen.height())
    };
    println!();
    println!("Hello from {}x{} VESA!", width, height);

    let mut fs = {
        static INITRD: &[u8] = include_bytes!("../../initrd/initrd.tar");
        file::ustar::get_all_entries(INITRD)
    };
    for entry in fs.iter() {
        println!("File: {}, Size: {}", entry.file_name(), entry.file_size());
    }
    if let Some(hello) = fs.iter_mut().find(|f| f.file_name() == "hello.txt") {
        let s = {
            use bare_io::Read;
            let mut buf = alloc::vec![0u8; hello.file_size()];
            hello.read(&mut buf).unwrap();
            alloc::string::String::from_utf8_lossy(&buf).into_owned()
        };
        println!("hello.txt contents:\n{}", s)
    }

    syscall::init();

    process::user_mode(fs, paging_service)
}
