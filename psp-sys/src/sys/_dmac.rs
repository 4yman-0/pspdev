// TODO: missing flags and version

use core::ffi::c_void;

psp_extern! {
    #![name = "sceDmac"]
    #![flags = 0x4001]
    #![version = (0x00, 0x00)]

	#[psp(0x617F3FE6)]
	/* DMA memcpy.
	
	## Parameters
	[in]	dst	- Destination
	[in]	src	- Source
	[in]	size	- Size
	## Returns
	< 0 on error.
	*/
    pub fn sceDmacMemcpy(*mut c_void dst, *const void src, usize size) -> i32;

    /*#[psp(0xD97F94D8)]
    pub fn sceDmacTryMemcpy();*/
}
