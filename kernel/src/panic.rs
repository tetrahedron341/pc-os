use core::panic::PanicInfo;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    // The kernel will never return from a panic anyways, and printing panic
    // information takes priority over being in a usable state afterwards.
    // FIXME: When we implement multiprocessing, we need to signal all other threads to stop execution first.
    unsafe {
        crate::video::console::CONSOLE.force_unlock();
        crate::serial::SERIAL1.force_unlock();
    }

    crate::serial_println!("{}", info);
    crate::println!("{}", info);

    crate::arch::loop_forever();
}

#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    crate::test::test_panic_handler(info);
}
