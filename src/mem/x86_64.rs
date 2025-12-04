/*
    https://github.com/rust-lang/compiler-builtins/blob/master/compiler-builtins/src/mem/x86_64.rs
*/

#![cfg(target_arch = "x86_64")]

pub mod impls {
    use core::arch::{self, asm};
    //use core::{intrinsics, mem};

    #[inline(always)]
    pub unsafe fn copy_forward(dest: *mut u8, src: *const u8, count: usize) {
        unsafe {
            core::arch::asm!(
                "repe movsb (%rsi), (%rdi)",
                inout("rcx") count => _,
                inout("rdi") dest => _,
                inout("rsi") src => _,
                options(att_syntax, nostack, preserves_flags)
            );
        }
    }

    #[inline(always)]
    pub unsafe fn copy_backward(dest: *mut u8, src: *const u8, count: usize) {
        let (pre_byte_count, qword_count, byte_count) = rep_param(dest, count);
        // We can't separate this block due to std/cld
        unsafe {
            asm!(
                "std",
                "rep movsb",
                "sub $7, %rsi",
                "sub $7, %rdi",
                "mov {qword_count}, %rcx",
                "rep movsq",
                "test {pre_byte_count:e}, {pre_byte_count:e}",
                "add $7, %rsi",
                "add $7, %rdi",
                "mov {pre_byte_count:e}, %ecx",
                "rep movsb",
                "cld",
                pre_byte_count = in(reg) pre_byte_count,
                qword_count = in(reg) qword_count,
                inout("ecx") byte_count => _,
                inout("rdi") dest.add(count - 1) => _,
                inout("rsi") src.add(count - 1) => _,
                // We modify flags, but we restore it afterwards
                options(att_syntax, nostack, preserves_flags)
            );
        }
    }

    #[inline(always)]
    pub unsafe fn set_bytes(dest: *mut u8, c: u8, count: usize) {
        // FIXME: Use the Intel syntax once we drop LLVM 9 support on rust-lang/rust.
        unsafe {
            core::arch::asm!(
                "repe stosb %al, (%rdi)",
                inout("rcx") count => _,
                inout("rdi") dest => _,
                inout("al") c => _,
                options(att_syntax, nostack, preserves_flags)
            )
        }
    }

    /*#[inline(always)]
    pub unsafe fn compare_bytes(a: *const u8, b: *const u8, n: usize) -> i32 {
        #[inline(always)]
        unsafe fn cmp<T, U, F>(mut a: *const T, mut b: *const T, n: usize, f: F) -> i32
        where
            T: Clone + Copy + Eq,
            U: Clone + Copy + Eq,
            F: FnOnce(*const U, *const U, usize) -> i32,
        {
            // Ensure T is not a ZST.
            const { assert!(mem::size_of::<T>() != 0) };

            let end = unsafe { a.add(intrinsics::unchecked_div(n, mem::size_of::<T>())) };
            while a != end {
                if unsafe { a.read_unaligned() != b.read_unaligned() }{
                    return f(a.cast(), b.cast(), mem::size_of::<T>());
                }
                a = unsafe { a.add(1) };
                b = unsafe { b.add(1) };
            }
            f(
                a.cast(),
                b.cast(),
                unsafe { intrinsics::unchecked_rem(n, mem::size_of::<T>()) },
            )
        }
        let c1 = |mut a: *const u8, mut b: *const u8, n| {
            for _ in 0..n {
                if unsafe { a.read() != b.read() }{
                    return unsafe { i32::from(a.read()) - i32::from(b.read())};
                }
                a = unsafe { a.add(1) };
                b = unsafe { b.add(1) };
            }
            0
        };
        let c2 = |a: *const u16, b, n| unsafe { cmp(a, b, n, c1) };
        let c4 = |a: *const u32, b, n| unsafe { cmp(a, b, n, c2) };
        let c8 = |a: *const u64, b, n| unsafe { cmp(a, b, n, c4) };
        let c16 = |a: *const u128, b, n| unsafe { cmp(a, b, n, c8) };
        c16(a.cast(), b.cast(), n)
    }*/
    
    fn rep_param(dest: *mut u8, mut count: usize) -> (usize, usize, usize) {
        // Unaligned writes are still slow on modern processors, so align the destination address.
        let pre_byte_count = ((8 - (dest as usize & 0b111)) & 0b111).min(count);
        count -= pre_byte_count;
        let qword_count = count >> 3;
        let byte_count = count & 0b111;
        (pre_byte_count, qword_count, byte_count)
    }

    #[inline(always)]
    pub unsafe fn c_string_length(mut s: *const core::ffi::c_char) -> usize {
        use core::arch::x86_64::{__m128i, _mm_cmpeq_epi8, _mm_movemask_epi8, _mm_set1_epi8};

        let mut n = 0;

        // The use of _mm_movemask_epi8 and company allow for speedups,
        // but they aren't cheap by themselves. Thus, possibly small strings
        // are handled in simple loops.

        for _ in 0..4 {
            if unsafe { *s } == 0 {
                return n;
            }

            n += 1;
            s = unsafe { s.add(1) };
        }

        // Shave of the least significand bits to align the address to a 16
        // byte boundary. The shaved of bits are used to correct the first iteration.

        let align = s as usize & 15;
        let mut s = ((s as usize) - align) as *const __m128i;
        let zero = unsafe { _mm_set1_epi8(0) };

        let x = {
            let r;
            unsafe {
                arch::asm!(
                    "movdqa ({addr}), {dest}",
                    addr = in(reg) s,
                    dest = out(xmm_reg) r,
                    options(att_syntax, nostack),
                );
            }
            r
        };
        let cmp = unsafe { _mm_movemask_epi8(_mm_cmpeq_epi8(x, zero)) } >> align;

        if cmp != 0 {
            return n + cmp.trailing_zeros() as usize;
        }

        n += 16 - align;
        s = unsafe { s.add(1) };

        loop {
            let x = {
                let r;
                unsafe {
                    arch::asm!(
                        "movdqa ({addr}), {dest}",
                        addr = in(reg) s,
                        dest = out(xmm_reg) r,
                        options(att_syntax, nostack),
                    );
                }
                r
            };
            let cmp = unsafe { _mm_movemask_epi8(_mm_cmpeq_epi8(x, zero)) } as u32;
            if cmp == 0 {
                n += 16;
                s = unsafe { s.add(1) };
            } else {
                return n + cmp.trailing_zeros() as usize;
            }
        }
    }
}