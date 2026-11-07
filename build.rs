use std::path::PathBuf;

const AIDL: &str = "aidl";
// Frozen AIDL v3 hash from AOSP: aidl_api/android.hardware.vibrator/3/.hash
// (verified against android-16.0.0_r1). AIDL freezes one hash per package
// version, so IVibrator and IVibratorCallback share it.
//
// The stock ruby service is older: its strings contain eeab78b6... which is
// the v1 package hash, plus e5c4e957... for Xiaomi's IVibratorExt. Neither
// matters for this HAL: we publish v3 and the framework negotiates down from
// its own version.
const V3_HASH: &str = "720a16b521507c378f14c516749ae178a60dfc44";

fn main() {
    let mut b = rsbinder_aidl::Builder::new();
    b = b.set_async_support(false);
    b = b.include_dir(PathBuf::from(AIDL).join("android/hardware/vibrator"));

    let vib = "android/hardware/vibrator";
    let base = PathBuf::from(AIDL).join(vib);


    b = b.source(base.join("IVibratorCallback.aidl"));
    b = b.version(3).hash(V3_HASH);

    b = b.source(base.join("IVibrator.aidl"));
    b = b.version(3).hash(V3_HASH);


    for f in [
        "ActivePwle",
        "Braking",
        "BrakingPwle",
        "CompositeEffect",
        "CompositePrimitive",
        "CompositePwleV2",
        "Effect",
        "EffectStrength",
        "FrequencyAccelerationMapEntry",
        "PrimitivePwle",
        "PwleV2Primitive",
        "VendorEffect",
    ] {
        b = b.source(base.join(format!("{f}.aidl")));
    }

    b.output(PathBuf::from("vibrator.rs"))
        .generate()
        .unwrap_or_else(|err| {
            eprintln!("{:?}", miette::Report::new(err));
            std::process::exit(1);
        });


    let mut b2 = rsbinder_aidl::Builder::new();
    b2 = b2.set_async_support(false);
    b2 = b2.include_dir(PathBuf::from(AIDL).join("vendor/oplus/hardware/oplusvibrator"));
    b2 = b2.include_dir(PathBuf::from(AIDL).join("vendor/aac/hardware/richtap/vibrator"));


    b2 = b2.source(PathBuf::from(AIDL)
        .join("vendor/aac/hardware/richtap/vibrator/IRichtapCallback.aidl"));

    b2 = b2.source(PathBuf::from(AIDL)
        .join("vendor/oplus/hardware/oplusvibrator/IOplusVibrator.aidl"));
    // Real COS17 interface from /system/framework/oplus-services.jar:
    // IOplusVibrator.VERSION = 2 and HASH = b103e6d8...
    // Its method table includes performEnvelope/performRtp and orders
    // getSupportFeatures at transaction 21. Our previous unversioned AIDL
    // put it at 20, so system_server read an empty reply and died with
    // ParcelFormatException("Not enough data").
    b2 = b2.version(2).hash("b103e6d812aa472b9c5b7deefd7986fa729ce1a8");

    b2.output(PathBuf::from("oplus_vibrator.rs"))
        .generate()
        .unwrap_or_else(|err| {
            eprintln!("{:?}", miette::Report::new(err));
            std::process::exit(1);
        });

    println!("cargo:rustc-link-lib=log");
}