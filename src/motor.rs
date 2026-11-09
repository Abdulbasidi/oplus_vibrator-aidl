





use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;


const HAPTIC_NAME: &str = "awinic_haptic";


/// ruby (MT6877) wires the AW869x haptic to i2c0 at address 0x5a.
const RUBY_HAPTIC_PATH: &str = "/sys/bus/i2c/devices/0-005a";


const VMAX_LO: u16 = 0x28;
const VMAX_HI: u16 = 0x50;

const GAIN_HI: u16 = 0xB4;

/// Long CONT pulses (incoming-call ringtone, alarms, always-on) accumulate far
/// more energy than a short tap. Keep them below the tap drive so a 750 ms
/// ring pulse no longer runs at full vmax/gain. Can be tuned at runtime with
/// `setprop debug.hoshikv.loop_scale 0.45` (clamped to 0.2..1.0).
const LOOP_DRIVE_SCALE: f32 = 0.60;
const LOOP_SCALE_PROP: &[u8] = b"debug.hoshikv.loop_scale\0";


const CONT_DRV1: u16 = 0x7F;
const CONT_DRV2: u16 = 0x36;
const _: () = assert!(CONT_DRV1 == 0x7F && CONT_DRV2 == 0x36);

pub struct Motor {
    dir: PathBuf,
    running_kind: u8,
    counter: i32,
}

/// HAL amplitude domain is 0.0..1.0, where 1.0 means full drive.
/// The stock awinic curves are roughly linear in register value, so use a
/// mild perceptual exponent to keep low levels usable without overdriving
/// the top end.
fn drive_level(amplitude: f32) -> f32 {
    amplitude.clamp(0.0, 1.0).powf(0.7)
}

fn loop_scale() -> f32 {
    extern "C" {
        fn __system_property_get(name: *const std::ffi::c_char, value: *mut std::ffi::c_char) -> i32;
    }

    // PROP_VALUE_MAX is 92 on Android; keep the full size so a long value
    // cannot overrun the buffer in __system_property_get.
    let mut buf = [0 as std::ffi::c_char; 92];
    let n = unsafe {
        __system_property_get(
            LOOP_SCALE_PROP.as_ptr() as *const std::ffi::c_char,
            buf.as_mut_ptr(),
        )
    };
    if n > 0 {
        let value = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) }.to_string_lossy();
        if let Ok(v) = value.trim().parse::<f32>() {
            return v.clamp(0.2, 1.0);
        }
    }
    LOOP_DRIVE_SCALE
}

/// Drive level for sustained CONT playback. Unlike a tap, the long pulse uses
/// a reduced level and also scales vmax instead of pinning it to VMAX_HI.
fn loop_drive_level(amplitude: f32) -> f32 {
    (drive_level(amplitude) * loop_scale()).clamp(0.0, 1.0)
}

fn has_haptic_name(dir: &Path) -> bool {
    fs::read_to_string(dir.join("name"))
        .map(|s| s.trim() == HAPTIC_NAME)
        .unwrap_or(false)
}


fn find_haptic_dir() -> io::Result<PathBuf> {
    // Debug/porting override. Do not stat() it: the HAL domain is not granted
    // getattr on generic sysfs, only read/write/open.
    if let Ok(path) = std::env::var("VIBRATOR_SYSFS") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    // ruby: the haptic node is aw869x_haptic@5a on &i2c0.
    for candidate in [RUBY_HAPTIC_PATH, "/sys/bus/i2c/drivers/awinic_haptic/0-005a"] {
        let p = PathBuf::from(candidate);
        if has_haptic_name(&p) {
            return Ok(p);
        }
    }

    // Fallback: any i2c bus exposing the awinic haptic driver.
    for base in ["/sys/bus/i2c/drivers/awinic_haptic", "/sys/bus/i2c/devices"] {
        let entries = match fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if has_haptic_name(&p) {
                return Ok(p);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "{} sysfs dir not found (looked at {} and all i2c buses)",
            HAPTIC_NAME, RUBY_HAPTIC_PATH
        ),
    ))
}


impl Motor {
    pub fn open() -> io::Result<Motor> {
        let dir = find_haptic_dir()?;

        // SELinux grants read/write/open but not getattr on generic sysfs,
        // so Path::exists() can fail on a perfectly usable node. Probe with
        // an actual read instead.
        fs::read_to_string(dir.join("activate")).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("awinic node {} is not readable: {}", dir.join("activate").display(), e),
            )
        })?;

        let m = Motor {
            dir,
            running_kind: 0,
            counter: 0,
        };

        let _ = m.write_sysfs("auto_boost", "1");
        let _ = m.write_sysfs("ram_vbat_comp", "0");
        crate::alogd!(
            "motor opened: {} auto_boost=1 ram_vbat_comp=0",
            m.dir.display()
        );
        Ok(m)
    }

    fn write_sysfs(&self, node: &str, v: &str) -> io::Result<()> {
        let path = self.dir.join(node);
        match fs::write(&path, v) {
            Ok(()) => Ok(()),
            Err(e) => {
                crate::aloge!("motor write {} = '{}' failed: {}", path.display(), v, e);
                Err(e)
            }
        }
    }



    fn force_vbat_comp_off(&self) {
        for _ in 0..20 {
            let _ = self.write_sysfs("ram_vbat_comp", "0");
            if let Ok(s) = fs::read_to_string(self.dir.join("ram_vbat_comp")) {
                if s.trim() == "0" || s.contains("ram_vbat_comp=0") {
                    crate::alogd!("ram_vbat_comp forced off");
                    return;
                }
            }
            thread::sleep(Duration::from_millis(2));
        }
        crate::aloge!("ram_vbat_comp still on after retries");
    }

    fn set_motor(&self, on: bool) -> io::Result<()> {
        self.write_sysfs("activate", if on { "1" } else { "0" })
    }






    pub fn play_tap(&mut self, effect_id: u16, amplitude: f32, _len_ms: u16) -> io::Result<i32> {
        let _ = self.set_motor(false);
        self.write_sysfs("activate_mode", "0")?;
        let t = drive_level(amplitude);
        let v = (VMAX_LO as f32 + (VMAX_HI - VMAX_LO) as f32 * t).round() as u16;
        let g = (1.0 + (GAIN_HI as f32 - 1.0) * t).round() as u16;
        let (v, g) = (v.min(VMAX_HI), g.min(GAIN_HI));
        self.write_sysfs("vmax", &v.to_string())?;
        self.write_sysfs("gain", &g.to_string())?;
        self.write_sysfs("effect_id", &effect_id.to_string())?;
        self.set_motor(true)?;
        self.force_vbat_comp_off();
        self.running_kind = 1;
        self.counter = self.counter.wrapping_add(1);
        crate::alogd!(
            "tap eid={} vmax=0x{:X} gain=0x{:X} amp={:.2} len={}ms",
            effect_id, v, g, amplitude, _len_ms
        );
        Ok(self.counter)
    }









    pub fn play_loop(&mut self, length_ms: u16, amplitude: f32) -> io::Result<i32> {
        let _ = self.set_motor(false);
        let t = loop_drive_level(amplitude);
        let g = (1.0 + (GAIN_HI as f32 - 1.0) * t).round() as u16;
        let g = g.min(GAIN_HI);
        let v = (VMAX_LO as f32 + (VMAX_HI - VMAX_LO) as f32 * t).round() as u16;
        let v = v.min(VMAX_HI);

        self.write_sysfs("activate_mode", "1")?;
        self.write_sysfs("vmax", &v.to_string())?;
        self.write_sysfs("gain", &g.to_string())?;



        self.write_sysfs("duration", &length_ms.to_string())?;
        self.set_motor(true)?;

        self.running_kind = 2;
        self.counter = self.counter.wrapping_add(1);
        crate::alogd!(
            "cont(CONT) vmax=0x{:X} gain=0x{:X} len={}ms (amp={:.2} t={:.2})",
            v, g, length_ms, amplitude, t
        );
        Ok(self.counter)
    }


    pub fn update(&mut self, _id: i32, amplitude: f32) -> io::Result<()> {
        if self.running_kind == 2 {
            let t = loop_drive_level(amplitude);
            let g = ((1.0 + (GAIN_HI as f32 - 1.0) * t).round() as u16).min(GAIN_HI);
            let v = ((VMAX_LO as f32 + (VMAX_HI - VMAX_LO) as f32 * t).round() as u16)
                .min(VMAX_HI);
            let _ = self.write_sysfs("vmax", &v.to_string());
            let _ = self.set_gain(g);
        } else if self.running_kind == 1 {
            let t = drive_level(amplitude);
            let g = ((1.0 + (GAIN_HI as f32 - 1.0) * t).round() as u16).min(GAIN_HI);
            let v = ((VMAX_LO as f32 + (VMAX_HI - VMAX_LO) as f32 * t).round() as u16)
                .min(VMAX_HI);
            let _ = self.write_sysfs("vmax", &v.to_string());
            let _ = self.set_gain(g);
        }
        Ok(())
    }

    fn set_gain(&self, v: u16) {
        let _ = self.write_sysfs("gain", &v.to_string());
    }

    pub fn remove(&mut self, _id: i32) {
        let _ = self.set_motor(false);
        self.running_kind = 0;
    }

    pub fn is_looping(&self) -> bool {
        self.running_kind == 2
    }
}

impl Drop for Motor {
    fn drop(&mut self) {
        let _ = self.set_motor(false);
    }
}
