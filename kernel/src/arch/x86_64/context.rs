use alloc::boxed::Box;

use super::interrupts::Registers;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Context {
    regs: Registers,
}

impl Context {
    pub fn new() -> Box<Self> {
        Box::new(Context {
            regs: unsafe { Registers::empty() },
        })
    }

    pub(super) fn save(&mut self, regs: Registers) {
        self.regs = regs;
    }

    /// # Safety
    /// Never call before a valid context has been saved.
    pub(super) unsafe fn load(&mut self, regs: &mut Registers) {
        *regs = self.regs.clone();
    }
}
