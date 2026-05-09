#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

use psp_apis::rtc;
use psp_sys::{dprint, sys};

psp_sys::module!("time", 0, 1);

fn print_clock(clock: &sys::ScePspDateTime) {
	dprint!(
		" {}/{}/{} {}:{}:{}",
		clock.day,
		clock.month,
		clock.year,
		clock.hour,
		clock.minutes,
		clock.seconds,
	);
	dprint!(
		" Day of week: {:?}",
		rtc::day_of_week(
			clock.year.into(),
			clock.month.into(),
			clock.day.into()
		),
	);
}

fn psp_main() {
    psp_sys::enable_home_button();

	dprint!(
		"Ticks per second: {}",
		rtc::ticks_per_second(),
	);

    let clock = rtc::current_local_clock().unwrap();
    dprint!("Current local clock: ");
    print_clock(&clock);
}
