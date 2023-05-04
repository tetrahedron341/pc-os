// use crate::arch::memory::{PhysAddr, VirtAddr};
// use crate::file::ustar::UstarFile;
// use alloc::vec::Vec;
// use core::convert::TryInto;
// use goblin::elf64::{
//     header::{Header, SIZEOF_EHDR},
//     program_header::{ProgramHeader, PT_LOAD, SIZEOF_PHDR},
// };
// // use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame};
// // use x86_64::VirtAddr;

// pub const PROCESS_START: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0x1000_0000);
// pub const STACK_TOP: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0xFFFF_FFFF);
// pub const STACK_BOTTOM: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(0xF000_0000);

// pub struct Process {
//     pub code_len: u32,
//     pub frames: Vec<x86_64::structures::paging::PhysFrame>,
//     pub kernel_stack: Vec<u8>,
//     pub stack_frames: Vec<PhysFrame>,
// }

// pub fn create_process(
//     paging_service: &mut crate::memory::PagingService,
//     code: &[u8],
//     rip: VirtAddr,
// ) -> Process {
//     static PID_COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

//     let code_len = (code.len() / 4096) as u32;
//     let mut code_frames = Vec::new();
//     // Copy the code into memory
//     for code_chunk in code.chunks(4096) {
//         let code_frame = paging_service
//             .frame_allocator
//             .allocate_frame()
//             .expect("Out of memory");
//         const HEAP_END: VirtAddr = VirtAddr::new_truncate(
//             (crate::allocator::HEAP_START + crate::allocator::HEAP_SIZE) as u64,
//         );
//         let copy_page = Page::from_start_address(HEAP_END + 4096u64).unwrap();
//         let flags =
//             PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
//         unsafe {
//             paging_service
//                 .mapper
//                 .map_to(
//                     copy_page,
//                     code_frame,
//                     flags,
//                     &mut paging_service.frame_allocator,
//                 )
//                 .unwrap()
//                 .flush();
//         }
//         let target_chunk = unsafe {
//             core::slice::from_raw_parts_mut(
//                 copy_page.start_address().as_mut_ptr(),
//                 code_chunk.len(),
//             )
//         };
//         target_chunk.copy_from_slice(code_chunk);
//         paging_service.mapper.unmap(copy_page).unwrap().1.flush();
//         code_frames.push(code_frame);
//     }

//     let kernel_stack = Vec::with_capacity(1024);

//     // We need to create a stack for the user
//     const STACK_FRAMES: usize = 4;
//     let user_stack = {
//         let mut v = Vec::with_capacity(STACK_FRAMES);
//         for _ in 0..STACK_FRAMES {
//             v.push(
//                 paging_service
//                     .frame_allocator
//                     .allocate_frame()
//                     .expect("Out of memory"),
//             );
//         }
//         v
//     };

//     Process {
//         code_len,
//         frames: code_frames,
//         kernel_stack,
//         stack_frames: user_stack,
//     }
// }

// pub fn create_process_from_elf(
//     paging_service: &mut crate::memory::PagingService,
//     data: &[u8],
// ) -> Result<Process, alloc::string::String> {
//     let header: Header = unsafe {
//         // Safety: There is no invalid state of `Header`, and the `try_into` will make sure the array
//         // is correctly sized.
//         core::mem::transmute_copy::<[u8; SIZEOF_EHDR], Header>(
//             &data[..SIZEOF_EHDR].try_into().unwrap(),
//         )
//     };
//     if &header.e_ident[0..4] != b"\x7FELF" {
//         return Err("ELF64 Format Error: Magic number mismatch".into());
//     }
//     if header.e_ident[4] != 2 {
//         return Err("ELF64 Format Error: 32-bit ELF file recieved".into());
//     }

//     let program_headers = {
//         let mut v = Vec::with_capacity(header.e_phnum as usize);
//         for i in 0..header.e_phnum as usize {
//             let offset = SIZEOF_EHDR + i * SIZEOF_PHDR;
//             // Safety: The program header struct has no invalid states, and the array dereference already check to make sure the struct is in bounds
//             let entry = unsafe {
//                 core::mem::transmute::<[u8; SIZEOF_PHDR], ProgramHeader>(
//                     data[offset..offset + SIZEOF_PHDR].try_into().unwrap(),
//                 )
//             };
//             v.push(entry);
//         }
//         v
//     };

//     for pheader in program_headers {
//         if pheader.p_type == PT_LOAD {
//             if pheader.p_vaddr != PROCESS_START.as_u64() {
//                 return Err("ELF64 Format Error: Invalid p_vaddr".into());
//             }
//             let program_data = &data
//                 [pheader.p_offset as usize..pheader.p_offset as usize + pheader.p_filesz as usize];
//             return Ok(create_process(paging_service, program_data, PROCESS_START));
//         }
//     }

//     Err("ELF64 Format Error: Missing `PT_LOAD` section".into())
// }
