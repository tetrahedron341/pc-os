use core::{panic::PanicInfo, sync::atomic::AtomicPtr};

use crate::serial_println;

pub mod unwind;

static PANIC_HOOK: AtomicPtr<()> = AtomicPtr::new(default_panic_handler as _);

/// # Safety:
/// you really shouldnt have to use this more than once. if i do end up having to do that ill rewrite this.
pub unsafe fn set_hook(hook: fn(&PanicInfo) -> !) {
    let hookptr = hook as *mut _;
    serial_println!("[set_hook] hookptr = {hookptr:?}");
    PANIC_HOOK.store(hookptr, core::sync::atomic::Ordering::SeqCst);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC!");
    serial_println!("{:?}", info);
    let hook = PANIC_HOOK.load(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        if hook.is_null() {
            serial_println!("[panic] WARNING: PANIC_HOOK is null, using default handler");
            default_panic_handler(info)
        } else {
            let hook: fn(&PanicInfo) -> ! = core::mem::transmute(hook); // holy unsafe, batman!
            serial_println!("[panic] hook = {:?}", hook as *const ());
            hook(info)
        }
    }
}

fn default_panic_handler(info: &PanicInfo) -> ! {
    // The kernel will never return from a panic anyways, and printing panic
    // information takes priority over being in a usable state afterwards.
    // FIXME: When we implement multiprocessing, we need to signal all other threads to stop execution first.
    unsafe {
        crate::video::console::CONSOLE.force_unlock();
        crate::serial::SERIAL1.force_unlock();
    }

    crate::serial_println!("{}", info);
    crate::println!("{}", info);

    if info.can_unwind() {
        unwind::unwind_by_rbp();
    }

    crate::arch::loop_forever();
}
