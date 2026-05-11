#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

use psp_apis::power;
use psp_sys::dprint;

psp_sys::module!("clock-speed", 0, 1);

fn print_clock_speeds() {
    dprint!(
        " cpu: {}Mhz, bus: {}Mhz",
        power::cpu_frequency(),
        power::bus_frequency(),
    );
}

fn psp_main() {
    psp_sys::enable_home_button();

    dprint!("init clock speeds");
    print_clock_speeds();

    power::set_frequencies(266, 266, 266 / 2).unwrap();

    dprint!("new clock speeds");
    print_clock_speeds();
}
