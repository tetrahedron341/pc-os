mod dispatch;
pub use dispatch::syscall_dispatch;

use crate::uapi::*;
use core::convert::TryFrom;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SyscallOpCode {
    /// Print out "Ping!" to the console screen
    Ping = SYS_PING as u8,
    PutChar = SYS_PUTCHAR as u8,
    GetKbdCode = SYS_GETCHAR as u8,
    SleepMs = SYS_SLEEP_MS as u8,

    /// Exits the current process
    Exit = SYS_EXIT as u8,
}

impl TryFrom<u8> for SyscallOpCode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SyscallOpCode::*;
        match value {
            SYS_PING => Ok(Ping),
            SYS_PUTCHAR => Ok(PutChar),
            SYS_GETCHAR => Ok(GetKbdCode),
            SYS_SLEEP_MS => Ok(SleepMs),

            SYS_EXIT => Ok(Exit),
            _ => Err(()),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallOp {
    pub opcode: SyscallOpCode,
    pub arg_u8: u8,
    pub arg_u16: u16,
    pub arg_u32: u32,
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
            opcode,
            arg_u8,
            arg_u16,
            arg_u32,
        } = unsafe { core::mem::transmute(value) };
        let opcode = SyscallOpCode::try_from(opcode)?;
        Ok(SyscallOp {
            opcode,
            arg_u8,
            arg_u16,
            arg_u32,
        })
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallStatus {
    Ok = 0,
    Error = 0x8000000000000000,
    InvalidOp = 0xFFFFFFFFFFFFFFFF,
}

impl From<SyscallStatus> for u64 {
    fn from(val: SyscallStatus) -> Self {
        val as u64
    }
}

// /// Triggers a syscall.
// #[no_mangle]
// pub extern "C" fn syscall(op: SyscallOp, ptr: *mut u8) -> SyscallStatus {
//     let status: u64;
//     unsafe {
//         asm!("syscall", inout("r14") core::mem::transmute::<_,u64>(op) => _, inout("r15") ptr => status);

//         core::mem::transmute(status)
//     }
// }
