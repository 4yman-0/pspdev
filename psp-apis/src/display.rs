use psp_sys::sys;

/// Wait for vertical blank start
pub fn wait_vblank_start() -> i32 {
    unsafe { sys::sceDisplayWaitVblankStart() }
}

/// Wait for vertical blank
pub fn wait_vblank() -> i32 {
    unsafe { sys::sceDisplayWaitVblank() }
}

/// Get frames per second (fps)
#[must_use]
pub fn frames_per_second() -> f32 {
    unsafe { sys::sceDisplayGetFramePerSec() }
}

#[must_use]
pub fn is_foreground() -> bool {
    let is_foreground = unsafe { sys::sceDisplayIsForeground() };
    if is_foreground == 1 {
        true
    } else if is_foreground == 0 {
        false
    } else {
        panic!("Invalid data from sceDisplayIsForeground");
    }
}
