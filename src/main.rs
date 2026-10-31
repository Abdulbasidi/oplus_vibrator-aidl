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


    let oplus = oplus::OplusVibrator::new(inner);
    let oplus_binder = gen_oplus::vendor::oplus::hardware::oplusvibrator::IOplusVibrator::BnOplusVibrator::new_binder(oplus);
    binder.as_binder().set_extension(&oplus_binder.as_binder())
        .unwrap_or_else(|e| eprintln!("failed to set oplus extension: {:?}", e));

    let instance = std::env::var("VIBRATOR_SERVICE_NAME").unwrap_or_else(|_| {
        "android.hardware.vibrator.IVibrator/default".to_string()
    });

    let host = Host::new().unwrap_or_else(|e| {
        eprintln!("failed to init binder host: {:?}", e);
        std::process::exit(1);
    });

    host.add_service(&instance, binder.as_binder())
        .unwrap_or_else(|e| {
            eprintln!("failed to add service {}: {:?}", instance, e);
            std::process::exit(1);
        });

    if instance != "android.hardware.vibrator.IVibrator/default" {
        host.add_service("android.hardware.vibrator.IVibrator/default", binder.as_binder())
            .unwrap_or_else(|e| {
                eprintln!("warn: failed to add /default: {:?}", e);
            });
    }

    // The ROM framework reaches IOplusVibrator only through IVibrator/default's
    // getExtension() (see OplusVibratorManager.getOplusVibrator), so no standalone
    // registration of the oplus interface is done.
    eprintln!("vibrator registered: {} + oplus extension", instance);

    host.serve().expect("serve");
}
