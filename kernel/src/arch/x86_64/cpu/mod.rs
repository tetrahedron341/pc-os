pub mod apic;
mod context;
mod registers;

use core::ptr::NonNull;

pub use registers::Registers;

use crate::process::{Process, ProcessState};

pub use self::context::Context;

const MAX_CORES: usize = 8;
const NONE_CPU: Option<Cpu> = None; // workaround because Cpu is non-Copy
static mut CPUS: [Option<Cpu>; MAX_CORES] = [NONE_CPU; MAX_CORES];

/// Contains all of the processor-specific structures.
/// Each CPU core must only ever be able to access its own `Cpu` struct.
pub struct Cpu {
    id: usize,
    /// The process currently being run, if we are currently in a process.
    process: Option<NonNull<Process>>,
    scheduler_ctx: *mut Context,
}

impl Cpu {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn run_process(&mut self, proc: &mut Process, waker: core::task::Waker) {
        proc.space.load();
        let load = proc.context;
        proc.state = ProcessState::Running(waker);
        self.process = Some(NonNull::from(proc));
        unsafe { Context::switch(&mut self.scheduler_ctx, load) }
        self.scheduler_ctx = core::ptr::null_mut();
    }

    /// Returns an error if the CPU is already holding a process.
    pub fn put_process<'a>(&mut self, proc: &'a mut Process) -> Result<(), &'a mut Process> {
        if self.process.is_some() {
            return Err(proc);
        }
        self.process = Some(NonNull::from(proc));
        Ok(())
    }

    pub fn try_take_process(&mut self) -> Option<&'static mut Process> {
        self.process.take().map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn return_from_process(&mut self, proc: &mut Process) {
        unsafe { Context::switch(&mut proc.context, self.scheduler_ctx) }
    }
}

fn current_cpu_id() -> usize {
    let regs = unsafe { apic::ApicRegisters::get().as_mut() };
    (*regs.apic_id.get() as usize) >> 24
}

/// Get access to the [`Cpu`] representing the currently running core.
/// # Panics
/// Panics if the current CPU has not yet been initialized with [`init_this_cpu`], or if interrupts
/// have not been disabled.
pub fn this_cpu() -> &'static mut Cpu {
    if x86_64::instructions::interrupts::are_enabled() {
        panic!("this_cpu called with interrupts enabled")
    }
    unsafe {
        CPUS[current_cpu_id()]
            .as_mut()
            .expect("CPU has not been initialized")
    }
}

pub fn init_this_cpu() {
    if x86_64::instructions::interrupts::are_enabled() {
        panic!("init_this_cpu called with interrupts enabled")
    }

    let id = current_cpu_id();

    let cpu = Cpu {
        id,
        process: None,
        scheduler_ctx: core::ptr::null_mut(),
    };

    unsafe {
        CPUS[id] = Some(cpu);
    }
}
