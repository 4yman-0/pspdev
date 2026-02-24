psp_extern! {
    #![name = "sceVideocodec"]
    #![flags = 0x4001]
    #![version = (0x00, 0x11)]

    #[psp(0xC01EC829)]
    pub fn sceVideocodecOpen(
        buffer: *mut u32,
        type_: i32,
    ) -> i32;

    #[psp(0x2D31F5B1)]
    pub fn sceVideocodecGetEDRAM(
        buffer: *mut u32,
        type_: i32,
    ) -> i32;

    #[psp(0x17099F0A)]
    pub fn sceVideocodecInit(
        buffer: *mut u32,
        type_: i32,
    ) -> i32;

    #[psp(0xDBA273FA)]
    pub fn sceVideocodecDecode(
        buffer: *mut u32,
        type_: i32,
    ) -> i32;

    #[psp(0x4F160BF4)]
    pub fn sceVideocodecReleaseEDRAM(buffer: *mut u32) -> i32;
}

// from PPSSPP's source code
#[repr(C)]
pub struct SceAudiocodecCodec {
    pub unk_init: i32,
    pub unk4: i32,
    pub error: i32,
    /// presumably in ME memory?
    pub edram_address: i32,
    /// 0x102400 for Atrac3+
    pub needed_memory: i32,
    pub inited: i32,
    /// Before decoding, set this to the start of the raw frame
    pub input_buffer: *const u8,
    pub input_bytes_read: i32,
    /// This is where the decoded data is written
    pub output_buffer: *mut i16,
    pub output_samples_written: i32,
    /// This is probably a union with different fields for different codecs
    pub codec_specific: [u32; 2],
    /// Atrac3 (non-+) related. Zero with Atrac3+
    pub unk48: i32,
    pub unk52: i32,
    pub mp3_9999: i32,
    /// gets the value 3
    pub mp3_3: i32,
    /// Atrac3+ size related
    pub unk64: i32,
    pub mp3_9: i32,
    pub mp3_0: i32,
    pub unk76: i32,
    pub unk80: i32,
    pub mp3_1_first: i32,
    pub unk88: i32,
    pub unk92: i32,
    pub mp3_1: i32,
    pub unk100: i32,
    pub alloc_mem: u32,
    /// Make sure the size is 128
    pub padding: [u8; 20],
}

#[derive(Copy, Clone)]
#[repr(i32)]
pub enum AudioCodec {
    At3Plus = 0x00001000,
    At3 = 0x00001001,
    Mp3 = 0x00001002,
    Aac = 0x00001003,
}

psp_extern! {
    #![name = "sceAudiocodec"]
    #![flags = 0x4009]
    #![version = (0x00, 0x00)]

    #[psp(0x9D3F790C)]
    pub fn sceAudiocodecCheckNeedMem(
        buffer: *mut SceAudiocodecCodec,
        type_: i32,
    ) -> i32;

    #[psp(0x5B37EB1D)]
    pub fn sceAudiocodecInit(
        buffer: *mut SceAudiocodecCodec,
        type_: i32,
    ) -> i32;

    #[psp(0x70A703F8)]
    pub fn sceAudiocodecDecode(
        buffer: *mut SceAudiocodecCodec,
        type_: i32,
    ) -> i32;

    #[psp(0x3A20A200)]
    pub fn sceAudiocodecGetEDRAM(
        buffer: *mut SceAudiocodecCodec,
        type_: i32,
    ) -> i32;

    #[psp(0x29681260)]
    pub fn sceAudiocodecReleaseEDRAM(buffer: *mut SceAudiocodecCodec) -> i32;

    #[psp(0x8ACA11D5)]
    pub fn sceAudiocodecGetInfo(
        buffer: *mut SceAudiocodecCodec,
        unk: i32,
    ) -> i32;

    #[psp(0x59176A0F)]
    pub fn sceAudiocodecGetOutputBytes(
        buffer: *mut SceAudiocodecCodec,
        unk: i32,
        ptr: *mut u32,
    ) -> i32;

    #[psp(0x3DD7EE1A)]
    pub fn sceAudiocodecInitMono(
        buffer: *mut SceAudiocodecCodec,
        type_: i32
    ) -> i32;

}
