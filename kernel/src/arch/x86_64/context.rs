use super::interrupts::Registers;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Context {}

impl Context {
    pub(super) fn save(&mut self, regs: Registers) {}
}
