use crate::arch::memory::PhysFrame;
use alloc::vec::Vec;
use core::convert::TryInto;
use goblin::elf64::{
    header::{Header, SIZEOF_EHDR},
    program_header::{ProgramHeader, PT_LOAD, SIZEOF_PHDR},
};

pub mod space;

pub const PROCESS_START: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0x1000_0000);
pub const STACK_TOP: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0xFFFF_FFFF);
pub const STACK_BOTTOM: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0xF000_0000);

pub struct Process {
    pub code_len: u32,
    pub frames: Vec<PhysFrame>,
    pub kernel_stack: Vec<u8>,
    pub stack_frames: Vec<PhysFrame>,
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
            // let program_data = &data
            //     [pheader.p_offset as usize..pheader.p_offset as usize + pheader.p_filesz as usize];
            // return Ok(create_process(paging_service, program_data, PROCESS_START));
            todo!()
        }
    }

    Err("ELF64 Format Error: Missing `PT_LOAD` section".into())
}
