pub static KERNEL_START: conquer_once::spin::OnceCell<usize> =
    conquer_once::spin::OnceCell::uninit();
pub static KERNEL_LEN: conquer_once::spin::OnceCell<usize> = conquer_once::spin::OnceCell::uninit();
pub static KERNEL_ELF: conquer_once::spin::OnceCell<
    &'static goblin::elf::header::header64::Header,
> = conquer_once::spin::OnceCell::uninit();

pub fn is_kernel_ip(ip: usize) -> bool {
    let kstart = *KERNEL_START.get().unwrap();
    let klen = *KERNEL_LEN.get().unwrap();
    (kstart..kstart + klen).contains(&ip)
}

pub unsafe fn unwind_by_rbp(rbp: *const u64) {
    unsafe fn inner(depth: usize, rbp: *const u64, syms: Option<&SymbolFinder>) {
        if rbp.is_null() {
            return;
        }
        let saved_rbp = *rbp;
        let return_addr = *(rbp.offset(1));
        let symbol = syms.and_then(|syms| syms.lookup_symbol(return_addr as usize));
        crate::serial_print!("    {depth}: {return_addr:#X}");
        if let Some((name, off)) = symbol {
            let demangled_name = rustc_demangle::demangle(name);
            crate::serial_println!(" ({demangled_name} + {off})");
        } else {
            crate::serial_println!("");
        }
        inner(depth + 1, saved_rbp as usize as *const _, syms);
    }

    crate::serial_println!("START OF BACKTRACE");
    unsafe {
        let syms = SymbolFinder::new();
        inner(0, rbp, syms.as_ref());
    }
    crate::serial_println!("END OF BACKTRACE");
}

struct SymbolFinder<'elf> {
    strtab: &'elf [u8],
    symtab: &'elf [goblin::elf64::sym::Sym],
    kernel_code_offset: usize,
}

impl<'elf> SymbolFinder<'elf> {
    unsafe fn new() -> Option<Self> {
        let kernel_load_vaddr = KERNEL_START.get()?;
        let kernel_code_offset = 0xFFFF_FFFF_8000_0000 - kernel_load_vaddr;
        let elf = *KERNEL_ELF.get()?;
        let elf_ptr = elf as *const _ as *const u8;
        let at_offset = |offset: usize| unsafe { elf_ptr.add(offset) };

        let shtab =
            at_offset(elf.e_shoff as usize).cast::<goblin::elf64::section_header::SectionHeader>();
        let shnum = elf.e_shnum;
        let shtab = unsafe { core::slice::from_raw_parts(shtab, shnum as usize) };

        let shstrtab = &shtab[elf.e_shstrndx as usize];
        let shstrtabdata = at_offset(shstrtab.sh_offset as usize);
        let shstrtablen = shstrtab.sh_size as usize;
        let shstrtab = unsafe { core::slice::from_raw_parts(shstrtabdata, shstrtablen) };
        let get_shstr = |offset: usize| {
            let mut len = 0;
            while shstrtab[offset + len] != b'\0' {
                len += 1;
            }
            return unsafe { core::str::from_utf8_unchecked(&shstrtab[offset..offset + len]) };
        };

        let strtab = shtab
            .iter()
            .find(|sh| get_shstr(sh.sh_name as usize) == ".strtab")?;
        let strtabdata = at_offset(strtab.sh_offset as usize);
        let strtablen = strtab.sh_size as usize;
        let strtab = unsafe { core::slice::from_raw_parts(strtabdata, strtablen) };
        let symtab = shtab
            .iter()
            .find(|&&sh| sh.sh_type == goblin::elf64::section_header::SHT_SYMTAB)?;
        let symtab = unsafe {
            core::slice::from_raw_parts(
                at_offset(symtab.sh_offset as usize).cast::<goblin::elf64::sym::Sym>(),
                symtab.sh_size as usize,
            )
        };

        Some(SymbolFinder {
            strtab,
            symtab,
            kernel_code_offset,
        })
    }

    fn get_str(&self, offset: usize) -> Option<&str> {
        let mut len = 0;
        while *self.strtab.get(offset + len)? != b'\0' {
            len += 1;
        }
        return core::str::from_utf8(&self.strtab[offset..offset + len]).ok();
    }

    fn lookup_symbol(&self, vaddr: usize) -> Option<(&str, usize)> {
        let vaddr = vaddr + self.kernel_code_offset;
        let (_sym_idx, sym) = self.symtab.iter().enumerate().find(|(_, sym)| {
            sym.is_function()
                && sym.st_value <= vaddr as u64
                && sym.st_value + sym.st_size > vaddr as u64
        })?;

        Some((
            self.get_str(sym.st_name as usize)?,
            vaddr - sym.st_value as usize,
        ))
    }
}
