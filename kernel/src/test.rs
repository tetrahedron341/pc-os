use core::sync::atomic::AtomicBool;

use crate::{init::InitServices, panic, serial_print, serial_println};

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    unreachable!("Should have already exited. Are you not using QEMU?")
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    if SUCCESS_ON_PANIC.load(core::sync::atomic::Ordering::SeqCst) {
        exit_qemu(QemuExitCode::Failure)
    } else {
        exit_qemu(QemuExitCode::Success)
    }
}

static SUCCESS_ON_PANIC: AtomicBool = AtomicBool::new(false);

fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[test_panic_handler]");
    let should_panic = SUCCESS_ON_PANIC.load(core::sync::atomic::Ordering::SeqCst);
    if should_panic {
        serial_println!("[success]\n");
        serial_println!("Message: {}\n", info);
        exit_qemu(QemuExitCode::Success);
    } else {
        serial_println!("[failed]\n");
        serial_println!("Error: {}\n", info);
        exit_qemu(QemuExitCode::Failure);
    }
}

pub struct TestMainBuilder {
    should_panic: bool,
    test_main: fn(),
}

impl TestMainBuilder {
    pub fn new(test_main: fn()) -> Self {
        TestMainBuilder {
            should_panic: false,
            test_main,
        }
    }

    pub fn should_panic(&mut self) -> &mut Self {
        self.should_panic = true;
        self
    }

    pub fn build(&mut self) -> impl FnOnce(InitServices) -> ! + '_ {
        |_| {
            unsafe { panic::set_hook(test_panic_handler) };
            SUCCESS_ON_PANIC.store(self.should_panic, core::sync::atomic::Ordering::SeqCst);
            (self.test_main)();
            serial_println!("ERROR: RETURN FROM `test_main()`");
            exit_qemu(QemuExitCode::Failure);
        }
    }
}
