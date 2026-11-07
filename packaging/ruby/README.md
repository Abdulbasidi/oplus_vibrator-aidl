# ruby integration (Redmi Note 12 Pro 5G, MT6877)

This directory contains the vendor overlay for replacing the stock Xiaomi
vibrator service with the Rust AIDL HAL.

## What gets replaced

| Path | Action |
|---|---|
| `/vendor/bin/hw/vendor.xiaomi.hardware.vibratorfeature.service` | overwrite with the Rust binary |
| `/vendor/etc/init/vendor.xiaomi.hardware.vibratorfeature.service.rc` | overwrite (same service name, control-node chowns) |
| `/vendor/etc/vintf/manifest/vendor.xiaomi.hardware.vibratorfeature.service.xml` | overwrite (declare both instances) |
| `/vendor/etc/init/vendor.xiaomi.hardware.vibratorfeature@1.0-service.rc` | delete (dead HIDL script, no binary) |

`file_contexts` and `service_contexts` do not need changes:

- `vendor_file_contexts` already labels the active binary as
  `hal_vibrator_default_exec`.
- `vendor_service_contexts` already maps `IVibrator/vibratorfeature` and
  `plat_service_contexts` already maps `IVibrator/default` to
  `hal_vibrator_service`.

The old HIDL library (`vendor/lib64/vendor.xiaomi.hardware.vibratorfeature@1.0.so`)
and its dead hwservice context can stay; nothing loads them.

## SELinux addition

`sepolicy/sysfs_haptics.cil` labels the eight AW869x control attributes under
`/sys/devices/platform/11e00000.i2c0/i2c-0/0-005a/` as `sysfs_vibrator`:

```
activate, activate_mode, duration, effect_id, vmax, gain, auto_boost, ram_vbat_comp
```

The client directory itself stays generic `sysfs`, so path traversal uses the
existing `allow domain sysfs:dir search` rule. The existing policy then does
the rest:

- `init` already has `setattr` on `sysfs_vibrator` files, so the rc `chown`
  lines work.
- `hal_vibrator_default` already has read/write/getattr on `sysfs_vibrator`
  files.

Compile this together with the vendor policy, the same way
`mtk_hal_light_extra.cil` is included. Without the labels the nodes are generic
`sysfs`: init cannot `chown` them (no setattr) and the HAL cannot write them as
`system`.

## Build

The CI workflow produces `vibrator-service` for `aarch64-linux-android` and
packages it with the files above. Locally:

```bash
cargo build --release --target aarch64-linux-android
```

Then copy `target/aarch64-linux-android/release/vibrator-service` over
`vendor/bin/hw/vendor.xiaomi.hardware.vibratorfeature.service` and strip it.

## Verify on device

```bash
# Both instances must be present.
service list | grep android.hardware.vibrator

# The HAL must own the control nodes.
ls -l /sys/bus/i2c/devices/0-005a/{activate,activate_mode,duration,effect_id,vmax,gain,auto_boost,ram_vbat_comp}

# Check labels and denials.
ls -Z /sys/bus/i2c/devices/0-005a/activate
logcat -b all | grep -iE 'avc.*vibrator|hoshikv-vibrator'
```
