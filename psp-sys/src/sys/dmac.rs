// TODO: missing flags and version

use core::ffi::c_void;

psp_extern! {
    #![name = "sceDmac"]
    #![flags = 0x4001]
    #![version = (0x00, 0x11)]

    #[psp(0x617F3FE6)]
    /// Copy data in memory using DMAC
    /// `dst` - The pointer to the destination
    /// `src` - The pointer to the source
    /// `n` - The size of data
    /// Returns 0 on success; otherwise an error code
    pub fn sceDmacMemcpy(dst: *mut c_void, src: *const c_void, size: usize) -> i32;

    #[psp(0xD97F94D8)]
    /// Copy data in memory using DMAC
    /// `dst` - The pointer to the destination
    /// `src` - The pointer to the source
    /// `n` - The size of data
    /// Returns 0 on success; otherwise an error code
    pub fn sceDmacTryMemcpy(dst: *mut c_void, src: *const c_void, size: usize) -> i32;
}
