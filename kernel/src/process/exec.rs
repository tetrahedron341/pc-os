use super::Process;
use crate::{
    arch::{
        cpu::{Context, Registers},
        memory::{phys_to_virt, Page, VirtAddr},
    },
    process::ProcessState,
};
use alloc::vec;
use core::{arch::asm, convert::TryInto};
use goblin::elf64::{
    header::{Header, SIZEOF_EHDR},
    program_header::{ProgramHeader, PT_LOAD, SIZEOF_PHDR},
};

const STACK_TOP: VirtAddr = VirtAddr::new_truncate(0x1000_0000_0000);

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
        use plain::Plain;
        ProgramHeader::slice_from_bytes_len(
            &data[SIZEOF_EHDR..][..header.e_phnum as usize * SIZEOF_PHDR],
            header.e_phnum as usize,
        )
        .unwrap()
    };

    let load_segments = program_headers.iter().filter_map(|ph| {
        if ph.p_type != PT_LOAD {
            return None;
        }
        assert!(
            ph.p_vaddr < 0xffff_8000_0000_0000,
            "Cannot load into kernel memory"
        );
        Some(LoadSegment {
            data: &data[ph.p_offset as usize..][..ph.p_filesz as usize],
            va: VirtAddr::new(ph.p_vaddr),
        })
    });

    Ok(create_process(load_segments, VirtAddr::new(header.e_entry)))
}

#[derive(Debug)]
struct LoadSegment<'a> {
    data: &'a [u8],
    va: VirtAddr,
}

fn create_process<'a, I>(load_segments: I, entry: VirtAddr) -> Process
where
    I: Iterator<Item = LoadSegment<'a>>,
{
    let mut space = crate::arch::memory::space::Space::new();

    // Copy the code into memory
    for seg in load_segments {
        let code = seg.data;
        let start = seg.va;

        log::info!("LOAD@{start:X?} LEN:{}", code.len());

        for page_start in
            (start.align_down(4096u64)..(start + code.len()).align_up(4096u64)).step_by(4096)
        {
            let to_page = Page::from_start_address(page_start).unwrap();
            let code_offset = page_start.as_u64().saturating_sub(start.as_u64()) as usize;
            // If the segment is non-page aligned, this is how many zeroes we need before the segment.
            let target_offset = start.as_u64().saturating_sub(page_start.as_u64()) as usize;
            let target_len = (code.len() - code_offset).min(4096);

            log::trace!(
                "ps: {:X}  co: {code_offset}  to: {target_offset}  tl: {target_len}",
                page_start.as_u64()
            );

            let load_frame = crate::arch::memory::allocate_frame().expect("Out of memory");
            // Map this frame into the process' address space
            unsafe {
                use x86_64::structures::paging::{Mapper, PageTableFlags};
                let mut fa = crate::arch::memory::FRAME_ALLOCATOR.get().unwrap().lock();
                space
                    .page_table()
                    .map_to(
                        to_page,
                        load_frame,
                        PageTableFlags::PRESENT
                            | PageTableFlags::USER_ACCESSIBLE
                            | PageTableFlags::WRITABLE,
                        &mut *fa,
                    )
                    .unwrap()
                    .ignore();
            }

            let copy_page = Page::from_start_address(crate::arch::memory::phys_to_virt(
                load_frame.start_address(),
            ))
            .unwrap();
            let target_chunk = unsafe {
                core::slice::from_raw_parts_mut(copy_page.start_address().as_mut_ptr(), 4096)
            };
            target_chunk[..target_offset].fill(0);
            target_chunk[target_offset..target_offset + target_len]
                .copy_from_slice(&code[code_offset..code_offset + target_len]);
            target_chunk[target_offset + target_len..].fill(0);
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
