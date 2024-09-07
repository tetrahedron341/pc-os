use core::mem::MaybeUninit;

use alloc::{boxed::Box, string::String, vec, vec::Vec};
use limine::LimineFramebuffer;

use crate::{
    arch::{
        self, cpu,
        memory::{self, mmap::MemoryRegion, phys_to_virt, PhysAddr, VirtAddr},
    },
    boot::BootModule,
    video::Framebuffer,
};

static MMAP_REQUEST: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(0);

static HHDM_REQUEST: limine::LimineHhdmRequest = limine::LimineHhdmRequest::new(0);

static MODULES_REQUEST: limine::LimineModuleRequest = limine::LimineModuleRequest::new(0);

static FRAMEBUFFER_REQUEST: limine::LimineFramebufferRequest =
    limine::LimineFramebufferRequest::new(0);

static RSDP_REQUEST: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(0);

static KERNEL_ADDRESS_REQUEST: limine::LimineKernelAddressRequest =
    limine::LimineKernelAddressRequest::new(0);
static KERNEL_FILE_REQUEST: limine::LimineKernelFileRequest =
    limine::LimineKernelFileRequest::new(0);

// With the HHDM feature on, 4-level paging, and KASLR enabled, our higher half looks like:
//
// 0xffff8000_00000000..=0xffff8fff_ffffffff -- HHDM is somewhere in here
// 0xffffffff_80000000..=0xffffffff_ffffffff -- Kernel is somewhere in here

const SERIAL_LOG_MAX: log::LevelFilter = {
    #[cfg(debug_assertions)]
    {
        log::LevelFilter::Debug
    }
    #[cfg(not(debug_assertions))]
    {
        log::LevelFilter::Info
    }
};
const CONSOLE_LOG_MAX: log::LevelFilter = log::LevelFilter::Warn;

#[no_mangle]
fn _start() -> ! {
    x86_64::instructions::interrupts::disable();
    enable_simd();
    let kernel_start = crate::panic::unwind::KERNEL_START.get_or_init(|| {
        KERNEL_ADDRESS_REQUEST
            .get_response()
            .get()
            .unwrap()
            .virtual_base as usize
    });
    crate::serial_println!("KERNEL_START: {kernel_start:#X}");
    crate::serial_println!("SERIAL_LOG_MAX: {SERIAL_LOG_MAX:?}");
    crate::panic::unwind::KERNEL_LEN.init_once(|| {
        KERNEL_FILE_REQUEST
            .get_response()
            .get()
            .unwrap()
            .kernel_file
            .get()
            .unwrap()
            .length as usize
    });

    crate::panic::unwind::KERNEL_ELF.init_once(|| unsafe {
        let ptr = KERNEL_FILE_REQUEST
            .get_response()
            .get()
            .unwrap()
            .kernel_file
            .get()
            .unwrap()
            .base
            .as_ptr()
            .unwrap() as *const _;
        &*ptr
    });
    arch::x86_64::gdt::init();
    arch::x86_64::interrupts::init_idt();
    arch::x86_64::syscall::init();

    unsafe { arch::x86_64::memory::init(get_phys_mem_offset(), get_mmap()) };
    crate::allocator::init_heap().unwrap();

    crate::log::init(SERIAL_LOG_MAX, CONSOLE_LOG_MAX, 128);
    let modules = get_modules();

    let framebuffer = unsafe { get_framebuffer() }
        .map(|fb| Box::new(fb) as Box<dyn Framebuffer + Send + Sync + 'static>);
    cpu::init_this_cpu();
    enumerate_acpi_tables();
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
    let limine_mmap = mmap_response.memmap();

    const MMAP_BUFFER_LEN: usize = 256;
    static mut MMAP_BUFFER: [MaybeUninit<MemoryRegion>; MMAP_BUFFER_LEN] =
        MaybeUninit::uninit_array();
    assert!(limine_mmap.len() <= MMAP_BUFFER_LEN, "Memory map too long");
    let mmap = MaybeUninit::slice_assume_init_mut(&mut MMAP_BUFFER[..limine_mmap.len()]);
    for (i, r) in limine_mmap.iter().enumerate() {
        mmap[i] = MemoryRegion::from(&**r);
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
        .map(|resp| resp.modules())
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

        let data =
            unsafe { core::slice::from_raw_parts_mut(m.base.as_ptr().unwrap(), m.length as usize) };

        modules.push(crate::boot::BootModule { name, data });
    }
    modules
}

fn enumerate_acpi_tables() {
    #[derive(Clone, Copy)]
    struct AcpiHandler;

    impl acpi::AcpiHandler for AcpiHandler {
        unsafe fn map_physical_region<T>(
            &self,
            physical_address: usize,
            size: usize,
        ) -> acpi::PhysicalMapping<Self, T> {
            let vaddr = core::ptr::NonNull::new(
                phys_to_virt(PhysAddr::new(physical_address as u64)).as_mut_ptr(),
            )
            .unwrap();
            acpi::PhysicalMapping::new(physical_address, vaddr, size, size, *self)
        }
        fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
    }

    let rsdp_response = RSDP_REQUEST.get_response().get().unwrap();
    let phys_offset = get_phys_mem_offset().as_u64() as usize;
    let tables = unsafe {
        acpi::AcpiTables::from_rsdp(
            AcpiHandler,
            rsdp_response.address.as_ptr().unwrap() as usize - phys_offset,
        )
    }
    .unwrap();

    let pci = acpi::PciConfigRegions::new(&tables).unwrap();
    log::trace!("{pci:#X?}");
    enumerate_pci_devices(&pci);

    let fadt = unsafe {tables.get_sdt::<acpi::fadt::Fadt>(acpi::sdt::Signature::FADT)}.unwrap().unwrap();
    // check if acpi is already enabled
    let enabled = fadt.smi_cmd_port == 0 || (fadt.acpi_disable == 0 && fadt.acpi_enable == 0);
    if !enabled {
        log::debug!("Manually enabling ACPI");
        unsafe {x86_64::instructions::port::Port::new(fadt.smi_cmd_port as u16).write(fadt.acpi_enable);}
    } else {
        log::debug!("ACPI enabled by BIOS");
    }
}

fn enumerate_pci_devices(pci: &acpi::PciConfigRegions) {
    use crate::pci::{BaseHeader, HeaderType1};
    use acpi::PciConfigRegions as Pci;

    fn check_bus(pci: &Pci, bus: u8) {
        log::trace!("check_bus({bus})");
        for dev in 0..32 {
            check_dev(pci, bus, dev);
        }
    }
    fn check_dev(pci: &Pci, bus: u8, dev: u8) {
        log::trace!("check_dev({bus}, {dev})");
        let Some(cfg) = pci.physical_address(0, bus, dev, 0) else {return};
        let cfg_ptr = phys_to_virt(PhysAddr::new(cfg)).as_ptr::<BaseHeader>();
        let cfg = unsafe { cfg_ptr.as_ref().unwrap() };
        if cfg.vid == 0xffff {
            return;
        }
        log::debug!("BUS {bus} DEV {dev}");
        check_fn(pci, cfg);
        if cfg.header_type & 0x80 != 0 {
            // bit 7 of the header type indicates multi-function device
            for function in 1..8 {
                if let Some(fncfg) = pci.physical_address(0, bus, dev, function) {
                    let fncfg = unsafe {
                        phys_to_virt(PhysAddr::new(fncfg))
                            .as_ptr::<BaseHeader>()
                            .as_ref()
                            .unwrap()
                    };
                    if fncfg.vid == 0xffff {
                        return;
                    }
                    log::debug!("BUS {bus} DEV {dev} FN {function}");
                    check_fn(pci, fncfg);
                }
            }
        }
    }
    fn check_fn(pci: &Pci, cfg: &BaseHeader) {
        let class = cfg.class;
        let subclass = cfg.subclass;

        let kind = crate::pci::device_kind(class, subclass).unwrap_or("Unknown");
        log::debug!("{kind}");
        match cfg.header_type & 0x7f {
            0 => {
                let extcfg = unsafe { core::mem::transmute::<_, &crate::pci::HeaderType0>(cfg) };
                log::debug!("{extcfg:#X?}");
            }
            1 => {
                let extcfg = unsafe { core::mem::transmute::<_, &crate::pci::HeaderType1>(cfg) };
                log::debug!("{extcfg:#X?}");
            }
            _ => log::debug!("{cfg:#X?}"),
        }

        if class == 0x6 && subclass == 0x4 {
            // Pci-to-Pci bridge
            let cfg = unsafe { core::mem::transmute::<_, &HeaderType1>(cfg) };
            let bus = cfg.sec_bus;
            check_bus(pci, bus);
        }
    }

    let Some(root_cfg) = pci.physical_address(0, 0, 0, 0) else {return};
    let root_cfg = unsafe {
        phys_to_virt(PhysAddr::new(root_cfg))
            .as_ptr::<BaseHeader>()
            .as_ref()
            .unwrap()
    };
    if root_cfg.header_type & 0x80 == 0 {
        log::trace!("single PCI controller");
        check_bus(pci, 0);
    } else {
        log::trace!("multiple PCI controllers");
        for function in 0..8 {
            if let Some(fncfg) = pci.physical_address(0, 0, 0, function) {
                let fncfg = unsafe {
                    phys_to_virt(PhysAddr::new(fncfg))
                        .as_ptr::<BaseHeader>()
                        .as_ref()
                        .unwrap()
                };
                if fncfg.vid == 0xffff {
                    return;
                }
                check_bus(pci, function);
            }
        }
    }
}

/// # Safety
/// Call only once.
unsafe fn get_framebuffer() -> Option<impl Framebuffer + Send + Sync + 'static> {
    FRAMEBUFFER_REQUEST.get_response().get().map(|resp| {
        let fbs = resp.framebuffers();
        let fb = &(*fbs[0]) as *const LimineFramebuffer;
        Box::new(FramebufferImpl(
            &mut *(fb as *mut limine::LimineFramebuffer),
        )) as Box<dyn Framebuffer + Send + Sync + 'static>
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
                self.0.address.as_ptr().unwrap(),
                (self.0.height * self.0.pitch) as _,
            )
        }
    }
}
