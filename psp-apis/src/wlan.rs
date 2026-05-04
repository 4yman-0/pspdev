use crate::error::{NativeResult, native_error};
use psp_sys::sys;

pub fn wlan_powered_on() -> bool {
    unsafe { sys::sceWlanDevIsPowerOn() == 1 }
}

pub fn wlan_switch_on() -> bool {
    unsafe { sys::sceWlanGetSwitchState() == 1 }
}

pub fn ethernet_address() -> NativeResult<[u8; 8]> {
    let mut addr = [0_u8; 8];
    native_error(unsafe { sys::sceWlanGetEtherAddr(addr.as_mut_ptr()) })?;
    Ok(addr)
}

pub fn device_attach() -> NativeResult<()> {
    native_error(unsafe { sys::sceWlanDevAttach() })
}

pub fn device_detach() -> NativeResult<()> {
    native_error(unsafe { sys::sceWlanDevDetach() })
}
