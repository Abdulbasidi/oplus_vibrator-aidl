mod gen {
    rsbinder::include_aidl!("vibrator");
}
mod gen_oplus {
    rsbinder::include_aidl!("oplus_vibrator");
}
mod log;
mod motor;
mod oplus;
mod service;

use rsbinder::service::{kernel::Host, Registry};

fn main() {
    let service = service::VibratorService::new();
    let inner = service.inner.clone();

    let binder: rsbinder::Strong<dyn gen::android::hardware::vibrator::IVibrator::IVibrator> =
        gen::android::hardware::vibrator::IVibrator::BnVibrator::new_binder(service);


    let oplus = oplus::OplusVibrator::new(inner.clone());
    let oplus_binder = gen_oplus::vendor::oplus::hardware::oplusvibrator::IOplusVibrator::BnOplusVibrator::new_binder(oplus);
    binder.as_binder().set_extension(&oplus_binder.as_binder())
        .unwrap_or_else(|e| eprintln!("failed to set oplus extension: {:?}", e));

    // ruby ships a Xiaomi-branded AIDL service that registers
    // IVibrator/vibratorfeature. COS17/AOSP asks for IVibrator/default, so
    // publish the same binder under both names.
    let primary = std::env::var("VIBRATOR_SERVICE_NAME").unwrap_or_else(|_| {
        "android.hardware.vibrator.IVibrator/vibratorfeature".to_string()
    });

    let host = Host::new().unwrap_or_else(|e| {
        eprintln!("failed to init binder host: {:?}", e);
        std::process::exit(1);
    });

    let mut instances = vec![primary.clone()];
    for name in [
        "android.hardware.vibrator.IVibrator/vibratorfeature",
        "android.hardware.vibrator.IVibrator/default",
    ] {
        if !instances.iter().any(|n| n.as_str() == name) {
            instances.push(name.to_string());
        }
    }

    for name in &instances {
        host.add_service(name, binder.as_binder())
            .unwrap_or_else(|e| {
                eprintln!("warn: failed to add service {}: {:?}", name, e);
            });
    }


    let oplus2 = oplus::OplusVibrator::new(inner);
    let oplus_binder2 = gen_oplus::vendor::oplus::hardware::oplusvibrator::IOplusVibrator::BnOplusVibrator::new_binder(oplus2);
    for name in &[
        "vendor.oplus.hardware.oplusvibrator.IOplusVibrator/default",
        "oplus.hardware.vibrator_oplus_v1",
    ] {
        host.add_service(name, oplus_binder2.as_binder())
            .unwrap_or_else(|e| {
                eprintln!("warn: failed to add oplus service {}: {:?}", name, e);
            });
    }

    eprintln!(
        "vibrator registered: {:?} + extension + oplus standalone",
        instances
    );

    host.serve().expect("serve");
}
