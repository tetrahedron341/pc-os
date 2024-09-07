use super::cpu::{this_cpu, Registers};
use pic8259::ChainedPics;
use x86_64::structures::idt::*;

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

const TIMER_VEC: u8 = PIC_1_OFFSET;
const KEYBOARD_VEC: u8 = PIC_1_OFFSET + 1;
// const MOUSE_VEC: u8 = PIC_2_OFFSET + 4;

#[derive(Debug)]
#[repr(C)]
struct IsfWithRegisters {
    pub registers: Registers,
    pub error_code: u64,
    pub isf: InterruptStackFrameValue,
}

macro_rules! save_regs {
    ($handler:ident) => {{
        let _ = $handler as extern "C" fn(*mut IsfWithRegisters) -> *mut IsfWithRegisters;
        #[naked]
        extern "x86-interrupt" fn wrapper(_: InterruptStackFrame) {
            use core::arch::asm;
            unsafe {asm!(
                "cld",
                "push 0",
                "push rax",
                "push rbx",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push rbp",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "push r12",
                "push r13",
                "push r14",
                "push r15",
                "mov rdi, rsp",
                "call {handler}",
                "mov rsp, rax",
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rbp",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rbx",
                "pop rax",
                "add rsp, 8",
                "iretq",
                handler = sym $handler,
                options(noreturn)
            );}
        }

        wrapper
    }};
}

extern "C" fn breakpoint_handler(regs: *mut IsfWithRegisters) -> *mut IsfWithRegisters {
    crate::serial_println!("EXCEPTION: BREAKPOINT\n{:X?}", unsafe { &*regs });
    regs
}

// extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
//     crate::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
// }

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    use crate::{arch::loop_forever, print, println};
    use x86_64::registers::control::Cr2;
    use x86_64::structures::idt::PageFaultErrorCode;
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed address: {:?}", Cr2::read());
    print!("Error code: ");
    if !error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
        print!("NOT_PRESENT | ");
    }
    println!("{:?}", error_code);
    println!("{:#?}", stack_frame);
    loop_forever();
}

extern "x86-interrupt" fn gp_fault_handler(isf: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\nError code: {error_code}\n{isf:#?}");
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::task::timer::tick_timer();

    x86_64::instructions::interrupts::without_interrupts(|| {
        let cpu = this_cpu();
        if let Some(proc) = cpu.try_take_process() {
            cpu.return_from_process(proc)
        }
    });

    unsafe {
        PICS.lock().notify_end_of_interrupt(TIMER_VEC);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(KEYBOARD_VEC);
    }
}

static IDT: conquer_once::spin::OnceCell<InterruptDescriptorTable> =
    conquer_once::spin::OnceCell::uninit();

pub fn init_idt() {
    IDT.try_init_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint
            .set_handler_fn(save_regs!(breakpoint_handler));
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(gp_fault_handler);
        idt[TIMER_VEC as usize].set_handler_fn(timer_interrupt_handler);
        idt[KEYBOARD_VEC as usize].set_handler_fn(keyboard_interrupt_handler);
        idt
    })
    .expect("Tried to initialize IDT twice");
    IDT.get().unwrap().load();

    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        pics.write_masks(0xFE, 0xFF);
    };
}

#[test_case]
fn test_breakpoint() {
    x86_64::instructions::interrupts::int3();
}
