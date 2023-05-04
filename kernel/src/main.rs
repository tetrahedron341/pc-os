#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use ::log::*;
use bootloader::BootInfo;

use kernel::*;

const SERIAL_LOG_MIN: LevelFilter = LevelFilter::Info;
const CONSOLE_LOG_MIN: LevelFilter = LevelFilter::Warn;

bootloader::entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let init::InitServices {
        idt_service: _idt_service,
        gdt_service: _gdt_service,
        paging_service,
        modules,
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

    println!("Loaded boot modules: {:#?}", modules);
    let initrd = modules
        .iter()
        .find(|m| m.name == "initrd")
        .expect("Boot module `initrd` not found.");

    let mut fs = file::ustar::get_all_entries(initrd.data);
    for entry in fs.iter() {
        println!("File: {}, Size: {}", entry.file_name(), entry.file_size());
    }
    if let Some(hello) = fs.iter_mut().find(|f| f.file_name() == "hello.txt") {
        let s = {
            use core2::io::Read;
            let mut buf = alloc::vec![0u8; hello.file_size()];
            hello.read(&mut buf).unwrap();
            alloc::string::String::from_utf8_lossy(&buf).into_owned()
        };
        println!("hello.txt contents:\n{}", s)
    }

    syscall::init();

    process::user_mode(fs, paging_service)
}
