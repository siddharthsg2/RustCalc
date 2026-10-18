#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! RustCalc desktop application entry point.

use eframe::egui;
use rustcalc_app::app::{HISTORY_STORAGE_KEY, RustCalcApp};
use rustcalc_app::{icon, theme};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([380.0, 620.0])
            .with_min_inner_size([300.0, 480.0])
            .with_icon(icon::window_icon()),
        ..Default::default()
    };
    eframe::run_native(
        rustcalc_core::APP_NAME,
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            let mut app = RustCalcApp::new();
            if let Some(text) = cc
                .storage
                .and_then(|storage| storage.get_string(HISTORY_STORAGE_KEY))
            {
                app.restore_history(&text);
            }
            Ok(Box::new(app))
        }),
    )
}
