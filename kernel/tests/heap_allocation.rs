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

    pc_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    pc_os::test_panic_handler(info);
}

#[test_case]
fn boxed_values() {
    use alloc::boxed::Box;
    let a = Box::new(1729);
    let b = Box::new(*b"hello");
    assert_eq!(*a, 1729);
    assert_eq!(*b, *b"hello");
}

#[test_case]
fn large_vec() {
    use alloc::vec::Vec;
    let n = 1000;
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i);
    }
    assert_eq!(v.iter().sum::<u64>(), (n-1) * n / 2);
}

#[test_case]
fn many_boxes() {
    use alloc::boxed::Box;
    let a = Box::new(-1);
    for i in 0..pc_os::allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*a, -1);
}