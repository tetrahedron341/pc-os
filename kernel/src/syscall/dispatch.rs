use crate::process::ProcessState;

use super::{SyscallOp, SyscallOpCode, SyscallStatus};

pub fn syscall_dispatch(op: u64, ptr: *mut u8) -> SyscallStatus {
    // crate::serial_print!("Syscall: ");
    if let Ok(op) = SyscallOp::try_from(op) {
        // crate::serial_println!("op: {:?}, ptr: {:#X?}", op, ptr);
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
                crate::serial_println!("TODO: Keyboard syscall");
                SyscallStatus::Error
            }
            SyscallOpCode::SleepMs => {
                let p = x86_64::instructions::interrupts::without_interrupts(|| {
                    let cpu = crate::arch::cpu::this_cpu();
                    cpu.try_take_process()
                        .expect("`sleep_ms` syscall not within a process")
                });
                let p_state = core::mem::replace(&mut p.state, ProcessState::Waiting);
                let waker = match p_state {
                    ProcessState::Running(w) => w,
                    _ => unreachable!(),
                };
                {
                    let mut exec = crate::task::EXECUTOR.get().unwrap().lock();
                    exec.spawn(async {
                        crate::task::timer::wait_n_ticks(100).await;
                        waker.wake();
                    });
                };

                x86_64::instructions::interrupts::without_interrupts(|| {
                    let cpu = crate::arch::cpu::this_cpu();
                    cpu.return_from_process(p);
                });

                SyscallStatus::Ok
            }

            SyscallOpCode::Exit => x86_64::instructions::interrupts::without_interrupts(|| {
                let cpu = crate::arch::cpu::this_cpu();
                let p = cpu
                    .try_take_process()
                    .expect("`exit` syscall not within a process");
                p.state = ProcessState::Killed;
                cpu.return_from_process(p);

                panic!("Tried to run a killed process")
            }),
        }
    } else {
        crate::println!("Invalid operation: {:#X}", op);
        SyscallStatus::InvalidOp
    }
}
