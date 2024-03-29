#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

kernel::kernel_main!(kernel::test::TestMainBuilder::new(test_main).build());

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
    assert_eq!(v.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    use alloc::boxed::Box;
    let a = Box::new(-1);
    for i in 0..kernel::allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*a, -1);
}
