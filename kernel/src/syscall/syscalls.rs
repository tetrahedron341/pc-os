use libkernel::{SyscallOp, SyscallOpCode, SyscallStatus};

pub(super) fn syscall_dispatch(op: u64, ptr: *mut u8) -> SyscallStatus {
    crate::serial_println!("Syscall!");
    crate::serial_println!("Operation: {}", op);
    crate::serial_println!("Target: {:p}", ptr);

    // Safety: The only way we can be in a syscall is if we are returning from user mode.
    // The only way we could have been in user mode is if `Executor.run()` was called.
    // If `Executor.run()` was called, then that call will never return so it's fine to take back the lock like this.
    unsafe {
        crate::process::EXECUTOR.force_unlock();
    }

    if let Ok(op) = SyscallOp::try_from(op) {
        match op.opcode {
            SyscallOpCode::Ping => {
                crate::println!("Ping!");
                SyscallStatus::Ok
            }
            SyscallOpCode::GetKbdCode => {
                crate::println!("TODO: Keyboard syscall");
                SyscallStatus::Error
            }

            SyscallOpCode::Exit => {
                let mut executor = crate::process::EXECUTOR.lock();
                let executor = executor.as_mut().unwrap();
                executor.exit_current_process();
                executor.run()
            }
        }
    } else {
        SyscallStatus::InvalidOp
    }
}
