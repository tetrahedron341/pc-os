use crate::arch::{
    cpu::{this_cpu, Context, Registers},
    memory::{phys_to_virt, Page, VirtAddr},
};
use alloc::vec;
use alloc::vec::Vec;
use core::{arch::asm, convert::TryInto, task::Poll};
use goblin::elf64::{
    header::{Header, SIZEOF_EHDR},
    program_header::{ProgramHeader, PT_LOAD, SIZEOF_PHDR},
};

pub mod space;

pub const PROCESS_START: VirtAddr = VirtAddr::new_truncate(0x1000_0000);
pub const STACK_TOP: VirtAddr = VirtAddr::new_truncate(0x1_0000_0000);

pub enum ProcessState {
    Running,
    Runnable,
    Killed,
}

pub struct Process {
    pub kernel_stack: Vec<u8>,
    pub state: ProcessState,
    pub space: crate::arch::memory::space::Space,
    pub context: *mut crate::arch::cpu::Context,
}

unsafe impl Send for Process {}

impl core::future::Future for Process {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        x86_64::instructions::interrupts::disable();
        let cpu = this_cpu();
        let p = self.get_mut();
        cpu.run_process(p);

        match p.state {
            ProcessState::Running => panic!("Yielded process in `Running` state!"),
            ProcessState::Runnable => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            ProcessState::Killed => Poll::Ready(()),
        }
    }
}

pub fn create_process_from_elf(data: &[u8]) -> Result<Process, alloc::string::String> {
    let header: Header = unsafe {
        // Safety: There is no invalid state of `Header`, and the `try_into` will make sure the array
        // is correctly sized.
        core::mem::transmute_copy::<[u8; SIZEOF_EHDR], Header>(
            &data[..SIZEOF_EHDR].try_into().unwrap(),
        )
    };
    if &header.e_ident[0..4] != b"\x7FELF" {
        return Err("ELF64 Format Error: Magic number mismatch".into());
    }
    if header.e_ident[4] != 2 {
        return Err("ELF64 Format Error: 32-bit ELF file recieved".into());
    }

    let program_headers = {
        let mut v = Vec::with_capacity(header.e_phnum as usize);
        for i in 0..header.e_phnum as usize {
            let offset = SIZEOF_EHDR + i * SIZEOF_PHDR;
            // Safety: The program header struct has no invalid states, and the array dereference already check to make sure the struct is in bounds
            let entry = unsafe {
                core::mem::transmute::<[u8; SIZEOF_PHDR], ProgramHeader>(
                    data[offset..offset + SIZEOF_PHDR].try_into().unwrap(),
                )
            };
            v.push(entry);
        }
        v
    };

    for pheader in program_headers {
        if pheader.p_type == PT_LOAD {
            if pheader.p_vaddr != PROCESS_START.as_u64() {
                return Err("ELF64 Format Error: Invalid p_vaddr".into());
            }
            let program_data = &data
                [pheader.p_offset as usize..pheader.p_offset as usize + pheader.p_filesz as usize];
            return Ok(create_process(program_data, PROCESS_START));
        }
    }

    Err("ELF64 Format Error: Missing `PT_LOAD` section".into())
}

pub fn create_process(code: &[u8], entry: VirtAddr) -> Process {
    let mut space = crate::arch::memory::space::Space::new();
    // Copy the code into memory
    for (i, code_chunk) in code.chunks(4096).enumerate() {
        let code_frame = crate::arch::memory::allocate_frame().expect("Out of memory");
        let copy_page = Page::from_start_address(crate::arch::memory::phys_to_virt(
            code_frame.start_address(),
        ))
        .unwrap();
        let target_chunk = unsafe {
            core::slice::from_raw_parts_mut(
                copy_page.start_address().as_mut_ptr(),
                code_chunk.len(),
            )
        };
        target_chunk.copy_from_slice(code_chunk);

        let target_page = Page::from_start_address(PROCESS_START + i * 4096).unwrap();
        unsafe {
            use x86_64::structures::paging::{Mapper, PageTableFlags};
            let mut fa = crate::arch::memory::FRAME_ALLOCATOR.get().unwrap().lock();
            space
                .page_table()
                .map_to(
                    target_page,
                    code_frame,
                    PageTableFlags::PRESENT
                        | PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::WRITABLE,
                    &mut *fa,
                )
                .unwrap()
                .ignore();
        }
    }

    let mut kernel_stack = vec![0u8; 1024];
    let mut sp = kernel_stack.len();
    // Put an interrupt stack frame at the top of the stack so we can `iret` into user mode
    let isf = x86_64::structures::idt::InterruptStackFrameValue {
        instruction_pointer: entry,
        cpu_flags: 1 << 9, // IF enabled
        code_segment: crate::arch::gdt::SELECTORS.user_code_selector.0 as u64,
        stack_segment: crate::arch::gdt::SELECTORS.user_data_selector.0 as u64,
        stack_pointer: STACK_TOP,
    };
    let isf_bytes = unsafe {
        core::mem::transmute::<
            _,
            [u8; core::mem::size_of::<x86_64::structures::idt::InterruptStackFrameValue>()],
        >(isf)
    };
    let isf_len = isf_bytes.len();
    sp -= isf_len;
    kernel_stack[sp..sp + isf_len].copy_from_slice(&isf_bytes);

    /// Empty function that just `iret`s
    #[naked]
    unsafe extern "C" fn trapret() {
        asm! {"iretq", options(noreturn)}
    }

    let context = Context {
        registers: Registers {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
        },
        rip: trapret as *const fn() as u64,
    };
    let context_bytes =
        unsafe { core::mem::transmute::<_, [u8; core::mem::size_of::<Context>()]>(context) };
    let ctx_len = core::mem::size_of::<Context>();
    sp -= ctx_len;
    kernel_stack[sp..sp + ctx_len].copy_from_slice(&context_bytes);
    let context = (&mut kernel_stack[sp] as *mut u8).cast::<Context>();

    // We need to create a stack for the user
    const STACK_FRAMES: usize = 4;
    for i in 0..STACK_FRAMES {
        let frame = crate::arch::memory::allocate_frame().expect("Out of memory");
        {
            // Zero out the stack
            let frame_slice = unsafe {
                let ptr = phys_to_virt(frame.start_address()).as_mut_ptr::<u8>();
                core::slice::from_raw_parts_mut(ptr, 4096)
            };
            frame_slice.fill(0);
        }
        let target_page = Page::from_start_address(STACK_TOP - (i + 1) * 4096).unwrap();
        unsafe {
            use x86_64::structures::paging::{Mapper, PageTableFlags};
            let mut fa = crate::arch::memory::FRAME_ALLOCATOR.get().unwrap().lock();
            space
                .page_table()
                .map_to(
                    target_page,
                    frame,
                    PageTableFlags::PRESENT
                        | PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::WRITABLE,
                    &mut *fa,
                )
                .unwrap()
                .ignore()
        }
    }

    Process {
        kernel_stack,
        state: ProcessState::Runnable,
        space,
        context,
    }
}
