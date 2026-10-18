//! RustCalc Android entry point.
//!
//! This is the one documented exception to the workspace-wide
//! `#![forbid(unsafe_code)]` rule (see AGENTS.md): Android loads the app as a
//! native library and calls an exported `android_main` symbol, which Rust only
//! produces through `#[unsafe(no_mangle)]`.

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// Entry point invoked by the Android runtime.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag(rustcalc_core::APP_NAME),
    );

    let persistence_path = app.internal_data_path().map(|dir| dir.join("app.ron"));
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        android_app: Some(app),
        persistence_path,
        ..Default::default()
    };
    if let Err(err) = eframe::run_native(
        rustcalc_core::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(rustcalc_app::create_app(cc)))),
    ) {
        log::error!("RustCalc failed to start: {err}");
        std::process::exit(1);
    }
}
