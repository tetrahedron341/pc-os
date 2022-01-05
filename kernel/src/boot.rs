pub struct BootModule {
    pub name: alloc::string::String,
    pub data: &'static mut [u8],
}

impl core::fmt::Debug for BootModule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootModule")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl BootModule {
    unsafe fn load(module_desc: bootloader::boot_info::Module) -> Self {
        let ptr =
            crate::memory::phys_to_virt(x86_64::PhysAddr::new(module_desc.phys_addr)).as_mut_ptr();
        BootModule {
            name: core::str::from_utf8(&module_desc.name)
                .unwrap()
                .trim_end_matches('\0')
                .into(),
            data: core::slice::from_raw_parts_mut(ptr, module_desc.len),
        }
    }
}
