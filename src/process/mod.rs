pub mod executor;
mod process;
mod registers;

pub use process::*;

use alloc::vec::Vec;
use crate::file::ustar::UstarFile;
use registers::Registers;
use spin::Mutex;

pub const PROCESS_START: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0x1000_0000);
pub const STACK_TOP: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0xFFFF_FFFF);
pub const STACK_BOTTOM: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0xF000_0000);

/// A pointer to the process executor. This is used for syscalls such as exit, fork, exec, etc.
pub static EXECUTOR: Mutex<Option<crate::process::executor::Executor>> = Mutex::new(None);

pub fn register_executor(executor: crate::process::executor::Executor) {
    EXECUTOR.lock().replace(executor);
}

/// Enters user mode. Only returns when all processes have ended.
pub fn user_mode(fs: Vec<UstarFile>, mut paging_service: crate::memory::PagingService) -> ! {
    let init_exec = fs.iter().find(|f| {
        f.file_name() == "init"
    }).expect("Missing `init` initialization program");

    let init_proc = process::create_process_from_elf(&mut paging_service, init_exec.data()).unwrap();

    let executor = executor::Executor::new(init_proc, PROCESS_START, paging_service);
    register_executor(executor);

    EXECUTOR.lock().as_mut().unwrap().run()
}

//     let user_rsp = {
//         let user_stack_frame = paging_service.frame_allocator.allocate_frame().unwrap();
//         let user_stack_page = Page::containing_address(VirtAddr::new(user_stack_frame.start_address().as_u64()));
//         let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::NO_EXECUTE;
//         unsafe {
//             paging_service.mapper.map_to(user_stack_page, user_stack_frame, flags, &mut paging_service.frame_allocator).unwrap().flush()
//         }

//         user_stack_page.start_address().as_u64() + user_stack_page.size() - 8
//     };

//     let target_ip = parsed_program.header.pt2.entry_point();
//     let mut ss = crate::gdt::GDT.1.user_data_selector;
//     ss.set_rpl(x86_64::PrivilegeLevel::Ring3);
//     let mut cs = crate::gdt::GDT.1.user_code_selector;
//     cs.set_rpl(x86_64::PrivilegeLevel::Ring3);

//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let ss = ss.0 as u64;
//         let cs = cs.0 as u64;
//         let rip = target_ip as u64;
//         unsafe {
//             asm!(
//                 "push {ss}",
//                 "push {rsp}",
//                 "pushf",
//                 "push {cs}",
//                 "push {rip}",
//                 "iretq",

//                 ss = in(reg) ss,
//                 rsp = in(reg) user_rsp,
//                 cs = in(reg) cs,
//                 rip = in(reg) rip,
//             );
//         }
//     });
// }