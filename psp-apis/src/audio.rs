use crate::error::{NativeError, NativeResult, native_error, native_result};
use core::ffi::c_void;
use psp_sys::sys;

pub type AudioResult<T> = Result<T, AudioError>;

#[derive(Clone, Debug)]
pub enum AudioError {
    Native(NativeError),

    SampleBufferTooSmall,
}

impl core::fmt::Display for AudioError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        write!(f, "Audio error: {:?}", self)
    }
}
impl core::error::Error for AudioError {}

impl From<NativeError> for AudioError {
    fn from(from: NativeError) -> Self {
        Self::Native(from)
    }
}

pub type Sample = i16;

#[derive(Clone)]
pub enum Volume {
    Mono(u32),
    Stereo(u32, u32),
}

impl Volume {
    pub const VOLUME_MAX: u32 = i16::MAX as u32;
    pub const VOLUME_MAX_F32: f32 = Self::VOLUME_MAX as f32;

    /// Converts the volume of a single channel (in the range 0.0..=1.0) to a [`Volume`]
    pub const fn from_mono_f32(mono: f32) -> Self {
        Self::Mono((mono * Self::VOLUME_MAX_F32) as u32)
    }

    /// Converts the volume of two channels (in the range 0.0..=1.0) to a [`Volume`]
    pub const fn from_stereo_f32(stereo: (f32, f32)) -> Self {
        Self::Stereo(
            (stereo.0 * Self::VOLUME_MAX_F32) as u32,
            (stereo.1 * Self::VOLUME_MAX_F32) as u32,
        )
    }

    pub fn mono(&self) -> u32 {
        match *self {
            Self::Mono(m) => m,
            Self::Stereo(l, r) => (l + r) / 2,
        }
    }

    pub fn stereo(&self) -> (u32, u32) {
        match *self {
            Self::Stereo(l, r) => (l, r),
            Self::Mono(m) => (m, m),
        }
    }
}

pub const AUDIO_SAMPLE_MIN: u32 = 64;
pub const AUDIO_SAMPLE_MAX: u32 = 65472;

pub const fn align_sample_count(sample_count: u32) -> u32 {
    // 63 = (1 in all bits less significant than 64)
    (sample_count + 63) & !63
}

#[derive(Clone, Copy, Debug)]
/// Same as [`sys::AudioFormat`], but with more trait implementation
pub enum AudioFormat {
    Mono,
    Stereo,
}

impl From<AudioFormat> for sys::AudioFormat {
    fn from(from: AudioFormat) -> Self {
        match from {
            AudioFormat::Mono => Self::Mono,
            AudioFormat::Stereo => Self::Stereo,
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// Same as [`sys::AudioOutputFrequency`], but with more trait implementation
pub enum AudioOutputFrequency {
    Khz48 = 48_000,
    Khz44_1 = 44_100,
    Khz32 = 32_000,
    Khz24 = 24_000,
    Khz22_05 = 22_050,
    Khz16 = 16_000,
    Khz12 = 12_000,
    Khz11_025 = 11_025,
    Khz8 = 8_000,
}

impl From<AudioOutputFrequency> for sys::AudioOutputFrequency {
    fn from(from: AudioOutputFrequency) -> Self {
        use AudioOutputFrequency as AudOutFreq;
        match from {
            AudOutFreq::Khz48 => Self::Khz48,
            AudOutFreq::Khz44_1 => Self::Khz44_1,
            AudOutFreq::Khz32 => Self::Khz32,
            AudOutFreq::Khz24 => Self::Khz24,
            AudOutFreq::Khz22_05 => Self::Khz22_05,
            AudOutFreq::Khz16 => Self::Khz16,
            AudOutFreq::Khz12 => Self::Khz12,
            AudOutFreq::Khz11_025 => Self::Khz11_025,
            AudOutFreq::Khz8 => Self::Khz8,
        }
    }
}

/// An audio channel
/// This type of channel uses the regular sceAudioOutput functions
// This seems to only support 16bit little-endian PCM at some frequency
pub struct AudioChannel {
    id: u32,
    sample_count: u32,
    format: AudioFormat,
}

impl AudioChannel {
    pub fn init() -> NativeResult<()> {
        native_result(unsafe {
            sys::sceAudioInit()
        })?;
        Ok(())
    }

    fn reserve_raw(
        channel: i32,
        sample_count: u32,
        format: AudioFormat,
    ) -> NativeResult<Self> {
        let id = native_result(unsafe {
            sys::sceAudioChReserve(channel, sample_count as i32, format.into())
        })?;
        Ok(Self {
            id,
            sample_count,
            format,
        })
    }
    pub fn reserve(
        channel: u8,
        sample_count: u32,
        format: AudioFormat,
    ) -> NativeResult<Self> {
        Self::reserve_raw(i32::from(channel), sample_count, format)
    }
    pub fn reserve_next(
        sample_count: u32,
        format: AudioFormat,
    ) -> NativeResult<Self> {
        Self::reserve_raw(sys::AUDIO_NEXT_CHANNEL, sample_count, format)
    }

    fn close_non_consuming(&self) {
        unsafe {
            sys::sceAudioChRelease(self.id as i32);
        };
    }

    pub fn output(
        &mut self,
        volume: &Volume,
        buffer: &[Sample],
    ) -> AudioResult<()> {
        if buffer.len() < self.sample_count() as usize {
            return Err(AudioError::SampleBufferTooSmall);
        }
        Ok(native_error(unsafe {
            sys::sceAudioOutput(
                self.id as i32,
                volume.mono() as i32,
                buffer.as_ptr() as *mut c_void,
            )
        })?)
    }

    pub fn output_blocking(
        &mut self,
        volume: &Volume,
        buffer: &[Sample],
    ) -> AudioResult<()> {
        if buffer.len() < self.sample_count() as usize {
            return Err(AudioError::SampleBufferTooSmall);
        }
        Ok(native_error(unsafe {
            sys::sceAudioOutputBlocking(
                self.id as i32,
                volume.mono() as i32,
                buffer.as_ptr() as *mut c_void,
            )
        })?)
    }

    pub fn output_stereo(
        &mut self,
        volume: &Volume,
        buffer: &[Sample],
    ) -> AudioResult<()> {
        let (l, r) = volume.stereo();
        if buffer.len() < self.sample_count() as usize {
            return Err(AudioError::SampleBufferTooSmall);
        }
        Ok(native_error(unsafe {
            sys::sceAudioOutputPanned(
                self.id as i32,
                r as i32,
                l as i32,
                buffer.as_ptr() as *mut c_void,
            )
        })?)
    }

    pub fn output_stereo_blocking(
        &mut self,
        volume: &Volume,
        buffer: &[Sample],
    ) -> AudioResult<()> {
        let (l, r) = volume.stereo();
        if buffer.len() < self.sample_count() as usize {
            return Err(AudioError::SampleBufferTooSmall);
        }
        Ok(native_error(unsafe {
            sys::sceAudioOutputPannedBlocking(
                self.id as i32,
                r as i32,
                l as i32,
                buffer.as_ptr() as *mut c_void,
            )
        })?)
    }

    pub fn unplayed_samples(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceAudioGetChannelRestLen(self.id as i32) })
            .map(|l| l as usize)
    }

    pub fn set_sample_count(&mut self, sample_count: u32) -> NativeResult<()> {
        let result = unsafe {
            sys::sceAudioSetChannelDataLen(self.id as i32, sample_count as i32)
        };
        native_result(result).map(|_| {
            self.sample_count = sample_count;
        })
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn sample_buffer_size(&self) -> usize {
        let mono_size = self.sample_count() as usize * size_of::<Sample>();
        match self.format() {
            AudioFormat::Mono => mono_size,
            AudioFormat::Stereo => mono_size * 2,
        }
    }

    pub fn set_format(&mut self, format: AudioFormat) -> NativeResult<()> {
        let result = unsafe {
            sys::sceAudioChangeChannelConfig(self.id as i32, format.into())
        };
        native_result(result).map(|_| {
            self.format = format;
        })
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn set_volume(
        &mut self,
        left_vol: u32,
        right_vol: u32,
    ) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceAudioChangeChannelVolume(
                self.id as i32,
                left_vol as i32,
                right_vol as i32,
            )
        })
    }

    /* TODO: this function is missing from the `psp` crate
    pub fn set_global_sampling_frequency(frequency: u32) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceAudioSetFrequency(frequency as i32)
        })
    }*/
}

impl Drop for AudioChannel {
    fn drop(&mut self) {
        self.close_non_consuming();
    }
}

/*impl core::future::Future for AudioChannel {
    type Output = ();
    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>
    ) -> core::task::Poll<Self::Output> {
        use core::task::Poll;
        let samples_left = crate::internal_unwrap(self.unplayed_samples());
        if samples_left == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}*/

// TODO: What is SRC? SouRCe?
/// An "SRC" audio channel
/// This type of channel uses the sceAudioSRC functions
// This seems to only support 16bit little-endian PCM
// at the frequencies defined by [`AudioOutputFrequency`]
pub struct AudioSrcChannel {
    frequency: AudioOutputFrequency,
    sample_count: u32,
    channels: u8,
}

impl AudioSrcChannel {
    pub fn reserve(
        sample_count: u32,
        frequency: AudioOutputFrequency,
        channels: u8,
    ) -> NativeResult<Self> {
        native_result(unsafe {
            sys::sceAudioSRCChReserve(
                sample_count as i32,
                frequency.into(),
                i32::from(channels),
            )
        })?;
        Ok(Self {
            frequency,
            sample_count,
            channels,
        })
    }
    fn close_non_consuming(&self) -> Result<(), AudioError> {
        native_result(unsafe { sys::sceAudioSRCChRelease() })?;
        Ok(())
    }

    #[must_use]
    pub fn channels(&self) -> u8 {
        self.channels
    }

    #[must_use]
    pub fn frequency(&self) -> AudioOutputFrequency {
        self.frequency
    }

    #[must_use]
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    #[must_use]
    pub fn sample_buffer_size(&self) -> usize {
        let mono_size = self.sample_count() as usize * size_of::<Sample>();
        mono_size * (self.channels() as usize) // or is it?
    }

    pub fn output_blocking(
        &mut self,
        volume: &Volume,
        buffer: &[Sample],
    ) -> NativeResult<()> {
        native_result(unsafe {
            sys::sceAudioSRCOutputBlocking(
                volume.mono() as i32,
                buffer.as_ptr() as *mut c_void,
            )
        })?;
        Ok(())
    }

    // TODO: Somehow find more functions?
}

impl Drop for AudioSrcChannel {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}

// TODO: somehow implement unplayed_samples
/*impl core::future::Future for AudioSrcChannel {
    type Output = ();
    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>
    ) -> core::task::Poll<Self::Output> {
        use core::task::Poll;
        let samples_left = crate::internal_unwrap(self.unplayed_samples());
        if samples_left == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}*/

/// Another type of audio channel
/// This type of channel uses the sceAudioOutput2 functions
// This seems to only support stereo 16bit LE PCM at 44100Hz
// Despite its name, this is not actually an updated version of [`AudioChannel`]
pub struct Audio2Channel {
    sample_count: u32,
}

impl Audio2Channel {
    pub fn reserve(sample_count: u32) -> NativeResult<Self> {
        native_result(unsafe {
            sys::sceAudioOutput2Reserve(sample_count as i32)
        })?;
        Ok(Self { sample_count })
    }
    fn close_non_consuming(&self) -> NativeResult<()> {
        native_result(unsafe { sys::sceAudioOutput2Release() })?;
        Ok(())
    }
    pub fn unplayed_samples(&self) -> NativeResult<u32> {
        native_result(unsafe { sys::sceAudioOutput2GetRestSample() })
    }
    pub fn set_sample_count(&mut self, sample_count: u32) -> NativeResult<()> {
        native_result(unsafe {
            sys::sceAudioOutput2ChangeLength(sample_count as i32)
        })?;
        self.sample_count = sample_count;
        Ok(())
    }
    #[must_use]
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }
    #[must_use]
    pub fn sample_buffer_size(&self) -> usize {
        #[allow(clippy::let_and_return)]
        let mono_size = self.sample_count() as usize * size_of::<Sample>();
        mono_size
    }
    pub fn output_blocking(
        &mut self,
        volume: &Volume,
        buffer: &[Sample],
    ) -> AudioResult<()> {
        if buffer.len() < self.sample_count() as usize {
            return Err(AudioError::SampleBufferTooSmall);
        }
        native_result(unsafe {
            sys::sceAudioOutput2OutputBlocking(
                volume.mono() as i32,
                buffer.as_ptr() as *mut c_void,
            )
        })?;
        Ok(())
    }

    // TODO: Somehow find more functions?
}

impl Drop for Audio2Channel {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}

// TODO: feature-gate this AND somehow implement async playback
/*impl core::future::Future for Audio2Channel {
    type Output = ();
    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>
    ) -> core::task::Poll<Self::Output> {
        use core::task::Poll;
        let samples_left = crate::internal_unwrap(self.unplayed_samples());
        if samples_left == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}*/

pub struct AudioInput;

impl AudioInput {
    pub fn init(unknown1: i32, gain: i32, unknown2: i32) -> NativeResult<Self> {
        native_result(unsafe {
            sys::sceAudioInputInit(unknown1, gain, unknown2)
        })?;
        Ok(Self)
    }

    pub fn init_ex(params: &mut sys::AudioInputParams) -> NativeResult<Self> {
        use core::ptr;
        native_result(unsafe {
            sys::sceAudioInputInitEx(ptr::from_mut(params))
        })?;
        Ok(Self)
    }

    pub fn input(
        &mut self,
        frequency: sys::AudioInputFrequency,
        buffer: &mut [Sample],
    ) {
        unsafe {
            sys::sceAudioInput(
                buffer.len() as i32,
                frequency,
                buffer.as_mut_ptr().cast(),
            )
        };
    }

    pub fn input_blocking(
        &mut self,
        frequency: sys::AudioInputFrequency,
        buffer: &mut [Sample],
    ) {
        unsafe {
            sys::sceAudioInputBlocking(
                buffer.len() as i32,
                frequency,
                buffer.as_mut_ptr().cast(),
            )
        };
    }

    // TODO: Somehow find more functions?
}

/*impl Drop for AudioInput {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}*/
