use std::path::PathBuf;

const AIDL: &str = "aidl";
const V3_HASH: &str = "2dad67b7e7f6d4c4ef71f9b0b08e9c0c6a5c3e9d6b2f8d1a4c3e5f60718293a4b";

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
    b2 = b2.version(2).hash("-1");

    b2.output(PathBuf::from("oplus_vibrator.rs"))
        .generate()
        .unwrap_or_else(|err| {
            eprintln!("{:?}", miette::Report::new(err));
            std::process::exit(1);
        });

    println!("cargo:rustc-link-lib=log");
}