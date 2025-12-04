#![no_std]
//#![allow(internal_features)]
//#![feature(core_intrinsics)]

pub mod mem;
pub use paste::paste;

/*
    Macro for importing functions using crystal palace conventions
*/
#[macro_export]
macro_rules! import {
    // MODULE$Func form
    ($vis:vis $lib:ident!$func:ident ($($arg:ident :$arg_ty:ty ),* $(,)?) -> $ret:ty) => {
        $crate::import!(
            @impl $vis,
            $func,
            concat!("\x01__imp_", stringify!($lib), "$", stringify!($func)),
            ( $( $arg : $arg_ty ),* ),
            $ret
        );
    };

    // Bare form
    ($vis:vis $func:ident ($($arg:ident :$arg_ty:ty ),* $(,)?) -> $ret:ty) => {
        $crate::import!(
            @impl $vis,
            $func,
            concat!("\x01__imp_", stringify!($func)),
            ( $( $arg : $arg_ty ),* ),
            $ret
        );
    };

    // Internal implementation
    (@impl $vis:vis, $func:ident, $link_name:expr, ( $( $arg:ident : $arg_ty:ty ),* ), $ret:ty) => {
        $crate::paste! {
            // import slot symbol
            unsafe extern {
                #[link_name = $link_name]
                static [<__ $func>]: u8;     // the type doesnt matter its just to have the symbol referenced
            }
            
            // function definition for structs and pointers
            #[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
            #[cfg(target_arch = "x86_64")] 
            $vis type [<$func Fn>] = unsafe extern "C" fn( $( $arg_ty ),* ) -> $ret;

            #[inline(always)]
            #[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
            unsafe fn [<$func _ptr>]() -> [<$func Fn>] {
                let f: [<$func Fn>];
                core::arch::asm!(
                    "mov rax, [rip + {slot}]",
                    slot = sym [<__ $func>],
                    lateout("rax") f,
                    options(nostack, preserves_flags),
                );
                f
            }

            // call function with compiler generated ABI stuff
            #[inline(always)]
            #[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
            $vis unsafe fn $func( $( $arg : $arg_ty ),* ) -> $ret {
                unsafe {
                    let f = [<$func _ptr>]();
                    f( $( $arg ),* )
                }
            }
        }
    };
}

#[macro_export]
macro_rules! append_data {
    // default section name based on symbol name since we are linking data to the symbol name anyway (rust)
    ($sym:ident, $fn_name:ident) => {
        $crate::append_data!(
            $sym,
            $fn_name,
            concat!(".rdata$", stringify!($sym))
        );
    };

    ($sym:ident, $fn_name:ident, $section:expr) => {
        // define symbol
        #[unsafe(no_mangle)]
        #[unsafe(link_section = $section)]
        static $sym: [u8; 0] = [];
    
        #[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
        unsafe fn $fn_name() -> *const u8 {
            let ptr: *const u8;
            core::arch::asm!(
                "lea {out}, [rip + {sym}]",
                sym = sym $sym,
                out = lateout(reg) ptr,
                options(readonly, nostack, preserves_flags),
            );
            ptr
        }
    };
}

#[macro_export]
macro_rules! patch {
    ($type:ty, $name:ident) => {

    };
}

/* Struct and helper function for retrieving appended resource using crystal palace convention */
#[repr(C)]
struct _RESOURCE {
    length: core::ffi::c_int,
    value: [u8; 0],
}

pub unsafe fn get_resource<'a>(ptr: *const u8) -> &'a [u8] {
    unsafe {
        let header = &*(ptr as *const _RESOURCE);
        core::slice::from_raw_parts(header.value.as_ptr(), header.length as _)
    }
}

/* for debugging */
#[inline(always)]
pub fn brk() { unsafe { core::arch::asm!("int3"); } }
