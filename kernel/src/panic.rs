use core::{panic::PanicInfo, sync::atomic::AtomicPtr};

pub mod unwind;

static PANIC_HOOK: AtomicPtr<()> = AtomicPtr::new(default_panic_handler as _);

/// # Safety:
/// you really shouldnt have to use this more than once. if i do end up having to do that ill rewrite this.
pub unsafe fn set_hook(hook: fn(&PanicInfo) -> !) {
    let hookptr = hook as *mut _;
    log::info!("hookptr = {hookptr:?}");
    PANIC_HOOK.store(hookptr, core::sync::atomic::Ordering::SeqCst);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::log::set_auto_flush(true);
    log::error!("PANIC!");
    log::error!("{:?}", info);
    let hook = PANIC_HOOK.load(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        if hook.is_null() {
            log::warn!("PANIC_HOOK is null, using default handler");
            default_panic_handler(info)
        } else {
            let hook: fn(&PanicInfo) -> ! = core::mem::transmute(hook); // holy unsafe, batman!
            log::info!("hook = {:?}", hook as *const ());
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

    log::error!("{}", info);
    log::error!("{}", info);

    if info.can_unwind() {
        unsafe {
            let rbp: *const u64;
            core::arch::asm!("mov {rbp}, rbp", rbp = out(reg) rbp);
            unwind::unwind_by_rbp(rbp);
        }
    }

    crate::arch::loop_forever();
}
