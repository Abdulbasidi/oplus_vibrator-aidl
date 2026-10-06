





use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;


const VMAX_LO: u16 = 0x28;
const VMAX_HI: u16 = 0x50;

const GAIN_HI: u16 = 0xB4;


const CONT_DRV1: u16 = 0x7F;
const CONT_DRV2: u16 = 0x36;
const _: () = assert!(CONT_DRV1 == 0x7F && CONT_DRV2 == 0x36);

pub struct Motor {
    dir: PathBuf,
    running_kind: u8,
    counter: i32,
}

impl Motor {
    pub fn open() -> io::Result<Motor> {
        let mut dir = None;
        for entry in fs::read_dir("/sys/bus/i2c/devices")? {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_name().to_string_lossy().starts_with("2-") {
                continue;
            }
            let p = entry.path();
            if let Ok(s) = fs::read_to_string(p.join("name")) {
                if s.trim() == "awinic_haptic" {
                    dir = Some(p);
                    break;
                }
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
        crate::alogd!("motor opened: auto_boost=1 ram_vbat_comp=0");
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
