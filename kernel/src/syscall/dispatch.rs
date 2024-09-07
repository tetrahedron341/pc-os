use super::{SyscallOp, SyscallOpCode, SyscallStatus};

pub fn syscall_dispatch(op: u64, ptr: *mut u8) -> SyscallStatus {
    crate::serial_print!("Syscall: ");
    if let Ok(op) = SyscallOp::try_from(op) {
        crate::serial_println!("op: {:?}, ptr: {:#X?}", op, ptr);
        match op.opcode {
            SyscallOpCode::Ping => {
                crate::println!("Ping!");
                SyscallStatus::Ok
            }
            SyscallOpCode::PutChar => {
                let c = char::from(op.arg_u8);
                if ('\x20'..='\x7E').contains(&c) || c == '\n' {
                    crate::print!("{}", c);
                    SyscallStatus::Ok
                } else {
                    SyscallStatus::Error
                }
            }
            SyscallOpCode::GetKbdCode => {
                crate::println!("TODO: Keyboard syscall");
                SyscallStatus::Error
            }

            SyscallOpCode::Exit => {
                // let executor = executor.as_mut().unwrap();
                // executor.exit_current_process();
                // executor.run()
                crate::arch::loop_forever()
            }
        }
    } else {
        crate::println!("Invalid operation: {:#X}", op);
        SyscallStatus::InvalidOp
    }
}
