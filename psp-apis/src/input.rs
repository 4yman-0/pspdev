use psp_sys::sys::{self, SceCtrlData};

fn ctrl_read_buffer(ctrl_data: &mut SceCtrlData) {
    unsafe {
        sys::sceCtrlReadBufferPositive(ctrl_data, 1);
    }
}

pub use psp_sys::sys::CtrlButtons as Buttons;

#[derive(Clone, Default)]
pub struct Input {
    ctrl_data: SceCtrlData,
    just_modified: Buttons,
}

impl Input {
    #[must_use]
    pub fn init(control_mode: sys::CtrlMode) -> Self {
        unsafe {
            sys::sceCtrlSetSamplingCycle(0);
            sys::sceCtrlSetSamplingMode(control_mode);
        }
        Self::default()
    }

    pub fn read_mut(&mut self) {
        let previous_ctrl = self.ctrl_data.buttons;
        ctrl_read_buffer(&mut self.ctrl_data);
        self.just_modified = self.ctrl_data.buttons ^ previous_ctrl;
    }

    #[must_use]
    pub const fn timestamp(&self) -> u32 {
        self.ctrl_data.timestamp
    }

    #[must_use]
    pub const fn button_pressed(&self, button: Buttons) -> bool {
        // `button` is `Copy`.
        self.ctrl_data.buttons.contains(button)
    }

    #[must_use]
    pub const fn button_just_modified(&self, button: Buttons) -> bool {
        self.just_modified.contains(button)
    }

    #[must_use]
    pub const fn button_just_pressed(&self, button: Buttons) -> bool {
        self.button_pressed(button) && self.button_just_modified(button)
    }

    #[must_use]
    pub const fn button_just_released(&self, button: Buttons) -> bool {
        (!self.button_pressed(button)) && self.button_just_modified(button)
    }

    #[must_use]
    pub const fn analog_stick(&self) -> (u8, u8) {
        (self.ctrl_data.lx, self.ctrl_data.ly)
    }
}
