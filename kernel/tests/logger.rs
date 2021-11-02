#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(pc_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use pc_os::{allocator, memory};
    use x86_64::VirtAddr;
    use log::LevelFilter;

    pc_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    pc_os::log::init(LevelFilter::Off, LevelFilter::Info, 64);

    // Clear the screen
    pc_os::print!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    pc_os::test_panic_handler(info);
}

#[test_case]
fn logging() {
    const TARGET: &[u8] = b"[INFO] logger - Test `logging`";

    log::info!("Test `logging`");
    pc_os::log::flush();

    let vga = pc_os::vga::WRITER.lock();
    let actual = {
        let mut buf = [0u8; TARGET.len()];
        let slice = &vga.buffer.chars[23][0..TARGET.len() * 2];
        for i in 0..TARGET.len() {
            buf[i] = slice[i].read().char;
        }
        buf
    };

    assert_eq!(*TARGET, actual, "{:?} != {:?}", core::str::from_utf8(TARGET), core::str::from_utf8(&actual));
}