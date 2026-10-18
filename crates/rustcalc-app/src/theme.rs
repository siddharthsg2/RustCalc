//! Visual theme for RustCalc.

use eframe::egui::{self, Color32, CornerRadius, Vec2};

/// Accent color (equals key, selection highlight).
pub const ACCENT: Color32 = Color32::from_rgb(0xF2, 0xA9, 0x3B);
/// Text color drawn on top of the accent color.
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(0x1B, 0x1E, 0x24);
/// Error text color.
pub const ERROR: Color32 = Color32::from_rgb(0xFF, 0x7A, 0x7A);
/// Function key background (`%`, `CE`, `C`, `DEL`, `±`).
pub const FUNCTION_BG: Color32 = Color32::from_rgb(0x34, 0x3B, 0x47);
/// Operator key background (`÷`, `×`, `−`, `+`).
pub const OPERATOR_BG: Color32 = Color32::from_rgb(0x2A, 0x34, 0x41);

/// Apply the RustCalc dark theme to a context.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(0x1B, 0x1E, 0x24);
    visuals.window_fill = visuals.panel_fill;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.4);

    let widgets = &mut visuals.widgets;
    widgets.inactive.bg_fill = Color32::from_rgb(0x2B, 0x30, 0x3B);
    widgets.hovered.bg_fill = Color32::from_rgb(0x36, 0x3D, 0x4A);
    widgets.active.bg_fill = Color32::from_rgb(0x42, 0x4A, 0x5A);
    for widget in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
    ] {
        widget.corner_radius = CornerRadius::same(10);
    }

    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 12.0);
    });
}
