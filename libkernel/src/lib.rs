//! Definitions and constants to help userland programs interface with the kernel.

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
