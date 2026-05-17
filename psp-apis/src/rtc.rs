use crate::error::{NativeResult, native_error};
use core::mem::MaybeUninit;
use psp_sys::sys;

pub fn ticks_per_second() -> u32 {
    unsafe { sys::sceRtcGetTickResolution() }
}

pub fn current_tick() -> NativeResult<u64> {
    let mut tick = 0_u64;
    native_error(unsafe { sys::sceRtcGetCurrentTick(&mut tick) })?;
    Ok(tick)
}

pub fn current_clock(
    timezone_minutes: i32,
) -> NativeResult<sys::ScePspDateTime> {
    let mut date_time = MaybeUninit::<sys::ScePspDateTime>::uninit();
    native_error(unsafe {
        sys::sceRtcGetCurrentClock(
            date_time.assume_init_mut(),
            timezone_minutes,
        )
    })?;
    Ok(unsafe { date_time.assume_init() })
}

pub fn current_local_clock() -> NativeResult<sys::ScePspDateTime> {
    let mut date_time = MaybeUninit::<sys::ScePspDateTime>::uninit();
    native_error(unsafe {
        sys::sceRtcGetCurrentClockLocalTime(date_time.assume_init_mut())
    })?;
    Ok(unsafe { date_time.assume_init() })
}

pub fn ticks_utc_to_local(utc: u64) -> NativeResult<u64> {
    let mut local = 0_u64;
    native_error(unsafe {
        sys::sceRtcConvertUtcToLocalTime(&utc, &mut local)
    })?;
    Ok(local)
}

pub fn is_leap_year(year: i32) -> bool {
    unsafe { sys::sceRtcIsLeapYear(year) == 1 }
}

pub fn days_in_month(year: i32, month: u8) -> u32 {
    // error (?)
    unsafe { sys::sceRtcGetDaysInMonth(year, month as i32) as u32 }
}

numeric_enum_macro::numeric_enum! {
    // are there any misspells?
    #[repr(u8)]
    #[derive(Clone, Copy, Debug)]
    pub enum DayOfWeek {
        Sunday = 0,
        Monday = 1,
        Tuesday = 2,
        Wednesday = 3,
        Thursday = 4,
        Friday = 5,
        Saturday = 6,
    }
}

pub fn day_of_week(year: i32, month: u8, day: u8) -> DayOfWeek {
    unsafe {
        let result =
            sys::sceRtcGetDayOfWeek(year, month as i32, day as i32) as u8;
        result.try_into().unwrap()
    }
}

pub fn check_date_time(date_time: &sys::ScePspDateTime) -> NativeResult<()> {
    native_error(unsafe { sys::sceRtcCheckValid(date_time) })
}

pub fn date_time_from_ticks(ticks: u64) -> NativeResult<sys::ScePspDateTime> {
    let mut date_time = MaybeUninit::<sys::ScePspDateTime>::uninit();
    native_error(unsafe {
        sys::sceRtcSetTick(date_time.assume_init_mut(), &ticks)
    })?;
    Ok(unsafe { date_time.assume_init() })
}

pub fn ticks_from_date_time(
    date_time: &sys::ScePspDateTime,
) -> NativeResult<u64> {
    let mut ticks = 0_u64;
    native_error(unsafe { sys::sceRtcGetTick(date_time, &mut ticks) })?;
    Ok(ticks)
}

// TODO: Do we have to implement sceRtc(Compare|Add)Tick[s]...?
// TODO: Do we have to implement parsers (PspDateTime -> string)?
