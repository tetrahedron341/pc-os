use core::ptr::NonNull;

use x86_64::registers::model_specific::Msr;

use crate::arch::memory::{phys_to_virt, PhysAddr};

const APIC_BASE_ADDRESS_REGISTER: Msr = Msr::new(0x0000_001B);

#[repr(C, align(16))]
pub struct ApicRegister([u32; 4]);

impl ApicRegister {
    pub fn get(&self) -> &u32 {
        &self.0[0]
    }

    pub fn get_mut(&mut self) -> &mut u32 {
        &mut self.0[0]
    }
}

#[repr(C)]
pub struct ApicRegisters {
    _reserved0: [ApicRegister; 2],                   // 000h ..= 010h
    pub apic_id: ApicRegister,                       // 020h
    pub apic_version: ApicRegister,                  // 030h
    _reserved1: [ApicRegister; 4],                   // 040h ..= 070h
    pub tpr: ApicRegister,                           // 080h
    pub apr: ApicRegister,                           // 090h
    pub ppr: ApicRegister,                           // 0A0h
    pub eoi: ApicRegister,                           // 0B0h
    pub remote_read: ApicRegister,                   // 0C0h
    pub ldr: ApicRegister,                           // 0D0h
    pub dfr: ApicRegister,                           // 0E0h
    pub sivr: ApicRegister,                          // 0F0h
    pub isr: [ApicRegister; 8],                      // 100h ..= 170h
    pub tmr: [ApicRegister; 8],                      // 180h ..= 1F0h
    pub irr: [ApicRegister; 8],                      // 200h ..= 270h
    pub esr: ApicRegister,                           // 280h
    _reserved2: ApicRegister,                        // 290h
    pub icr: [ApicRegister; 2],                      // 300h ..= 310h
    pub timer_local_vte: ApicRegister,               // 320h
    pub thermal_local_vte: ApicRegister,             // 330h
    pub performance_counter_local_vte: ApicRegister, // 340h
    pub local_int0_vte: ApicRegister,                // 350h
    pub local_int1_vte: ApicRegister,                // 360h
    pub error_vte: ApicRegister,                     // 370h
    pub timer_initial_count: ApicRegister,           // 380h
    pub timer_current_count: ApicRegister,           // 390h
    _reserved3: [ApicRegister; 4],                   // 3A0h ..= 3D0h
    pub timer_divide_config: ApicRegister,           // 3E0h
    _reserved4: ApicRegister,                        // 3F0h
    pub extended_apic_features: ApicRegister,        // 400h
    pub extended_apic_control: ApicRegister,         // 410h
    pub specific_eoi: ApicRegister,                  // 420h
    _reserved5: [ApicRegister; 5],                   // 430h ..= 470h
    pub ier: [ApicRegister; 8],                      // 480h ..= 4F0h
    pub extended_interrupt_local_vector_tables: [ApicRegister; 4], // 500h ..= 530h
}

impl ApicRegisters {
    /// Allows use/modification of the APIC registers.
    /// # Safety
    /// Interrupts must be disabled while using the registers.
    pub unsafe fn get() -> NonNull<ApicRegisters> {
        let apic_base_register = APIC_BASE_ADDRESS_REGISTER.read();
        let apic_addr = PhysAddr::new(apic_base_register & 0x000F_FFFF_FFFF_F000);
        let apic: *mut ApicRegisters = phys_to_virt(apic_addr).as_mut_ptr();
        NonNull::new(apic).unwrap()
    }
}
