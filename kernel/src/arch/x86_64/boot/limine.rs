use core::mem::MaybeUninit;

use alloc::{boxed::Box, string::String, vec, vec::Vec};

use crate::{
    arch::{
        self, cpu,
        memory::{self, mmap::MemoryRegion, phys_to_virt, PhysAddr, VirtAddr},
    },
    boot::BootModule,
    video::Framebuffer,
};

static MMAP_REQUEST: limine::LimineMmapRequest = limine::LimineMmapRequest::new(0);

static HHDM_REQUEST: limine::LimineHhdmRequest = limine::LimineHhdmRequest::new(0);

static MODULES_REQUEST: limine::LimineModuleRequest = limine::LimineModuleRequest::new(0);

static FRAMEBUFFER_REQUEST: limine::LimineFramebufferRequest =
    limine::LimineFramebufferRequest::new(0);

static RSDP_REQUEST: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(0);

// With the HHDM feature on, 4-level paging, and KASLR enabled, our higher half looks like:
//
// 0xffff8000_00000000..=0xffff8fff_ffffffff -- HHDM is somewhere in here
// 0xffffffff_80000000..=0xffffffff_ffffffff -- Kernel is somewhere in here

const SERIAL_LOG_MIN: log::LevelFilter = log::LevelFilter::Info;
const CONSOLE_LOG_MIN: log::LevelFilter = log::LevelFilter::Warn;

#[no_mangle]
fn _start() -> ! {
    x86_64::instructions::interrupts::disable();
    enable_simd();
    arch::x86_64::gdt::init();
    arch::x86_64::interrupts::init_idt();
    arch::x86_64::syscall::init();

    unsafe { arch::x86_64::memory::init(get_phys_mem_offset(), get_mmap()) };
    crate::allocator::init_heap().unwrap();

    crate::log::init(SERIAL_LOG_MIN, CONSOLE_LOG_MIN, 128);
    let modules = get_modules();

    enumerate_acpi_tables();

    let framebuffer = unsafe { get_framebuffer() }
        .map(|fb| Box::new(fb) as Box<dyn Framebuffer + Send + Sync + 'static>);
    cpu::init_this_cpu();
    x86_64::instructions::interrupts::enable();

    crate::init::kernel_main(crate::init::InitServices {
        modules,
        framebuffer,
    });
}

fn enable_simd() {
    unsafe {
        x86_64::registers::control::Cr0::update(|r| {
            use x86_64::registers::control::Cr0Flags;
            r.remove(Cr0Flags::EMULATE_COPROCESSOR);
            r.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
        x86_64::registers::control::Cr4::update(|r| {
            use x86_64::registers::control::Cr4Flags;
            r.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE);
        });
    }
}

/// # Safety
/// Must be called only once
unsafe fn get_mmap() -> &'static mut [MemoryRegion] {
    let mmap_response = MMAP_REQUEST
        .get_response()
        .get()
        .expect("MMAP request failed");
    let limine_mmap = mmap_response.mmap().expect("MMAP request failed");

    const MMAP_BUFFER_LEN: usize = 256;
    static mut MMAP_BUFFER: [MaybeUninit<MemoryRegion>; MMAP_BUFFER_LEN] =
        MaybeUninit::uninit_array();
    assert!(limine_mmap.len() <= MMAP_BUFFER_LEN, "Memory map too long");
    let mmap = MaybeUninit::slice_assume_init_mut(&mut MMAP_BUFFER[..limine_mmap.len()]);
    for (i, r) in limine_mmap.iter().enumerate() {
        mmap[i] = r.into();
    }

    crate::serial_println!("{:X?}", mmap);

    mmap
}

fn get_phys_mem_offset() -> VirtAddr {
    let phys_mem_start = HHDM_REQUEST
        .get_response()
        .get()
        .expect("HHDM request failed")
        .offset;

    VirtAddr::new(phys_mem_start)
}

fn get_modules() -> Vec<BootModule> {
    let limine_modules = MODULES_REQUEST
        .get_response()
        .get()
        .and_then(|resp| resp.modules())
        .unwrap_or_else(|| {
            crate::serial_println!("Module request failed");
            &[]
        });
    let mut modules = vec![];
    for m in limine_modules {
        let name_ptr = m.path.as_ptr().unwrap().cast::<u8>();
        let mut name_len = 0;
        while unsafe { name_ptr.offset(name_len).read() } != 0 {
            name_len += 1;
        }
        let name = unsafe {
            String::from_utf8_lossy(core::slice::from_raw_parts(name_ptr, name_len as usize))
                .split('/')
                .last()
                .unwrap()
                .into()
        };

        let data = unsafe {
            core::slice::from_raw_parts_mut(m.base.as_mut_ptr().unwrap(), m.length as usize)
        };

        modules.push(crate::boot::BootModule { name, data });
    }
    modules
}

fn enumerate_acpi_tables() {
    #[repr(C, packed)]
    struct Rsdp {
        signature: [u8; 8],
        checksum: u8,
        oem_id: [u8; 6],
        revision: u8,
        rsdt_addr: u32,
    }

    #[repr(C, packed)]
    struct RsdpExtended {
        rsdp: Rsdp,
        len: u32,
        xsdt_addr: u64,
        checksum: u8,
        reserved: [u8; 3],
    }

    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy)]
    struct SdtHeader {
        signature: [u8; 4],
        len: u32,
        revision: u8,
        checksum: u8,
        oem_id: [u8; 6],
        oem_table_id: [u8; 8],
        oem_revision: u32,
        creator_id: u32,
        creator_revision: u32,
    }

    #[repr(C, packed)]
    #[derive(Debug)]
    struct Mcfg {
        header: SdtHeader,
        _reserved: [u8; 8],
        entries: [McfgEntry],
    }

    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy)]
    struct McfgEntry {
        base_addr: u64,
        pci_seg_group: u16,
        start_bus: u8,
        end_bus: u8,
        _reserved: [u8; 4],
    }

    let rsdp_response = RSDP_REQUEST.get_response().get().unwrap();
    let rsdp = rsdp_response.address.as_mut_ptr().unwrap().cast::<Rsdp>();

    unsafe {
        let sig = &(*rsdp).signature;
        assert_eq!(sig, b"RSD PTR ", "RSDP signature");
        crate::serial_println!(
            "RSDP signature: `{}`",
            core::str::from_utf8_unchecked(&(*rsdp).signature)
        );
        crate::serial_println!(
            "RSDP.OEMID: `{}`",
            core::str::from_utf8_unchecked(&(*rsdp).oem_id)
        );

        let rev = (*rsdp).revision;
        assert_eq!(rev, 2, "ACPI 2.0 required");
    }
    let rsdp = rsdp.cast::<RsdpExtended>();
    let xsdt_addr = unsafe { phys_to_virt(PhysAddr::new((*rsdp).xsdt_addr)) };
    let xsdt = xsdt_addr.as_mut_ptr::<SdtHeader>();
    unsafe {
        let xsdt_len = (*xsdt).len;
        let xsdt_entries = (xsdt_len - core::mem::size_of::<SdtHeader>() as u32) / 8;
        crate::serial_println!(
            "XSDT signature: `{}`; length: {}; entries: {}",
            core::str::from_utf8_unchecked(&(*xsdt).signature),
            xsdt_len,
            xsdt_entries
        );

        let table_addrs = {
            let p = xsdt
                .cast::<u8>()
                .add(core::mem::size_of::<SdtHeader>())
                .cast::<u64>();
            core::slice::from_raw_parts(p, xsdt_entries as usize)
        };

        let mut mcfg = None;

        crate::serial_print!("ACPI table signatures: ");
        for e in table_addrs {
            let vaddr = phys_to_virt(PhysAddr::new(*e));
            let table = vaddr.as_mut_ptr::<SdtHeader>().as_ref().unwrap();
            crate::serial_print!("`{}`, ", core::str::from_utf8_unchecked(&table.signature));
            if &table.signature == b"MCFG" {
                let mcfg_entries =
                    (table.len - core::mem::size_of::<SdtHeader>() as u32 - 8) as usize / 16;
                let p = vaddr.as_mut_ptr::<()>();
                let mcfg_ptr = core::ptr::from_raw_parts::<Mcfg>(p, mcfg_entries);
                mcfg.replace(mcfg_ptr);
            }
        }
        crate::serial_println!("");

        if let Some(mcfg) = mcfg {
            let mcfg = &*mcfg;
            crate::serial_println!("{mcfg:?}");

            for e in &mcfg.entries {
                let base_addr = e.base_addr;
                for bus in e.start_bus..=e.end_bus {
                    let bus_off = ((bus - e.start_bus) as u64) << 20;
                    for device in 0..32 {
                        let dev_off = (device as u64) << 15;
                        for func in 0..8 {
                            let fun_off = (func as u64) << 12;
                            let addr = PhysAddr::new(base_addr | bus_off | dev_off | fun_off);
                            let p = phys_to_virt(addr).as_mut_ptr::<[u16; 6]>();
                            let [vid, did, ..] = *p;
                            if vid == 0xFFFF {
                                continue;
                            }
                            crate::serial_println!("PCI @ BUS {bus} - DEV {device} - FUN {func}: VID {vid:04X} - DID {did:04X}");
                            let [subclass, class] = u16::to_le_bytes((*p)[5]);
                            crate::serial_println!(
                                "    Class: {class:02X}, Subclass: {subclass:02X}"
                            );
                        }
                    }
                }
            }
        }
    }
}

/// # Safety
/// Call only once.
unsafe fn get_framebuffer() -> Option<impl Framebuffer + Send + Sync + 'static> {
    FRAMEBUFFER_REQUEST.get_response().get().and_then(|resp| {
        resp.framebuffers().map(|fbs| {
            let fb = &fbs[0] as *const _;
            Box::new(FramebufferImpl(
                &mut *(fb as *mut limine::LimineFramebuffer),
            )) as Box<dyn Framebuffer + Send + Sync + 'static>
        })
    })
}

impl From<limine::LimineMemoryMapEntryType> for memory::mmap::MemoryKind {
    fn from(k: limine::LimineMemoryMapEntryType) -> Self {
        use limine::LimineMemoryMapEntryType::*;
        match k {
            Usable => memory::mmap::MemoryKind::Available,
            BootloaderReclaimable => memory::mmap::MemoryKind::Reserved,
            Reserved => memory::mmap::MemoryKind::Reserved,
            KernelAndModules => memory::mmap::MemoryKind::Reserved,
            _ => memory::mmap::MemoryKind::Other,
        }
    }
}

impl From<&limine::LimineMemmapEntry> for memory::mmap::MemoryRegion {
    fn from(e: &limine::LimineMemmapEntry) -> Self {
        memory::mmap::MemoryRegion {
            start: e.base as usize,
            len: e.len as usize,
            kind: e.typ.into(),
        }
    }
}

struct FramebufferImpl(&'static mut limine::LimineFramebuffer);

unsafe impl Send for FramebufferImpl {}
unsafe impl Sync for FramebufferImpl {}

impl crate::video::Framebuffer for FramebufferImpl {
    fn info(&self) -> crate::video::framebuffer::FramebufferInfo {
        crate::video::framebuffer::FramebufferInfo {
            format: crate::video::framebuffer::PixelFormat {
                red_shift_bits: self.0.red_mask_shift,
                red_width_bits: self.0.red_mask_size,
                green_shift_bits: self.0.green_mask_shift,
                green_width_bits: self.0.green_mask_size,
                blue_shift_bits: self.0.blue_mask_shift,
                blue_width_bits: self.0.blue_mask_size,
            },
            bytes_per_pixel: (self.0.bpp as usize) / 8,
            width: self.0.width as _,
            height: self.0.height as _,
            stride: self.0.pitch as _,
            buffer_len: (self.0.height * self.0.pitch) as _,
        }
    }

    fn get_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.0.address.as_mut_ptr().unwrap(),
                (self.0.height * self.0.pitch) as _,
            )
        }
    }
}
