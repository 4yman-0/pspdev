use super::vram_alloc::VramChunk;
use alloc::boxed::Box;
use psp_sys::sys::TexturePixelFormat;

pub type TextureResult<T> = Result<T, TextureError>;

#[derive(Debug)]
pub enum TextureError {
    InvalidTexture,
}

impl core::fmt::Display for TextureError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        write!(f, "Texture error: {self:?}")
    }
}
impl core::error::Error for TextureError {}

pub fn texture_size(len: usize, psm: TexturePixelFormat) -> usize {
    match psm {
        TexturePixelFormat::PsmT4 | TexturePixelFormat::PsmDxt1 => len / 2,
        TexturePixelFormat::PsmT8
        | TexturePixelFormat::PsmDxt5
        | TexturePixelFormat::PsmDxt3 => len,

        TexturePixelFormat::Psm5650
        | TexturePixelFormat::Psm5551
        | TexturePixelFormat::Psm4444
        | TexturePixelFormat::PsmT16 => len * 2,

        TexturePixelFormat::Psm8888 | TexturePixelFormat::PsmT32 => len * 4,
    }
}

enum TextureBuffer {
    Ram(Box<[u8]>),
    Vram(VramChunk),
}

impl core::ops::Deref for TextureBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Self::Ram(boxed) => boxed.as_ref(),
            Self::Vram(chunk) => chunk.as_ref(),
        }
    }
}

impl core::ops::DerefMut for TextureBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Ram(boxed) => boxed.as_mut(),
            Self::Vram(chunk) => chunk.as_mut(),
        }
    }
}

// // TODO: figure out why RAM texture don't work
pub struct Texture {
    size: (u16, u16),
    format: TexturePixelFormat,
    swizzled: bool,
    buffer: TextureBuffer,
}

impl Texture {
    /// Allocates a new `Texture` from VRAM.
    // // TODO: documentation is unclear.
    pub fn allocate(
        width: u16,
        height: u16,
        format: TexturePixelFormat,
        swizzled: bool,
    ) -> TextureResult<Self> {
        Self::from_vram_chunk(width, height, format, swizzled, {
            let size = texture_size(width as usize * height as usize, format);
            // align to 4 by default
            VramChunk::alloc(size, 4)
        })
    }

    pub fn from_vram_chunk(
        width: u16,
        height: u16,
        format: TexturePixelFormat,
        swizzled: bool,
        chunk: VramChunk,
    ) -> TextureResult<Self> {
        if chunk.len() != texture_size(width as usize * height as usize, format)
        {
            return Err(TextureError::InvalidTexture);
        }
        Ok(Self {
            size: (width, height),
            buffer: TextureBuffer::Vram(chunk),
            format,
            swizzled,
        })
    }
    pub fn from_boxed_slice(
        width: u16,
        height: u16,
        format: TexturePixelFormat,
        swizzled: bool,
        boxed: Box<[u8]>,
    ) -> TextureResult<Self> {
        if boxed.len() != texture_size(width as usize * height as usize, format)
        {
            return Err(TextureError::InvalidTexture);
        }
        Ok(Self {
            size: (width, height),
            buffer: TextureBuffer::Ram(boxed),
            format,
            swizzled,
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
        &self.buffer
    }
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
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

    pub fn can_be_depthbuffer(&self) -> bool {
        use TexturePixelFormat as TexelFmt;
        if self.swizzled {
            return false;
        }
        matches!(
            self.format,
               TexelFmt::Psm5650
                |TexelFmt::Psm4444
                | TexelFmt::Psm5551
        )
    }

    pub fn swizzle_from_slice(&mut self, src: &[u8]) -> Option<()> {
        let (width, height) = (self.width() as usize, self.height() as usize);
        if src.len() != texture_size(width * height, self.format) {
            return None;
        }
        if !width.is_multiple_of(16) || !height.is_multiple_of(8) {
            return None;
        }
        todo!();
        //Some(())
    }
}
