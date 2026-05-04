use super::vram_alloc::{VramAllocator, VramAllocatorError};
use core::ptr::NonNull;
use core::slice;
use psp_sys::sys::TexturePixelFormat;
pub type TextureResult<T> = Result<T, TextureError>;

#[derive(Clone, Debug)]
pub enum TextureError {
    VramAllocatorError(VramAllocatorError),

    TextureTooSmall,
    WidthNotPowerOfTwo,
    InvalidInternalBuffer,
}

impl From<VramAllocatorError> for TextureError {
    fn from(from: VramAllocatorError) -> Self {
        Self::VramAllocatorError(from)
    }
}

impl core::fmt::Display for TextureError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        write!(f, "{self:?}")
    }
}
impl core::error::Error for TextureError {}

pub fn texture_pixel_size(psm: TexturePixelFormat) -> usize {
    match psm {
        //TexturePixelFormat::PsmT4 => length >> 1,
        TexturePixelFormat::PsmT8 => 1,

        TexturePixelFormat::Psm5650
        | TexturePixelFormat::Psm5551
        | TexturePixelFormat::Psm4444
        | TexturePixelFormat::PsmT16 => 2,

        TexturePixelFormat::Psm8888 | TexturePixelFormat::PsmT32 => 4,

        // TODO: support other texture types (Dxt1/3/5)
        _ => unimplemented!(),
    }
}

// TODO: figure out why RAM texture don't work
pub struct Texture {
    size: (u16, u16),
    format: TexturePixelFormat,
    swizzled: bool,
    buffer: NonNull<[u8]>,
}

impl Texture {
    /// Allocates a new `Texture`.
    // TODO: documentation is unclear.
    pub fn allocate(
        vram_allocator: &mut VramAllocator,
        width: u16,
        height: u16,
        format: TexturePixelFormat,
        swizzled: bool,
    ) -> TextureResult<Self> {
        Self::from_slice(
            unsafe {
                let length = width as usize * height as usize;
                let size = length * texture_pixel_size(format);
                let ptr = vram_allocator.allocate(size)?;
                slice::from_raw_parts(ptr.cast(), size)
            },
            width,
            height,
            format,
            swizzled,
        )
    }

    pub fn from_slice(
        slice: &[u8],
        width: u16,
        height: u16,
        format: TexturePixelFormat,
        swizzled: bool,
    ) -> TextureResult<Self> {
        if !width.is_power_of_two() {
            return Err(TextureError::WidthNotPowerOfTwo);
        }
        if width < 4 || height == 0 {
            return Err(TextureError::TextureTooSmall);
        }
        if usize::from(width) * usize::from(height) * texture_pixel_size(format)
            != core::mem::size_of_val(slice)
        {
            return Err(TextureError::InvalidInternalBuffer);
        }
        Ok(Self {
            size: (width, height),
            swizzled,
            format,
            buffer: NonNull::from_ref(slice),
        })
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }
    pub fn width(&self) -> u16 {
        self.size.0
    }
    pub fn height(&self) -> u16 {
        self.size.1
    }
    pub fn format(&self) -> TexturePixelFormat {
        self.format
    }
    pub fn swizzled(&self) -> bool {
        self.swizzled
    }
    pub fn set_swizzled(&mut self, swizzled: bool) {
        self.swizzled = swizzled;
    }
    pub fn buffer(&self) -> &[u8] {
        unsafe { self.buffer.as_ref() }
    }
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe { self.buffer.as_mut() }
    }
    pub fn can_be_framebuffer(&self) -> bool {
        use TexturePixelFormat as TexelFmt;
        if self.swizzled {
            return false;
        }
        matches!(
            self.format,
            TexelFmt::Psm8888
                | TexelFmt::Psm4444
                | TexelFmt::Psm5650
                | TexelFmt::Psm5551
        )
    }

    pub fn swizzle_from_slice(&mut self, src: &[u8]) -> Option<()> {
        let (width, height) = (self.width() as usize, self.height() as usize);
        if src.len() != width * height * texture_pixel_size(self.format) {
            return None;
        }
        if !width.is_multiple_of(16) || !height.is_multiple_of(8) {
            return None;
        }

        let dst = self.buffer_mut();

        // TODO: actual swizzling
        let rowblocks = width / 16;

        for y in 0..height {
            for x in 0..width {
                let l_x = x % 16;
                let l_y = y % 8;
                let blockx = (x - l_x) / 16;
                let blocky = (y - l_y) / 8;
                let block_index = blockx + (blocky * rowblocks);
                let block_address = block_index * 16 * 8;

                dst[block_address + (l_y * 16) + l_x] = src[x + (y * width)];
            }
        }

        let _ = dst;

        self.swizzled = true;
        Some(())
    }
}
