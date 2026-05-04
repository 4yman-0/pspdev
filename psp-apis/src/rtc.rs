use crate::error::{NativeResult, native_error};
use psp_sys::sys;

pub fn ticks_per_second() -> u32 {
    unsafe { sys::sceRtcGetTickResolution() }
}

pub fn current_tick() -> NativeResult<u64> {
    let mut tick = 0_u64;
    native_error(unsafe { sys::sceRtcGetCurrentTick(&raw mut tick) })?;
    Ok(tick)
}

pub fn current_clock(
    timezone_minutes: i32,
) -> NativeResult<sys::ScePspDateTime> {
    let mut date_time = sys::ScePspDateTime::default();
    native_error(unsafe {
        sys::sceRtcGetCurrentClock(&raw mut date_time, timezone_minutes)
    })?;
    Ok(date_time)
}

pub fn current_local_clock() -> NativeResult<sys::ScePspDateTime> {
    let mut date_time = sys::ScePspDateTime::default();
    native_error(unsafe {
        sys::sceRtcGetCurrentClockLocalTime(&raw mut date_time)
    })?;
    Ok(date_time)
}

pub fn ticks_utc_to_local(utc: u64) -> NativeResult<u64> {
    let mut local = 0_u64;
    native_error(unsafe {
        sys::sceRtcConvertUtcToLocalTime(&raw const utc, &raw mut local)
    })?;
    Ok(local)
}

pub fn is_leap_year(year: i32) -> bool {
    unsafe { sys::sceRtcIsLeapYear(year) == 1 }
}

pub fn days_in_month(year: i32, month: i32) -> u32 {
    // error (?)
    unsafe { sys::sceRtcGetDaysInMonth(year, month) as u32 }
}

numeric_enum_macro::numeric_enum! {
    // are there any misspells?
    #[repr(u8)]
    pub enum DayOfWeek {
        Monday = 0,
        Tuesday = 1,
        Wednesday = 2,
        Thursday = 3,
        Friday = 4,
        Saturday = 5,
        Sunday = 6,
    }
}

pub fn day_of_week(year: i32, month: i32, day: i32) -> DayOfWeek {
    unsafe {
        let result = sys::sceRtcGetDayOfWeek(year, month, day) as u8;
        result.try_into().unwrap()
    }
}

// etc...
