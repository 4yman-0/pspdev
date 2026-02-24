/// Texture pixel formats
// TODO: Better documentation
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[allow(dead_code)]
pub enum TexturePixelFormat {
    /// Hicolor, 16-bit.
    Psm5650 = 0,
    /// Hicolor, 16-bit
    Psm5551 = 1,
    /// Hicolor, 16-bit
    Psm4444 = 2,
    /// Truecolor, 32-bit
    Psm8888 = 3,
    /// Indexed, 4-bit (2 pixels per byte)
    PsmT4 = 4,
    /// Indexed, 8-bit
    PsmT8 = 5,
    /// Indexed, 16-bit
    PsmT16 = 6,
    /// Indexed, 32-bit
    PsmT32 = 7,
    PsmDxt1 = 8,
    PsmDxt3 = 9,
    PsmDxt5 = 10,
}
