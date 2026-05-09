#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

use psp_apis::wlan;
use psp_sys::dprint;

psp_sys::module!("wlan", 0, 1);

fn psp_main() {
    psp_sys::enable_home_button();
	dprint!("Wlan powered on: {}", wlan::wlan_powered_on());
	dprint!("Wlan switch on: {}",wlan::wlan_switch_on());

	match wlan::ethernet_address() {
		Ok(addr) => dprint!("Wlan ethernet address: {addr:X?}"),
		Err(err) => dprint!("wlan ethernet address: Error: {err}"),
	}
}
