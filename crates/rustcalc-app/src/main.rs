#![forbid(unsafe_code)]
//! RustCalc desktop application entry point.

use eframe::egui;
use rustcalc_app::app::RustCalcApp;
use rustcalc_app::theme;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([380.0, 620.0])
            .with_min_inner_size([300.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        rustcalc_core::APP_NAME,
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(RustCalcApp::new()))
        }),
    )
}
