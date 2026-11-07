use std::sync::{Arc, Mutex};
use rsbinder::{Strong, BinderResult, ParcelFileDescriptor};
use crate::gen_oplus::vendor::oplus::hardware::oplusvibrator::IOplusVibrator::IOplusVibrator;
use crate::gen_oplus::vendor::aac::hardware::richtap::vibrator::IRichtapCallback::IRichtapCallback;
use crate::service::Inner;
use crate::{alogd, alogi};

pub struct OplusVibrator {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Clone, Copy, Debug)]
enum OplusEffect {
    Ram(u16),
    Cont(u16),
}

/// COS17 sends waveform ids from its own bank. Translate the ids we actually
/// observe into the stock awinic RAM bank, matching the intended duration.
fn oplus_effect(waveform_id: i32) -> OplusEffect {
    match waveform_id {
        1 => OplusEffect::Ram(3),   // short tick (~35 ms)
        2 => OplusEffect::Ram(0),   // click (~45 ms)
        7 => OplusEffect::Ram(3),   // touch tick (~30 ms)
        111 => OplusEffect::Ram(3), // IME key press (~36 ms)
        310 => OplusEffect::Ram(4), // virtual key (~80 ms)
        315 => OplusEffect::Cont(150), // volume tick (~155 ms)
        id if (0..=10).contains(&id) => OplusEffect::Ram(id as u16),
        _ => OplusEffect::Ram(3),
    }
}

impl OplusVibrator {
    pub fn new(inner: Arc<Mutex<Inner>>) -> Self {
        OplusVibrator { inner }
    }

    /// OPlus strength domain is 0..2400, where 2400 is the slider maximum.
    fn strength_to_amplitude(strength: i32) -> f32 {
        let s = (strength as f32).clamp(0.0, 2400.0);
        (s / 2400.0).clamp(0.0, 1.0)
    }

    /// Raw 0..255 amplitude used by the older OPlus on()/setAmplitude() calls.
    fn raw_amplitude(amplitude: i32) -> f32 {
        let a = (amplitude as f32).clamp(0.0, 255.0);
        (a / 255.0).clamp(0.0, 1.0)
    }
}

impl IOplusVibrator for OplusVibrator {
    fn r#setVmax(&self, strength: i32) -> BinderResult<()> {
        let amp = Self::strength_to_amplitude(strength);
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
        }
        alogi!("setVmax({}) -> amp={:.2}", strength, amp);
        Ok(())
    }

    fn r#linearMotorVibratorOn(&self, waveform_id: i32, strength: i32, is_rtp: bool) -> BinderResult<()> {
        let amp = Self::strength_to_amplitude(strength);
        let effect = oplus_effect(waveform_id);
        alogi!(
            "linearMotorVibratorOn waveformId={} strength={} rtp={} amp={:.2} -> {:?}",
            waveform_id,
            strength,
            is_rtp,
            amp,
            effect
        );
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
            match effect {
                OplusEffect::Ram(eid) => {
                    inner.play_ram(eid, amp);
                }
                OplusEffect::Cont(ms) => {
                    inner.play_cont(ms, amp);
                }
            }
        }
        Ok(())
    }

    fn r#linearMotorVibratorOff(&self) -> BinderResult<()> {
        alogd!("linearMotorVibratorOff");
        if let Ok(mut inner) = self.inner.lock() {
            inner.stop_effect();
        }
        Ok(())
    }

    fn r#init(&self, _callback: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus init");
        Ok(())
    }

    fn r#setDynamicScale(&self, _scale: i32, _cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        Ok(())
    }

    fn r#setF0(&self, _f0: i32, _cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        Ok(())
    }

    fn r#stop(&self, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus stop");
        if let Ok(mut inner) = self.inner.lock() {
            inner.stop_effect();
        }
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#setAmplitude(&self, amplitude: i32, _cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        let amp = Self::raw_amplitude(amplitude);
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
            inner.rescale_current();
        }
        alogd!("oplus setAmplitude({}) -> amp={:.2}", amplitude, amp);
        Ok(())
    }

    fn r#performHeParam(&self, a: i32, b: i32, c: i32, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {

        alogd!("oplus performHeParam(a={}, b={}, c={})", a, b, c);
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#off(&self, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus off");
        if let Ok(mut inner) = self.inner.lock() {
            inner.stop_effect();
        }
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#on(&self, amplitude: i32, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        let amp = Self::raw_amplitude(amplitude);
        alogd!("oplus on({}) -> amp={:.2}", amplitude, amp);
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
            inner.play_cont(2000, amp);
        }
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#perform(&self, _effect_id: i32, _strength: i8, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<i32> {
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(0)
    }

    fn r#performEnvelope(&self, env_info: &[i32], fast_flag: bool, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus performEnvelope len={} fast={}", env_info.len(), fast_flag);
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#performRtp(&self, _hdl: &ParcelFileDescriptor, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus performRtp");
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#performHe(&self, a: i32, b: i32, c: i32, d: i32, e: &[i32], cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        let pattern: Vec<i32> = e.iter().take(24).copied().collect();
        alogd!("oplus performHe(looper={} interval={} amp={} freq={} pattern_len={} pattern={:?})",
            a, b, c, d, e.len(), pattern);

        let bursts = a.clamp(1, 16);
        let gap = b.clamp(0, 500) as u64;
        let amp = if (0..=255).contains(&c) {
            Self::raw_amplitude(c)
        } else {
            1.0
        };
        for i in 0..bursts {
            if let Ok(mut inner) = self.inner.lock() {
                inner.amplitude = amp;
                inner.ring_pulse(50, amp);
            }
            if i + 1 < bursts {
                std::thread::sleep(std::time::Duration::from_millis(gap.max(30) + 50));
            }
        }
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#setHapticParam(&self, _params: &[i32], _length: i32, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus setHapticParam len={}", _length);
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#startCalibrate(&self) -> BinderResult<()> {
        Ok(())
    }

    fn r#getCalibrateResults(&self) -> BinderResult<Vec<i32>> {
        Ok(vec![0, 0])
    }

    fn r#getStatus(&self) -> BinderResult<i32> {
        Ok(1)
    }

    fn r#setMotorOldTest(&self, _test: i32) -> BinderResult<()> {
        Ok(())
    }

    fn r#getSupportFeatures(&self) -> BinderResult<Vec<u8>> {
        // java.util.Properties text expected by VibratorFeatureLoader.
        // Empty means "no OPlus/RichTap/Qualcomm-LM extensions": system_server
        // then drives us through the plain android.hardware.vibrator AIDL.
        Ok(Vec::new())
    }

    fn r#writeGunType(&self, _type: i32) -> BinderResult<()> {
        Ok(())
    }

    fn r#writeGunMode(&self, _mode: i32) -> BinderResult<()> {
        Ok(())
    }

    fn r#writeBulletNum(&self, _num: i32) -> BinderResult<()> {
        Ok(())
    }

    fn r#writeHapticAudio(&self, buffer: &str) -> BinderResult<()> {
        alogd!("oplus writeHapticAudio len={}", buffer.len());
        Ok(())
    }

    fn r#setVibratorTouchStyle(&self, _style: i32) -> BinderResult<()> {
        Ok(())
    }

    fn r#getVibratorTouchStyle(&self) -> BinderResult<i32> {
        Ok(0)
    }

    fn r#setPowerOnVibratorSwitch(&self, _switch: &str) -> BinderResult<()> {
        Ok(())
    }
}

impl rsbinder::Interface for OplusVibrator {}
