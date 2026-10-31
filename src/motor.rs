use std::fs;
use std::io;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const VMAX_LO: u16 = 0x28;
const VMAX_HI: u16 = 0x50;

const GAIN_HI: u16 = 0xB4;

const CONT_DRV1: u16 = 0x7F;
const CONT_DRV2: u16 = 0x36;
const _: () = assert!(CONT_DRV1 == 0x7F && CONT_DRV2 == 0x36);

// activate_mode values on ruby (awinic_haptic @ i2c 0-005a), checked on device:
//   0 = RAM effect (effect_id), weak at default vmax/gain
//   1 = continuous with duration, strongest
//   3 = continuous with duration, medium (not used here)
//   2 = does nothing
const MODE_RAM: &str = "0";
const MODE_CONT: &str = "1";

// The driver may not be bound yet when the HAL starts at boot.
const OPEN_RETRIES: u32 = 10;
const OPEN_RETRY_MS: u64 = 200;

pub struct Motor {
    dir: PathBuf,
    running_kind: u8,
    counter: i32,
}

/// Parses node contents like "0x42", "0", "effect_id =2" or "activate_mode = 0".
fn parse_node(s: &str) -> Option<u32> {
    let v = s.rsplit('=').next()?.trim();
    match v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16).ok(),
        None => v.parse().ok(),
    }
}

/// Finds the I2C client bound as awinic_haptic on any bus (ruby: "0-005a").
fn find_dir() -> io::Result<PathBuf> {
    for entry in fs::read_dir("/sys/bus/i2c/devices")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        // clients are "<bus>-<addr>"; skip adapters like "i2c-0"
        if name.starts_with("i2c-") || !name.contains('-') {
            continue;
        }
        let p = entry.path();
        if let Ok(s) = fs::read_to_string(p.join("name")) {
            if s.trim() == "awinic_haptic" {
                return Ok(p);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "awinic_haptic sysfs dir not found",
    ))
}

impl Motor {
    pub fn open() -> io::Result<Motor> {
        let mut dir = None;
        for attempt in 0..OPEN_RETRIES {
            match find_dir() {
                Ok(p) => {
                    dir = Some(p);
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    if attempt + 1 < OPEN_RETRIES {
                        thread::sleep(Duration::from_millis(OPEN_RETRY_MS));
                    }
                }
                Err(e) => return Err(e),
            }
        }
        let dir = dir.ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "awinic_haptic sysfs dir not found")
        })?;
        for f in ["gain", "duration", "activate", "activate_mode", "effect_id", "vmax"] {
            if !dir.join(f).exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("awinic node {} missing", f),
                ));
            }
        }
        let m = Motor {
            dir,
            running_kind: 0,
            counter: 0,
        };

        let _ = m.write_sysfs("auto_boost", "1");
        let _ = m.write_sysfs("ram_vbat_comp", "0");
        crate::alogd!("motor opened at {}: auto_boost=1 ram_vbat_comp=0", m.dir.display());
        Ok(m)
    }

    fn write_sysfs(&self, node: &str, v: &str) -> io::Result<()> {
        let path = self.dir.join(node);
        let mut last = None;
        for i in 0..3 {
            match fs::write(&path, v) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last = Some(e);
                    if i < 2 {
                        thread::sleep(Duration::from_millis(3));
                    }
                }
            }
        }
        let e = last.unwrap();
        crate::aloge!("motor write {} = '{}' failed: {}", path.display(), v, e);
        Err(e)
    }

    fn read_node(&self, node: &str) -> Option<u32> {
        fs::read_to_string(self.dir.join(node))
            .ok()
            .and_then(|s| parse_node(&s))
    }

    fn force_vbat_comp_off(&self) {
        for _ in 0..20 {
            let _ = self.write_sysfs("ram_vbat_comp", "0");
            if self.read_node("ram_vbat_comp") == Some(0) {
                crate::alogd!("ram_vbat_comp forced off");
                return;
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
        self.write_sysfs("activate_mode", MODE_RAM)?;
        let raw_t = amplitude.clamp(0.0, 2.0) / 2.0;
        let t = raw_t.sqrt();
        let v = (VMAX_LO as f32 + (VMAX_HI - VMAX_LO) as f32 * t) as u16;
        let g = (1.0 + (GAIN_HI as f32 - 1.0) * t) as u16;
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
        let raw_t = (amplitude * 2.0).clamp(0.0, 2.0) / 2.0;
        let t = raw_t * raw_t;
        let g = (1.0 + (GAIN_HI as f32 - 1.0) * t) as u16;
        let g = g.min(GAIN_HI);
        let v = VMAX_HI;

        // The driver clears duration after every playback, so all parameters
        // are rewritten before each start.
        self.write_sysfs("activate_mode", MODE_CONT)?;
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
            let raw_t = (amplitude * 2.0).clamp(0.0, 2.0) / 2.0;
            let t = raw_t * raw_t;
            let g = (1.0 + (GAIN_HI as f32 - 1.0) * t) as u16;
            let _ = self.set_gain(g.min(GAIN_HI));
        } else if self.running_kind == 1 {
            let raw_t = amplitude.clamp(0.0, 2.0) / 2.0;
            let t = raw_t.sqrt();
            let g = (1.0 + (GAIN_HI as f32 - 1.0) * t) as u16;
            let v = (VMAX_LO as f32 + (VMAX_HI - VMAX_LO) as f32 * t) as u16;
            let _ = self.write_sysfs("vmax", &v.min(VMAX_HI).to_string());
            let _ = self.set_gain(g.min(GAIN_HI));
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
