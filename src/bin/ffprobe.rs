use std::io;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::fs::File;
use core::ffi::{c_int, c_ulong, c_void};

const _IOC_WRITE: u32 = 1;
const _IOC_READ: u32 = 2;
const _IOC_TYPESHIFT: u32 = 8;
const _IOC_SIZESHIFT: u32 = 16;

const fn ioc(dir: u32, typ: u8, nr: u8, size: usize) -> u64 {
    let d = (dir as u64) << 30;
    let t = (typ as u64) << _IOC_TYPESHIFT;
    let s = (size as u64) << _IOC_SIZESHIFT;
    d | t | s | (nr as u64)
}

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FfEff {
    r#type: u16,
    id: i16,
    direction: u16,
    trigger: [u16; 2],
    replay: [u16; 2],
    data: [u8; 40],
}

fn main() {
    let mut opened = None;
    for idx in 0..64u32 {
        let p = format!("/dev/input/event{}", idx);
        if let Ok(f) = File::open(&p) {
            if let Ok(s) = std::fs::read_to_string(format!(
                "/sys/class/input/event{}/device/device/name",
                idx
            )) {
                if s.trim() == "awinic_haptic" {
                    opened = Some(f);
                    println!("device: {}", p);
                    break;
                }
            }
        }
    }
    let Some(dev) = opened else {
        println!("device not found");
        return;
    };
    let fd = dev.as_raw_fd();


    for size in [28usize, 32, 40, 48, 56, 64] {
        for typ in [0x50u16, 0x51, 0x52] {
            let mut ef = FfEff {
                r#type: typ,
                id: -1,
                direction: 0,
                trigger: [0, 0],
                replay: [40, 0],
                data: [0u8; 40],
            };


            ef.data[4] = 0x40; ef.data[5] = 0x40;
            ef.data[0] = 1; ef.data[1] = 0;
            let cmd = ioc(_IOC_WRITE, b'E', 0x80, size);
            let ret = unsafe {
                ioctl(fd, cmd, &mut ef as *mut FfEff as *mut c_void)
            };
            let err = io::Error::last_os_error();
            let errno = err.raw_os_error().unwrap_or(0);
            let tag = if ret < 0 {
                match errno {
                    14 => "EFAULT",
                    22 => "EINVAL",
                    25 => "ENOTTY",
                    9 => "EBADF",
                    _ => "ERR",
                }
            } else {
                "OK"
            };
            println!(
                "size={:2} type=0x{:02X} -> ret={} errno={} id={} ({})",
                size, typ, ret, errno, ef.id, tag
            );

            if ret == 0 {
                let rid = ef.id as i32;
                unsafe {
                    ioctl(fd, ioc(_IOC_WRITE, b'E', 0x81, mem::size_of::<i32>()), &rid as *const i32 as *mut c_void);
                }
            }
        }
        println!("");
    }
}