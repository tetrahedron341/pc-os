cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        #[no_mangle]
        pub extern "C" fn raw_syscall(syscall: Syscall) -> SyscallResult {
            let mut r = core::mem::MaybeUninit::uninit();
            unsafe {
                core::arch::asm!{
                    "syscall",
                    in("rdi") &syscall,
                    inout("rsi") r.as_mut_ptr() => _,
                    out("rcx") _,
                    out("r11") _,
                };
                r.assume_init()
            }
        }
    }
}

macro_rules! syscall_define {
    ($(
        $(#[doc = $docs:expr])*
        pub extern "C" fn $name:ident ($($(#[doc = $field_docs:expr])* $arg:ident : $t:ty),*) -> $return:ty;
    )*) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[allow(non_camel_case_types)]
        pub enum Syscall {$(
            $(#[doc = $docs])*
            $name {$(
                $(#[doc = $field_docs])*
                $arg: $t
            ),*}
        ),*}

        #[repr(C)]
        #[allow(non_camel_case_types)]
        pub union SyscallResultInner {$(
            pub $name: $return
        ),*}

        #[repr(C)]
        pub enum SyscallResult {
            Ok(SyscallResultInner),
            Err(SyscallErrorCode)
        }

        impl From<Result<SyscallResultInner, SyscallErrorCode>> for SyscallResult {
            fn from(r: Result<SyscallResultInner, SyscallErrorCode>) -> Self {
                match r {
                    Ok(v) => SyscallResult::Ok(v),
                    Err(e) => SyscallResult::Err(e)
                }
            }
        }
        impl From<SyscallResult> for Result<SyscallResultInner, SyscallErrorCode> {
            fn from(r: SyscallResult) -> Self {
                match r {
                    SyscallResult::Ok(v) => Ok(v),
                    SyscallResult::Err(e) => Err(e)
                }
            }
        }

        $(
            $(#[doc = $docs])*
            #[no_mangle]
            pub extern "C" fn $name($($(#[doc = $field_docs])* $arg: $t,)* out: Option<&mut core::mem::MaybeUninit<$return>>) -> SyscallErrorCode {
                match raw_syscall(Syscall::$name{$($arg),*}) {
                    SyscallResult::Ok(v) => unsafe {out.map(|out| out.write(v.$name)); SyscallErrorCode::Ok},
                    SyscallResult::Err(e) => e
                }
            }
        )*

    };
}

syscall_define! {
    /// Print out "Ping!" to the console screen
    pub extern "C" fn ping() -> ();
    pub extern "C" fn put_char(c: u8) -> ();
    pub extern "C" fn get_kbd_code() -> u8;
    pub extern "C" fn sleep_ms(duration_ms: u32) -> ();

    /// Exits the current process
    pub extern "C" fn exit(code: i8) -> ();
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyscallErrorCode {
    Ok = 0,
    InvalidArgumentError,
}
