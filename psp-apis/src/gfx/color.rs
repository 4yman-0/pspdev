#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct Color32(u32);

impl Color32 {
    pub const BLACK: Self = Self::from_rgb(0x00_00_00);
    pub const RED: Self = Self::from_rgb(0xff_00_00);
    pub const GREEN: Self = Self::from_rgb(0x00_ff_00);
    pub const BLUE: Self = Self::from_rgb(0x00_00_ff);
    pub const WHITE: Self = Self::from_rgb(0xff_ff_ff);
    pub const YELLOW: Self = Self::from_rgb(0xff_ff_00);
    pub const CYAN: Self = Self::from_rgb(0x00_ff_ff);
    pub const MAGENTA: Self = Self::from_rgb(0xff_00_ff);
    pub const PURPLE: Self = Self::from_rgb(0x80_00_80);
    pub const ORANGE: Self = Self::from_rgb(0xff_a5_00);
    pub const BROWN: Self = Self::from_rgb(0xa5_2a_2a);
    pub const PINK: Self = Self::from_rgb(0xff_c0_cb);
    pub const GRAY: Self = Self::from_rgb(0x80_80_80);
    pub const LIGHT_GRAY: Self = Self::from_rgb(0xd3_d3_d3);
    pub const DARK_GRAY: Self = Self::from_rgb(0x40_40_40);
    pub const TRANSPARENT: Self = Self::from_rgba(0x00_00_00_00);

    /// Create a new [`Color32`] from an integer in the R8G8B8A8 format
    #[must_use]
    pub const fn from_rgba(x: u32) -> Self {
        Self(x.swap_bytes())
    }
    #[must_use]
    pub const fn from_rgba_channels(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_abgr_channels(a, b, g, r)
    }
    /// Create a new [`Color32`] from an integer in the A8B8G8R8 format (Native PSP)
    #[must_use]
    pub const fn from_abgr(x: u32) -> Self {
        Self(x)
    }
    #[must_use]
    pub const fn from_abgr_channels(a: u8, b: u8, g: u8, r: u8) -> Self {
        Self((a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32))
    }
    /// Create a new [`Color32`] from an integer in the A8B8G8R8 format (native PSP without alpha)
    #[must_use]
    pub const fn from_bgr(x: u32) -> Self {
        Self(x & 0x00_ff_ff_ff)
    }
    #[must_use]
    pub const fn from_bgr_channels(b: u8, g: u8, r: u8) -> Self {
        Self::from_abgr_channels(0xff, b, g, r)
    }
    /// Create a new [`Color32`] from an integer in the R8G8B8 format
    #[must_use]
    pub const fn from_rgb(x: u32) -> Self {
        Self::from_rgba(x << 8 | 0xff)
    }
    #[must_use]
    pub const fn from_rgb_channels(r: u8, g: u8, b: u8) -> Self {
        Self::from_abgr_channels(0xff, b, g, r)
    }
    /// Get the color in the R8G8B8A8 format
    #[must_use]
    pub const fn as_rgba(&self) -> u32 {
        self.0.swap_bytes()
    }
    /// Get the color in the A8B8G8R8 format (native PSP)
    #[must_use]
    pub const fn as_abgr(&self) -> u32 {
        self.0
    }
    /// Get the color in the B8G8R8 format (native PSP without alpha)
    #[must_use]
    pub const fn as_bgr(&self) -> u32 {
        self.0 & 0x00_ff_ff_ff
    }
    /// Get the color in the R8G8B8 format (without alpha)
    #[must_use]
    pub const fn as_rgb(&self) -> u32 {
        self.as_rgba() & 0xff_ff_ff_00
    }

    /// Get the red component of the color
    #[must_use]
    pub const fn r(&self) -> u8 {
        self.0 as u8
    }
    /// Get the green component of the color
    #[must_use]
    pub const fn g(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    /// Get the blue component of the color
    #[must_use]
    pub const fn b(&self) -> u8 {
        (self.0 >> 16) as u8
    }
    /// Get the alpha component of the color
    #[must_use]
    pub const fn a(&self) -> u8 {
        (self.0 >> 24) as u8
    }
}
