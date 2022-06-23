use alloc::boxed::Box;
use spin::Mutex;

pub use self::framebuffer::Framebuffer;

pub mod console;
pub mod font;
// pub mod vesa;
pub mod framebuffer;

// pub static FRAMEBUFFER: Mutex<Box<dyn Framebuffer + Send>> =
//     Mutex::new(framebuffer::NullFramebuffer);
