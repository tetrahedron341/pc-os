#![no_std]

pub mod syscall;

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use syscall::exit;

    exit(-42, Some(&mut core::mem::MaybeUninit::uninit()));
    loop {}
}
