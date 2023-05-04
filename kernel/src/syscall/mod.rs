mod dispatch;
pub use dispatch::syscall_dispatch;

use crate::uapi::*;
use core::convert::TryFrom;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SyscallOpCode {
    /// Print out "Ping!" to the console screen
    Ping = SYS_PING as u32,
    PutChar = SYS_PUTCHAR as u32,
    GetKbdCode = SYS_GETCHAR as u32,
    SleepMs = SYS_SLEEP_MS as u32,

    /// Exits the current process
    Exit = SYS_EXIT as u32,
}

impl TryFrom<u32> for SyscallOpCode {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
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
    pub args: [u64; 4],
}

impl SyscallOp {
    pub fn new(opcode: u32, args: [u64; 4]) -> Option<Self> {
        let opcode = SyscallOpCode::try_from(opcode).ok()?;
        Some(SyscallOp { opcode, args })
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
