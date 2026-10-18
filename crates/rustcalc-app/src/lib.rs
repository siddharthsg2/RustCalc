#![forbid(unsafe_code)]
//! RustCalc GUI shell.
//!
//! This crate renders the calculator and forwards input to `rustcalc-core`.
//! All calculator logic lives in `rustcalc_core`; this crate is a thin shell.

pub mod app;
pub mod icon;
mod icon_pixels;
pub mod theme;

/// Shared startup for every platform: apply the theme, then restore any
/// persisted history.
pub fn create_app(cc: &eframe::CreationContext<'_>) -> app::RustCalcApp {
    theme::apply(&cc.egui_ctx);
    let mut app = app::RustCalcApp::new();
    if let Some(text) = cc
        .storage
        .and_then(|storage| storage.get_string(app::HISTORY_STORAGE_KEY))
    {
        app.restore_history(&text);
    }
    app
}
