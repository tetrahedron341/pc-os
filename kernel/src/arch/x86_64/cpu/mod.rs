mod apic;

use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard};

use super::interrupts::InterruptGuard;

/// Contains all of the processor-specific structures.
pub struct Cpu {}

const MAX_CORES: usize = 16;
const NONE_CPU: OnceCell<Mutex<Cpu>> = OnceCell::uninit(); // workaround because Cpu is non-Copy
static mut CPUS: [OnceCell<Mutex<Cpu>>; MAX_CORES] = [NONE_CPU; MAX_CORES];

fn current_cpu_id() -> usize {
    unsafe { apic::ApicRegisters::with(|regs| (*regs.apic_id.get() as usize) >> 24) }
}

/// Get access to the [`Cpu`] representing the currently running core.
/// # Panics
/// Panics if the current CPU has not yet been initialized with [`init_this_cpu`]
pub fn this_cpu(int_guard: &InterruptGuard) -> MutexGuard<'_, Cpu> {
    raw_this_cpu(int_guard).as_ref().unwrap().lock()
}

/// Initializes the current CPU core.
pub fn init_this_cpu(int_guard: &InterruptGuard) {
    let cpu = unsafe { &mut CPUS[current_cpu_id()] };
    cpu.init_once(|| Mutex::new(Cpu {}))
}

fn raw_this_cpu(_int_guard: &InterruptGuard) -> Option<&Mutex<Cpu>> {
    unsafe { CPUS[current_cpu_id()].get() }
}
