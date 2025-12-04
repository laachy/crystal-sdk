/*
    https://github.com/rust-lang/compiler-builtins/blob/master/compiler-builtins/src/mem/mod.rs
*/

#![allow(dead_code)]

mod x86_64;
use core::ffi::c_int;

use crate::mem::x86_64::impls;


#[inline(always)]
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe { impls::copy_forward(dest, src, n) };
    dest
}

#[inline(always)]
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let delta = (dest as usize).wrapping_sub(src as usize);
    if delta >= n {
        // We can copy forwards because either dest is far enough ahead of src,
        // or src is ahead of dest (and delta overflowed).
        unsafe { impls::copy_forward(dest, src, n) };
    } else {
        unsafe { impls::copy_backward(dest, src, n) };
    }
    dest
}

#[inline(always)]
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    unsafe { impls::set_bytes(s, c as u8, n) };
    s
}

/*#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe { impls::compare_bytes(s1, s2, n) }
}*/

/*#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe { memcmp(s1, s2, n) }
}*/

#[inline(always)]
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const core::ffi::c_char) -> usize {
    unsafe { impls::c_string_length(s) }
}