//! RustCalc Android entry point.
//!
//! This is the one documented exception to the workspace-wide
//! `#![forbid(unsafe_code)]` rule (see AGENTS.md): Android loads the app as a
//! native library and calls an exported `android_main` symbol, which Rust only
//! produces through `#[unsafe(no_mangle)]`.

#[cfg(target_os = "android")]
use std::time::Duration;

#[cfg(target_os = "android")]
use rustcalc_app::app::InsetsHandle;

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

    let poll_app = app.clone();
    let insets = InsetsHandle::new();

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
        Box::new(move |cc| {
            spawn_inset_poller(poll_app, insets.clone(), cc.egui_ctx.clone());
            Ok(Box::new(rustcalc_app::create_app_with_insets(
                cc,
                Some(insets),
            )))
        }),
    ) {
        log::error!("RustCalc failed to start: {err}");
        std::process::exit(1);
    }
}

/// The OS reports the rectangle that system UI (status bar, gesture bar)
/// leaves free, but winit does not surface it as an event. Poll it instead,
/// and repaint when it changes so the layout follows rotation.
#[cfg(target_os = "android")]
fn spawn_inset_poller(app: AndroidApp, insets: InsetsHandle, ctx: eframe::egui::Context) {
    let spawned = std::thread::Builder::new()
        .name("rustcalc-insets".to_owned())
        .spawn(move || {
            loop {
                let rect = app.content_rect();
                if insets.set_rect_px(rect.left, rect.top, rect.right, rect.bottom) {
                    ctx.request_repaint();
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    if let Err(err) = spawned {
        log::warn!("inset poller failed to start: {err}");
    }
}
