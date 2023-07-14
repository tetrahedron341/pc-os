use crate::process::ProcessState;

use kernel_uapi::syscall::{Syscall, SyscallErrorCode, SyscallResult, SyscallResultInner};
use log::info;

pub extern "C" fn syscall_handler(op: &mut Syscall) -> SyscallResult {
    log::trace!("Syscall: {:?}", op);
    match op {
        Syscall::ping {} => {
            info!("Ping!");
            Ok(SyscallResultInner { ping: () }).into()
        }
        Syscall::put_char { c } => {
            let c = char::from(*c);
            if ('\x20'..='\x7E').contains(&c) || c == '\n' {
                crate::print!("{}", c);
                crate::serial_print!("{}", c);
                Ok(SyscallResultInner { put_char: () }).into()
            } else {
                Err(SyscallErrorCode::InvalidArgumentError).into()
            }
        }
        Syscall::get_kbd_code { .. } => {
            unimplemented!()
        }
        Syscall::sleep_ms { .. } => {
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

            Ok(SyscallResultInner { sleep_ms: () }).into()
        }
        Syscall::exit { .. } => x86_64::instructions::interrupts::without_interrupts(|| {
            let cpu = crate::arch::cpu::this_cpu();
            let p = cpu
                .try_take_process()
                .expect("`exit` syscall not within a process");
            p.state = ProcessState::Killed;
            cpu.return_from_process(p);

            panic!("Tried to run a killed process")
        }),
    }
}
