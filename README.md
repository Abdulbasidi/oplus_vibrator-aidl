# oplus_vibrator-aidl

Rust AIDL vibrator HAL for Qualcomm devices using the `awinic_haptic` driver.
It implements Android's `android.hardware.vibrator.IVibrator` HAL and exposes an
OPlus/ColorOS vibrator extension on the same binder.

The main service is registered as both:

```
android.hardware.vibrator.IVibrator/vibratorfeature
android.hardware.vibrator.IVibrator/default
```

`/vibratorfeature` matches the Xiaomi service that ships on ruby, while
`/default` is what COS17/AOSP looks up.

The OPlus extension is attached to that binder and is also registered as a
standalone service:

```
vendor.oplus.hardware.oplusvibrator.IOplusVibrator/default
oplus.hardware.vibrator_oplus_v1
```

The service name can be overridden with the `VIBRATOR_SERVICE_NAME` environment
variable. The standard `/vibratorfeature` and `/default` instances are always
registered as well.

## Features

### Android vibrator HAL

- Implements `android.hardware.vibrator.IVibrator` version 3.
- Registers both `IVibrator/vibratorfeature` and `IVibrator/default`.
- Uses `rsbinder` and does not depend on `libbinder_ndk`.
- Reports callback, amplitude-control, and external-control capabilities.
- Supports timed playback through `on()` and completion callbacks.
- Supports predefined effects through `perform()` and callback completion.
- Supports `off()`, amplitude changes, and external control.
- Generates completion callbacks with generation tracking so stale callbacks
  are not delivered after a newer command or a cancel.
- Implements composition by converting the requested primitive sequence into a
  single timed motor command.
- Provides primitive lists, primitive durations, effect lists, and fixed motor
  metadata for the framework.
- Reports no supported braking mechanisms.
- Returns an unsupported-operation error for vendor effects.

Supported predefined effects:

| Effect | Duration | Motor routing |
|---|---:|---|
| `CLICK` | 60 ms | click waveform |
| `DOUBLE_CLICK` | 130 ms | click waveform |
| `TICK` | 25 ms | light-tick waveform |
| `THUD` | 110 ms | click waveform |
| `POP` | 35 ms | click waveform |
| `HEAVY_CLICK` | 80 ms | heavy-click waveform |
| `TEXTURE_TICK` | 12 ms | light-tick waveform |
| `RINGTONE_1` through `RINGTONE_15` | 200 ms | ringtone waveform |

`EffectStrength::LIGHT`, `MEDIUM`, and `STRONG` scale the effect amplitude to
`0.6`, `0.85`, and `1.0`.

### Amplitude behavior

- Tracks the current amplitude in shared state.
- Rescales the running motor immediately when amplitude changes.
- Applies a 0.7-power perceptual curve for tap effects.
- Long continuous (CONT) playback, including incoming-call ring pulses, is
  capped at 60% of the tap drive and scales both `vmax` and `gain`. This keeps
  a 750 ms ring pulse from running at full `vmax=0x50`/`gain=0xB4`.
- The CONT cap can be tuned at runtime with
  `setprop debug.hoshikv.loop_scale 0.45` (`0.2`-`1.0`); the default is `0.60`.
- Supports amplitudes from `0.0` to `1.0`.

### External control

- Emulates audio-coupled ring haptics in userspace.
- Saves and restores the previous amplitude when external control is enabled or
  disabled.
- Uses a pulse/gap pattern that can be changed at runtime.
- Stops and cancels the ring loop when control is released or its generation is
  invalidated.
- Refuses to start the ring loop when no motor is attached.

Environment variables:

| Variable | Default | Purpose |
|---|---|---|
| `VIBRATOR_SERVICE_NAME` | `android.hardware.vibrator.IVibrator/vibratorfeature` | Extra primary IVibrator instance |
| `VIBRATOR_SYSFS` | auto-detect | Explicit awinic haptic sysfs directory |
| `HOSHIVK_RING_PATTERN` | `60,50,60,50,60,220` | External-control pulse/gap pattern in ms |
| `HOSHIVK_RING_AMP` | `1.6` | External-control amplitude, clamped to `0.0`-`2.0` |

### OPlus extension

- Exposes `vendor.oplus.hardware.oplusvibrator.IOplusVibrator` version 2.
- Attaches the extension to the main vibrator binder.
- Also registers standalone OPlus service names for framework compatibility.
- Implements `setVmax()`, `on()`, `off()`, `stop()`, `setAmplitude()`, and
  linear-motor on/off.
- Converts OPlus strength values from `0`-`2400` into the HAL amplitude range.
- Applies a soft response curve to OPlus strength values.
- Implements `performHe()` as a configurable burst pattern using the shared
  motor.
- Completes OPlus callbacks after stop, off, on, and burst operations.
- Provides compatibility stubs for calibration, gun controls, touch style,
  haptic-audio data, and power-on switch calls.

### Motor backend

- Prefers `/sys/bus/i2c/devices/0-005a` (ruby/MT6877: `aw869x_haptic@5a` on
  `&i2c0`), then `/sys/bus/i2c/drivers/awinic_haptic/0-005a`.
- Falls back to scanning every i2c bus for a client whose `name` is
  `awinic_haptic`.
- `VIBRATOR_SYSFS` can point at an explicit sysfs directory for debugging or
  for boards with a different bus.
- Probes `activate` with a real read instead of `stat()`; SELinux grants the HAL
  read/write/open but not getattr on generic sysfs.
- Writes `gain`, `duration`, `activate_mode`, `effect_id`, and `vmax` on demand.
- Attaches lazily and retries every 5 s while the motor is unavailable.
- Enables `auto_boost` at startup.
- Disables `ram_vbat_comp` at startup and retries after playback starts.
- Routes effects to light, normal, or heavy motor waveforms based on the effect
  base value.
- Supports tap playback, continuous playback, live amplitude updates, and stop.
- Writes the motor off when the `Motor` object is dropped.
- Keeps running when the motor is unavailable and reports the failure through
  Android logging.

### Logging and diagnostics

- Logs through `liblog` with the tag `hoshikv-vibrator-rs`.
- Provides info, debug, and error macros.
- Includes `tools/haptic-dbg.sh` for direct sysfs testing without binder.
- Includes an `ffprobe` helper binary for probing the haptic input device and
  force-feedback ioctl layout.

Haptic debug commands:

```bash
./tools/haptic-dbg.sh status
./tools/haptic-dbg.sh scan
./tools/haptic-dbg.sh map
./tools/haptic-dbg.sh tap 1.0 60
./tools/haptic-dbg.sh loop 2000 1.0
./tools/haptic-dbg.sh amplitude 1.0 60
./tools/haptic-dbg.sh play 1 60 0x50 0xB4
./tools/haptic-dbg.sh sweep
./tools/haptic-dbg.sh off
```

### Compatibility stubs

The HAL exposes the full AIDL surface expected by the framework, but some
advanced APIs are intentionally limited:

- `composePwle()` and `composePwleV2()` are accepted and return success without
  generating a waveform.
- Frequency-to-acceleration maps are empty.
- PWLE duration and composition limits return `-1`.
- Always-on methods exist, but always-on control is not advertised in the
  capability mask.
- Vendor effects return `EX_UNSUPPORTED_OPERATION`.
- The OPlus calibration, gun, and touch-style calls are no-ops or fixed values.

## Project structure

```
.
├── aidl/
│   ├── android/hardware/vibrator/      # IVibrator V3 interfaces and types
│   ├── android/os/PersistableBundle.aidl
│   └── vendor/
│       ├── aac/hardware/richtap/vibrator/
│       │   └── IRichtapCallback.aidl
│       └── oplus/hardware/oplusvibrator/
│           └── IOplusVibrator.aidl     # OPlus extension, version 2
├── src/
│   ├── main.rs                         # binder registration and service setup
│   ├── service.rs                      # IVibrator implementation and ring loop
│   ├── motor.rs                        # awinic_haptic sysfs backend
│   ├── oplus.rs                        # IOplusVibrator implementation
│   ├── log.rs                          # Android liblog bindings and macros
│   └── bin/ffprobe.rs                  # force-feedback ioctl probe
├── tools/
│   └── haptic-dbg.sh                   # sysfs debug and test tool
├── build.rs                            # AIDL code generation
├── Cargo.toml
├── Cargo.lock
├── .gitignore
└── README.md
```

## Build

Requirements:

- Rust toolchain with Cargo.
- Android NDK.
- `aarch64-linux-android` target support.

This repository does not include a `.cargo/config.toml`. Pass the NDK linker
through the environment:

```bash
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang \
cargo build --release --target aarch64-linux-android
```

Outputs:

```
target/aarch64-linux-android/release/vibrator-service
target/aarch64-linux-android/release/ffprobe
```

The AIDL binding for `IVibrator` is generated at version 3. The OPlus extension
is generated at version 2. Both use synchronous binder methods.

## Deployment notes

- The service is a 64-bit Android executable for Android 28 or newer.
- It needs read/write access to the `awinic_haptic` I2C sysfs directory.
- It needs permission to register binder services.
- SELinux policy must allow both the sysfs access and binder registration.
- `packaging/ruby/` contains the init script, VINTF fragment, fs_config and the
  `sysfs_vibrator` genfscon used by the Redmi Note 12 Pro 5G.
- Start the binary from the vendor init configuration used by the target ROM.
- The service keeps running without a motor, but vibration calls become no-ops.

## Compatibility

The motor backend defaults to the ruby/MT6877 layout: `aw869x_haptic@5a` on
`i2c0`, exposed as `/sys/bus/i2c/devices/0-005a`. It also scans
`/sys/bus/i2c/drivers/awinic_haptic` and then every i2c bus for a client whose
`name` is `awinic_haptic`. Different drivers or node layouts require changes in
`src/motor.rs` or `VIBRATOR_SYSFS`.

The AIDL interfaces are VINTF-stable. The OPlus extension is vendor-specific and
does not replace the framework's normal `IVibrator/default` service.

## License

The project is licensed under the Apache License 2.0. See `LICENSE`.

The Android vibrator AIDL definitions retain their original AOSP Apache-2.0
copyright headers.
