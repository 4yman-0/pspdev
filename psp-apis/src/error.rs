use core::fmt;

pub type NativeResult<T> = Result<T, NativeError>;

#[derive(Clone)]
pub struct NativeError(i32);

impl NativeError {
    pub fn facility(&self) -> NativeFacility {
        let facility = ((self.0 >> 16) & 0xfff) as u16;
        facility.try_into().unwrap()
    }

    pub const fn is_critical(&self) -> bool {
        ((self.0 >> 30) & 1) == 1
    }

    pub(crate) const fn inner(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Native {} {:?} error: {:X}",
            if self.is_critical() {
                "Critical"
            } else {
                "Non-critical"
            },
            self.facility(),
            self.0
        )
    }
}

impl fmt::Debug for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{self}")
    }
}

impl core::error::Error for NativeError {}

impl TryFrom<i32> for NativeError {
    type Error = ();
    fn try_from(from: i32) -> Result<Self, Self::Error> {
        if from < 0 { Ok(Self(from)) } else { Err(()) }
    }
}

#[inline]
pub(crate) fn native_result(result: i32) -> Result<u32, NativeError> {
    match result.try_into() {
        Err(()) => Ok(result as u32),
        Ok(err) => Err(err),
    }
}

#[inline]
pub(crate) fn native_error(result: i32) -> Result<(), NativeError> {
    match result.try_into() {
        Err(()) => Ok(()),
        Ok(err) => Err(err),
    }
}

numeric_enum_macro::numeric_enum! {
    #[repr(u16)]
    #[derive(Copy, Clone, Debug)]
    pub enum NativeFacility {
        Codec = 0x7F,
        Aac = 0x69,
        G729 = 0x68,
        Mp3 = 0x67,
        Avi = 0x66,
        Jpeg = 0x65,
        Asf = 0x64,
        Atrac = 0x63,
        Avc = 0x62,
        Mpeg = 0x61,
        Library = 0x5F,
        Face = 0x58,
        Fmac = 0x57,
        Gameupdate = 0x56,
        Np = 0x55,
        Mtp = 0x54,
        Dnas = 0x53,
        Openpsid = 0x52,
        Cphio = 0x51,
        Magicgate = 0x50,
        P3da = 0x47,
        Font = 0x46,
        Snd = 0x45,
        Wave = 0x44,
        Http = 0x43,
        Sas = 0x42,
        Network = 0x41,
        Periph = 0x3F,
        Mediasync = 0x2D,
        Audiorouting = 0x2C,
        Power = 0x2B,
        Irda = 0x2A,
        Sircs = 0x29,
        Lfatfs = 0x28,
        Lflash = 0x27,
        Audio = 0x26,
        Syscon = 0x25,
        Usb = 0x24,
        Flash = 0x23,
        Memstick = 0x22,
        Umd = 0x21,
        Msapp = 0x13,
        Sysfile = 0x12,
        Utility = 0x11,
        Vsh = 0x10,
        Registry = 0x08,
        Kernel = 0x02,
        Errno = 0x01,
        Null = 0x00,
    }
}
