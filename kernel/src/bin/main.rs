#![no_std]
#![no_main]

use alloc::{boxed::Box, string::String};
use kernel::{
    println, serial_println,
    video::{
        framebuffer::{GfxRectangle, Pixel},
        Framebuffer,
    },
};

extern crate alloc;
extern crate kernel;

kernel::kernel_main!(main);

fn main(init_services: kernel::init::InitServices) -> ! {
    serial_println!("[main] Initializing console...");

    if let Some(mut fb) = init_services.framebuffer {
        let width = fb.info().width;
        let height = fb.info().height;
        let bg_rect = GfxRectangle::with(width, height, |x, y| {
            let u = x as f64 / width as f64;
            let v = y as f64 / height as f64;
            Pixel::new_rgb(((1.0 - u) * 255.9) as u8, 0, ((1.0 - v) * 255.9) as u8)
            // let n = x + y;
            // Pixel::new_rgb(
            //     if n % 8 < 4 { 255 } else { 0 },
            //     if n % 16 < 8 { 255 } else { 0 },
            //     if n % 32 < 16 { 255 } else { 0 },
            // )
        });
        fb.blit(&bg_rect, (0, 0));

        let mut console = kernel::video::console::Console::new(fb as Box<dyn Framebuffer + Send>);

        for r in 0..16 {
            for c in 0..32 {
                console.write_glyph(r * 32 + c);
            }
            console.newline();
        }

        *kernel::video::console::CONSOLE.lock() = Some(console);
    }

    crate::serial_println!("[main] Console ready");

    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let mut pics = kernel::arch::interrupts::PICS.lock();
        pics.write_masks(0xFC, 0xFF);
    });
    crate::serial_println!("[main] Timer & keyboard interrupts unmasked");

    println!();
    println!("Hello world!");
    println!(
        "HHDM offset: {:#?}",
        kernel::arch::memory::phys_to_virt(kernel::arch::memory::PhysAddr::zero())
    );

    println!("Loaded boot modules: {:#?}", init_services.modules);
    let initrd = init_services
        .modules
        .iter()
        .find(|m| m.name == "initrd")
        .expect("Boot module `initrd` not found.");

    let mut fs = kernel::file::ustar::get_all_entries(initrd.data);
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

    kernel::task::init_executor();

    if let Some(init) = fs.iter_mut().find(|f| f.file_name() == "init") {
        let init_elf = {
            use core2::io::Read;
            let mut buf = alloc::vec![0u8; init.file_size()];
            init.read(&mut buf).unwrap();
            buf
        };

        init_process(&init_elf).unwrap();
    }

    kernel::task::run()
}

fn init_process(init_elf: &[u8]) -> Result<(), String> {
    let p = kernel::process::create_process_from_elf(init_elf)?;

    let mut exec = kernel::task::EXECUTOR.get().unwrap().lock();
    exec.spawn(p);
    exec.spawn(kernel::task::keyboard::print_keypresses());
    Ok(())
}
