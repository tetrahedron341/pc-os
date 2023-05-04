//! Entry point for Philipp Oppermann's bootloader crate

use core::mem::MaybeUninit;

use alloc::boxed::Box;
use bootloader::BootInfo;

use crate::arch::memory::mmap::MemoryRegion;
use crate::arch::{self, memory};
use crate::boot::BootModule;
use crate::video::framebuffer::PixelFormat;
use crate::video::Framebuffer;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        bootloader::entry_point!(kernel_entry);

        fn kernel_entry(boot_info: &'static mut BootInfo) -> ! {
            crate::init::kernel_main(initialize(boot_info));
        }
    } else {
        bootloader::entry_point!(test_entry_point);

        /// Entry point for `cargo test`
        fn test_entry_point(boot_info: &'static mut bootloader::BootInfo) -> ! {
            initialize(boot_info);
            crate::test_main();
            crate::arch::loop_forever();
        }
    }
}

fn initialize(boot_info: &'static mut bootloader::BootInfo) -> crate::init::InitServices {
    arch::x86_64::interrupts::init_idt();
    arch::x86_64::gdt::init();

    let mmap: &'static [MemoryRegion] = {
        static mut MMAP_BUFFER: [MaybeUninit<MemoryRegion>; 256] = MaybeUninit::uninit_array();
        let mmap = unsafe {
            MaybeUninit::slice_assume_init_mut(&mut MMAP_BUFFER[..boot_info.memory_regions.len()])
        };
        for (i, &r) in boot_info.memory_regions.iter().enumerate() {
            mmap[i] = r.into();
        }
        mmap
    };

    unsafe { arch::x86_64::memory::init(boot_info.recursive_index.into_option().unwrap(), mmap) };
    crate::allocator::init_heap().unwrap();
    arch::x86_64::syscall::init();

    let modules = boot_info
        .modules
        .iter()
        .map(|m| unsafe { load_module(*m) })
        .collect();

    x86_64::instructions::interrupts::enable();

    const SERIAL_LOG_MIN: log::LevelFilter = log::LevelFilter::Info;
    const CONSOLE_LOG_MIN: log::LevelFilter = log::LevelFilter::Warn;

    crate::log::init(SERIAL_LOG_MIN, CONSOLE_LOG_MIN, 128);

    crate::serial_println!("[phil_opp_bootloader] Allocating Box<FB>");

    let framebuffer: Option<Box<dyn Framebuffer + Send + Sync>> = boot_info
        .framebuffer
        .as_mut()
        .map(|fb: &'static mut _| Box::new(fb) as Box<dyn Framebuffer + Send + Sync>);

    crate::serial_println!("[phil_opp_bootloader] Handing over to the kernel");

    crate::init::InitServices {
        modules,
        framebuffer,
    }
}

unsafe fn load_module(module_desc: bootloader::boot_info::Module) -> BootModule {
    let ptr = arch::x86_64::memory::phys_to_virt(x86_64::PhysAddr::new(module_desc.phys_addr))
        .as_mut_ptr();
    BootModule {
        name: core::str::from_utf8(&module_desc.name)
            .unwrap()
            .trim_end_matches('\0')
            .into(),
        data: core::slice::from_raw_parts_mut(ptr, module_desc.len),
    }
}

impl From<bootloader::boot_info::MemoryRegionKind> for memory::mmap::MemoryKind {
    fn from(k: bootloader::boot_info::MemoryRegionKind) -> Self {
        use bootloader::boot_info::MemoryRegionKind::*;
        match k {
            Usable => memory::mmap::MemoryKind::Available,
            Bootloader => memory::mmap::MemoryKind::Reserved,
            UnknownBios(_) => memory::mmap::MemoryKind::Other,
            UnknownUefi(_) => memory::mmap::MemoryKind::Other,
            _ => memory::mmap::MemoryKind::Other,
        }
    }
}

impl From<bootloader::boot_info::MemoryRegion> for memory::mmap::MemoryRegion {
    fn from(r: bootloader::boot_info::MemoryRegion) -> Self {
        memory::mmap::MemoryRegion {
            start: r.start as usize,
            len: (r.end - r.start) as usize,
            kind: r.kind.into(),
        }
    }
}

impl crate::video::Framebuffer for bootloader::boot_info::FrameBuffer {
    fn info(&self) -> crate::video::framebuffer::FramebufferInfo {
        let info = bootloader::boot_info::FrameBuffer::info(self);
        crate::video::framebuffer::FramebufferInfo {
            buffer_len: info.byte_len,
            bytes_per_pixel: info.bytes_per_pixel,
            width: info.horizontal_resolution as u32,
            height: info.vertical_resolution as u32,
            stride: info.stride * info.bytes_per_pixel,
            format: info.pixel_format.into(),
        }
    }

    fn get_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl From<bootloader::boot_info::PixelFormat> for PixelFormat {
    fn from(fmt: bootloader::boot_info::PixelFormat) -> Self {
        use bootloader::boot_info::PixelFormat::*;
        match fmt {
            RGB => PixelFormat::RGBA,
            BGR => PixelFormat {
                red_shift_bits: 16,
                red_width_bits: 8,
                green_shift_bits: 8,
                green_width_bits: 8,
                blue_shift_bits: 0,
                blue_width_bits: 8,
            },
            U8 => PixelFormat {
                red_shift_bits: 0,
                red_width_bits: 0,
                green_shift_bits: 0,
                green_width_bits: 8,
                blue_shift_bits: 0,
                blue_width_bits: 0,
            },
            _ => PixelFormat::RGBA,
        }
    }
}
