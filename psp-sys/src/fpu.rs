//! Allegrex FPU control (FCR31) helpers.
//!
//! PSP defaults enable FPU exception traps that desktop IEEE-754 hardware does
//! not. Operations like 0/0, x/0, or overflow that produce NaN/Inf on x86_64
//! will instead fault on PSP unless the enable bits in FCR31 are cleared. The
//! thread that runs `main` (spawned by module.zig) clears these bits at entry;
//! user-spawned threads start with PSP defaults and can call `setIEEE754`
//! themselves if they want desktop-like semantics.

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    // TODO: better storage
    pub struct Fcr31: u32 {
        const ROUND_MODE_NEAREST = 0;
        const ROUND_MODE_ZERO = 1;
        const ROUND_MODE_NEG_INF = 2;
        const ROUND_MODE_POS_INF = 3;
        const FLAG_INEXACT = 1 << 2;
        const FLAG_UNDERFLOW = 1 << 3;
        const FLAG_OVERFLOW = 1 << 4;
        const FLAG_DIV_ZERO = 1 << 5;
        const FLAG_INVALID = 1 << 6;
        const ENABLE_INEXACT = 1 << 7;
        const ENABLE_UNDERFLOW = 1 << 8;
        const ENABLE_OVERFLOW = 1 << 9;
        const ENABLE_DIV_ZERO = 1 << 10;
        const ENABLE_INVALID = 1 << 11;
        const CAUSE_INEXACT = 1 << 12;
        const CAUSE_UNDERFLOW = 1 << 13;
        const CAUSE_OVERFLOW = 1 << 14;
        const CAUSE_DIV_ZERO = 1 << 15;
        const CAUSE_INVALID = 1 << 16;
        const CAUSE_UNIMPLEMENTED = 1 << 17;
        // TODO: cc0 and fs
    }
}

unsafe fn get_fcr31_raw() -> u32 {
    let r: u32;
    unsafe {
        core::arch::asm!(
            "cfc1 {r}, $31",
            r = out(reg) r,
        );
    }
    r
}

unsafe fn set_fcr31_raw(r: u32) {
    unsafe {
        core::arch::asm!(
            ".set noat",
            "ctc1 {r}, $31",
            r = in(reg) r,
        );
    }
}

pub fn get_fcr31() -> Fcr31 {
    unsafe { core::mem::transmute::<u32, Fcr31>(get_fcr31_raw()) }
}

pub fn set_fcr31(r: Fcr31) {
    unsafe { set_fcr31_raw(core::mem::transmute::<Fcr31, u32>(r)) }
}

pub fn set_fcr31_ieee754() {
    set_fcr31(Fcr31::ROUND_MODE_NEAREST)
}

pub fn set_fcr31_psp_default() {
    set_fcr31(
        Fcr31::ROUND_MODE_NEAREST
            .union(Fcr31::ENABLE_INVALID)
            .union(Fcr31::ENABLE_DIV_ZERO)
            .union(Fcr31::ENABLE_OVERFLOW),
    )
}
