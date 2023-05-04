use core::convert::TryFrom;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SyscallOpCode {
    /// Print out "Ping!" to the console screen
    Ping = 0,
    GetKbdCode = 1,

    /// Exits the current process
    Exit = 127,
}

impl TryFrom<u8> for SyscallOpCode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SyscallOpCode::*;
        match value {
            0 => Ok(Ping),
            1 => Ok(GetKbdCode),

            127 => Ok(Exit),
            _ => Err(())
        }
    }
}

#[repr(C, packed)]
pub struct SyscallOp {
    opcode: SyscallOpCode,
    arg_u8: u8,
    arg_u16: u16,
    arg_u32: u32,
}

impl TryFrom<u64> for SyscallOp {
    type Error = ();
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        #[repr(C, packed)]
        #[allow(dead_code)]
        struct SyscallOpIntermediate {
            pub opcode: u8,
            pub arg_u8: u8,
            pub arg_u16: u16,
            pub arg_u32: u32,
        }
        let SyscallOpIntermediate {
            opcode, arg_u8, arg_u16, arg_u32
        } = unsafe {core::mem::transmute(value)};
        let opcode = SyscallOpCode::try_from(opcode)?;
        Ok(SyscallOp {
            opcode, arg_u8, arg_u16, arg_u32
        })
    }
}

pub const SYSCALL_OK: u64 = 0;
pub const SYSCALL_ERR: u64 = 0x8000000000000000;
pub const INVALID_OP: u64 = 0xFFFFFFFFFFFFFFFF;

pub(super) fn syscall_dispatch(op: u64, ptr: *mut u8) -> u64 {
    crate::serial_println!("Syscall!");
    crate::serial_println!("Operation: {}", op);
    crate::serial_println!("Target: {:p}", ptr);

    // Safety: The only way we can be in a syscall is if we are returning from user mode.
    // The only way we could have been in user mode is if `Executor.run()` was called.
    // If `Executor.run()` was called, then that call will never return so it's fine to take back the lock like this.
    unsafe { crate::process::EXECUTOR.force_unlock(); }

    if let Ok(op) = SyscallOp::try_from(op) {
        match op.opcode {
            SyscallOpCode::Ping => { crate::println!("Ping!"); SYSCALL_OK },
            SyscallOpCode::GetKbdCode => { crate::println!("TODO: Keyboard syscall"); SYSCALL_ERR },

            SyscallOpCode::Exit => {
                let mut executor = crate::process::EXECUTOR.lock();
                let executor = executor.as_mut().unwrap();
                executor.exit_current_process();
                executor.run()
            },
        }
    } else {
        INVALID_OP
    }
}