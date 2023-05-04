use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::boot::BootModule;
use crate::video::Framebuffer;

pub struct InitServices {
    pub modules: Vec<BootModule>,
    pub framebuffer: Option<Box<dyn Framebuffer + Send + Sync>>,
}

extern "Rust" {
    fn __kernel_main_impl(init_services: InitServices) -> !;
}

#[cfg_attr(test, allow(unused_variables))]
pub fn kernel_main(init_services: InitServices) -> ! {
    unsafe { __kernel_main_impl(init_services) };
}

#[macro_export]
macro_rules! kernel_main {
    ($f:expr) => {
        #[no_mangle]
        fn __kernel_main_impl(init_services: $crate::init::InitServices) -> ! {
            $f(init_services)
        }
    };
}
