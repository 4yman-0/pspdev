//! NOT TESTED

use crate::error::{NativeResult, native_error, native_result};
use psp_sys::sys;

pub fn is_plugged_in() -> NativeResult<bool> {
    native_result(unsafe { sys::scePowerIsPowerOnline() })
        .map(|plugged_in| plugged_in == 1)
}

pub fn battery_is_present() -> NativeResult<bool> {
    native_result(unsafe { sys::scePowerIsBatteryExist() })
        .map(|battery_present| battery_present == 1)
}

pub fn battery_is_charging() -> NativeResult<bool> {
    native_result(unsafe { sys::scePowerIsBatteryCharging() })
        .map(|charging| charging == 1)
}

/// Get the status of the battery charging
pub fn battery_charging_status() -> NativeResult<u32> {
    native_result(unsafe { sys::scePowerGetBatteryChargingStatus() })
}

pub fn battery_is_low() -> NativeResult<bool> {
    native_result(unsafe { sys::scePowerIsLowBattery() }).map(|low| low == 1)
}

pub fn battery_life_percentage() -> NativeResult<u8> {
    native_result(unsafe { sys::scePowerGetBatteryLifePercent() })
        .map(|percent| percent as u8)
}

/// Get battery life in minutes
pub fn battery_life_time() -> NativeResult<usize> {
    native_result(unsafe { sys::scePowerGetBatteryLifeTime() })
        .map(|life_time| life_time as usize)
}

/// Get temperature of the battery
pub fn battery_temperature() -> NativeResult<usize> {
    native_result(unsafe { sys::scePowerGetBatteryTemp() })
        .map(|temp| temp as usize)
}

// /// unknown? - crashes PSP in usermode
// pub fn scePowerGetBatteryElec() -> i32;

/// Get battery volt level
pub fn battery_voltage() -> i32 {
    unsafe { sys::scePowerGetBatteryVolt() }
}

/// Set CPU Frequency
///
/// # Parameters
///
/// - `cpufreq`: new CPU frequency, valid values are 1 - 333
pub fn set_cpu_frequency(cpu_freq: u16) -> NativeResult<()> {
    native_error(unsafe {
        sys::scePowerSetCpuClockFrequency(i32::from(cpu_freq))
    })
}

/// Set Bus Frequency
///
/// # Parameters
///
/// - `busfreq`: new BUS frequency, valid values are 1 - 166
pub fn set_bus_frequency(bus_freq: u16) -> NativeResult<()> {
    native_error(unsafe {
        sys::scePowerSetBusClockFrequency(i32::from(bus_freq))
    })
}

// /// Alias for scePowerGetCpuClockFrequencyInt
// ///
// /// # Return Value
// ///
// /// Frequency as integer
// pub fn scePowerGetCpuClockFrequency() -> usize {
//     (unsafe { sys::scePowerGetCpuClockFrequency() }) as usize
// }

pub fn cpu_frequency() -> usize {
    (unsafe { sys::scePowerGetCpuClockFrequencyInt() }) as usize
}

pub fn cpu_frequency_float() -> f32 {
    unsafe { sys::scePowerGetCpuClockFrequencyFloat() }
}

/*
/// Alias for scePowerGetBusClockFrequencyInt
///
/// # Return Value
///
/// Frequency as an integer
pub fn scePowerGetBusClockFrequency() -> i32;
*/

/// Get Bus frequency as Integer
///
/// # Return Value
///
/// Frequency as an integer
pub fn bus_clock_frequency() -> usize {
    (unsafe { sys::scePowerGetBusClockFrequencyInt() }) as usize
}

/// Get Bus frequency as Float
///
/// # Return Value
///
/// frequency as float
pub fn bus_clock_frequency_float() -> f32 {
    unsafe { sys::scePowerGetBusClockFrequencyFloat() }
}

/// Set Clock Frequencies
///
/// # Parameters
///
/// - `pllfreq`: pll frequency, valid from 19-333
/// - `cpufreq`: cpu frequency, valid from 1-333
/// - `busfreq`: bus frequency, valid from 1-166
///
/// and:
///
/// cpufreq <= pllfreq
/// busfreq*2 <= pllfreq
///
pub fn set_frequencies(
    pllfreq: u16,
    cpufreq: u16,
    busfreq: u16,
) -> NativeResult<()> {
    native_error(unsafe {
        sys::scePowerSetClockFrequency(
            i32::from(pllfreq),
            i32::from(cpufreq),
            i32::from(busfreq),
        )
    })
}

/// Lock power switch
///
/// Note: if the power switch is toggled while locked it will fire
/// immediately after being unlocked.
pub fn lock_power_switch() -> NativeResult<()> {
    native_error(unsafe { sys::scePowerLock(0) })
}

pub fn unlock_power_switch() -> NativeResult<()> {
    native_error(unsafe { sys::scePowerUnlock(0) })
}

/// Generate a power tick, preventing unit from powering off and turning off
/// display.
///
/// # Parameters
///
/// - `type_`: type of power tick to generate
///
/// # Return Value
///
/// 0 on success, < 0 on error.
pub fn generate_power_tick(tick: sys::PowerTick) -> NativeResult<()> {
    native_error(unsafe { sys::scePowerTick(tick) })
}

pub fn idle_timer() -> NativeResult<usize> {
    native_result(unsafe { sys::scePowerGetIdleTimer() })
        .map(|time| time as usize)
}

/// Enable Idle timer
pub fn enable_timer() -> NativeResult<()> {
    native_error(unsafe { sys::scePowerIdleTimerEnable(0) })
}

/// Disable Idle timer
pub fn disable_timer() -> NativeResult<()> {
    native_error(unsafe { sys::scePowerIdleTimerDisable(0) })
}

/// Request the PSP to go into standby
pub fn request_standby() {
    let _ = unsafe { sys::scePowerRequestStandby() };
}

/// Request the PSP to go into suspend
pub fn request_suspend() {
    let _ = unsafe { sys::scePowerRequestSuspend() };
}
