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
