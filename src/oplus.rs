use std::sync::{Arc, Mutex};
use rsbinder::{Strong, BinderResult};
use crate::gen_oplus::vendor::oplus::hardware::oplusvibrator::IOplusVibrator::IOplusVibrator;
use crate::gen_oplus::vendor::aac::hardware::richtap::vibrator::IRichtapCallback::IRichtapCallback;
use crate::service::Inner;
use crate::{alogd, alogi};

// Safe starting point: report no RichTap / linear-motor features so the framework
// uses the plain IVibrator path. Switch individual keys to 1 (via
// HOSHIVK_OPLUS_FEATURES) to test the ColorOS linear-motor paths.
const DEFAULT_FEATURES: &str = "oplus.software.vibrator_richctap=0\n\
oplus.software.vibrator_lmvibrator=0\n\
oplus.software.vibrator_qcom_lmvibrator=0\n";

pub struct OplusVibrator {
    inner: Arc<Mutex<Inner>>,
}

impl OplusVibrator {
    pub fn new(inner: Arc<Mutex<Inner>>) -> Self {
        OplusVibrator { inner }
    }

    fn strength_to_amplitude(strength: i32) -> f32 {
        let s = (strength as f32).clamp(0.0, 2400.0);
        let t = (s / 1200.0).clamp(0.0, 2.0);
        let t = t.powf(0.8);
        t.clamp(0.0, 2.0)
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

    fn r#linearMotorVibratorOn(&self, duration: i32, strength: i32, _extra: bool) -> BinderResult<()> {
        let amp = Self::strength_to_amplitude(strength);
        alogi!("linearMotorVibratorOn dur={} strength={} amp={:.2}", duration, strength, amp);
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
            let len = (duration as u16).max(500);
            inner.upload(0xFFFF, len, false);
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
        let amp = (amplitude as f32).clamp(0.0, 2.0);
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
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
        let amp = Self::strength_to_amplitude(amplitude);
        alogd!("oplus on({}) -> amp={:.2}", amplitude, amp);
        if let Ok(mut inner) = self.inner.lock() {
            inner.amplitude = amp;
            inner.upload(0xFFFF, 2000, false);
        }
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#perform(&self, _effect_id: i32, _strength: i8, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<i32> {
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(0)
    }

    // Added to match the ROM framework's IOplusVibrator V2 (transactions 10 and 11).
    // Envelope / RTP waveforms are not supported by this motor backend: accept and ack.
    fn r#performEnvelope(&self, env_info: &[i32], _fast_flag: bool, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus performEnvelope len={} (unsupported, ignored)", env_info.len());
        if let Some(cb) = cb { let _ = cb.onCallback(0); }
        Ok(())
    }

    fn r#performRtp(&self, _fd: &rsbinder::ParcelFileDescriptor, cb: Option<&Strong<dyn IRichtapCallback>>) -> BinderResult<()> {
        alogd!("oplus performRtp (unsupported, ignored)");
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
            ((c as f32) / 255.0) * 1.8
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

    // The ROM framework reads this as UTF-8 text in java.util.Properties format
    // ("key=value" per line) and parses the values with Integer.parseInt
    // (missing key => -1). Keys it asks for: see DEFAULT_FEATURES.
    // Override without rebuilding: HOSHIVK_OPLUS_FEATURES="key=1,key2=0" (set via init `setenv`).
    fn r#getSupportFeatures(&self) -> BinderResult<Vec<u8>> {
        let text = match std::env::var("HOSHIVK_OPLUS_FEATURES") {
            Ok(s) if !s.trim().is_empty() => s
                .split(',')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
            _ => DEFAULT_FEATURES.to_string(),
        };
        alogd!("oplus getSupportFeatures -> {:?}", text);
        Ok(text.into_bytes())
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

    fn r#writeHapticAudio(&self, _data: &str) -> BinderResult<()> {
        alogd!("oplus writeHapticAudio len={}", _data.len());
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
