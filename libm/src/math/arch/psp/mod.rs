use psp_vfpu::vfpu_asm;

const HALF_PI: f32 = ::core::f32::consts::PI / 2.0;

pub fn sqrtf(x: f32) -> f32 {
    let out: f32;
    unsafe {
        vfpu_asm! (
            "mtv {x}, S000",
            "nop",
            "vsqrt.s S000, S000",
            "mfv {out}, S000",
            "nop",
            x = in(reg) x,
            out = out(reg) out,
            options(nostack, nomem),
        );
    }
    out
}

pub fn exp2f(x: f32) -> f32 {
    let out: f32;
    unsafe {
        vfpu_asm! (
            "mtv {x}, S000",
            "nop",
            "vexp2.s S000, S000",
            "mfv {out}, S000",
            "nop",
            x = in(reg) x,
            out = out(reg) out,
            options(nostack, nomem),
        );
    }
    out
}

pub fn log2f(x: f32) -> f32 {
    let out: f32;
    unsafe {
        vfpu_asm! (
            "mtv {x}, S000",
            "nop",
            "vlog2.s S000, S000",
            "mfv {out}, S000",
            "nop",
            x = in(reg) x,
            out = out(reg) out,
            options(nostack, nomem),
        );
    }
    out
}

/*pub fn logbf(x: f32) -> f32 {
    let out: f32;
    unsafe {
        vfpu_asm! (
            "mtv {x}, S000",
            "nop",
            "vlogb.s S000, S000",
            "mfv {out}, S000",
            "nop",
            x = in(reg) x,
            out = out(reg) out,
            options(nostack, nomem),
        );
    }
    out
}

pub fn ilog2f(x: f32) -> i32 {
    log2f(x) as i32
}

pub fn ilogbf(x: f32) -> i32 {
    logbf(x) as i32
}*/

pub fn cosf(x: f32) -> f32 {
    let x = x / HALF_PI;
    let out: f32;
    unsafe {
        vfpu_asm! (
            "mtv {x}, S000",
            "nop",
            "vcos.s S000, S000",
            "mfv {out}, S000",
            "nop",
            x = in(reg) x,
            out = out(reg) out,
            options(nostack, nomem),
        );
    }
    out
}

pub fn sinf(x: f32) -> f32 {
    cosf(x - HALF_PI)
}

pub fn sincosf(x: f32) -> (f32, f32) {
    (sinf(x), cosf(x))
}

pub fn tanf(x: f32) -> f32 {
    sinf(x) / cosf(x)
}

/* asin and acos are inaccurate (Absolute error is smaller than 0.02)
pub fn asinf(x: f32) -> f32 {
    let x = x / HALF_PI;
    let out: f32;
    unsafe {
        vfpu_asm! (
            "mtv {x}, S000",
            "nop",
            "vasin.s S000, S000",
            "mfv {out}, S000",
            "nop",
            x = in(reg) x,
            out = out(reg) out,
            options(nostack, nomem),
        );
    }
    out
}*/
