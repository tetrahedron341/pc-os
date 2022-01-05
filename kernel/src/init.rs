use alloc::vec::Vec;

use crate::boot::BootModule;
use crate::file;
use crate::println;
use crate::video;

pub struct InitServices {
    pub modules: Vec<BootModule>,
}

pub fn kernel_main(init_services: InitServices) -> ! {
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

    println!("Loaded boot modules: {:#?}", init_services.modules);
    let initrd = init_services
        .modules
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

    // crate::process::user_mode(fs, paging_service)
    panic!("kernel_main finished successfully");
}
