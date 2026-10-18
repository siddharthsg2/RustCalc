//! Visual theme for RustCalc.

use eframe::egui::{self, Color32, CornerRadius, Frame, Vec2};

use crate::icon_pixels;

/// Accent color (equals key, selection highlight).
pub const ACCENT: Color32 = Color32::from_rgb(
    icon_pixels::ACCENT[0],
    icon_pixels::ACCENT[1],
    icon_pixels::ACCENT[2],
);
/// Text color drawn on top of the accent color.
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(
    icon_pixels::ACCENT_TEXT[0],
    icon_pixels::ACCENT_TEXT[1],
    icon_pixels::ACCENT_TEXT[2],
);
/// Error text color.
pub const ERROR: Color32 = Color32::from_rgb(0xFF, 0x7A, 0x7A);

/// Background of digit keys (`0`–`9`, `.`).
const KEY_DIGIT_BG: Color32 = Color32::from_rgb(0x2B, 0x30, 0x3B);
/// Background of function keys (`%`, `CE`, `C`, `DEL`, `±`).
const KEY_FUNCTION_BG: Color32 = Color32::from_rgb(0x34, 0x3B, 0x47);
/// Background of operator keys (`÷`, `×`, `−`, `+`).
const KEY_OPERATOR_BG: Color32 = Color32::from_rgb(0x28, 0x33, 0x42);

/// Colors of one class of keypad button, including hover/active states.
#[derive(Debug, Clone, Copy)]
pub struct KeyPalette {
    /// Fill when idle.
    pub base: Color32,
    /// Fill while hovered.
    pub hover: Color32,
    /// Fill while pressed.
    pub active: Color32,
    /// Text color in all states.
    pub text: Color32,
}

impl KeyPalette {
    fn new(base: Color32, text: Color32) -> Self {
        Self {
            base,
            hover: base.linear_multiply(1.12),
            active: base.linear_multiply(0.92),
            text,
        }
    }

    /// Digit keys.
    pub fn digit(text: Color32) -> Self {
        Self::new(KEY_DIGIT_BG, text)
    }

    /// Function keys.
    pub fn function(text: Color32) -> Self {
        Self::new(KEY_FUNCTION_BG, text)
    }

    /// Operator keys: amber symbols on a dark blue-gray.
    pub fn operator() -> Self {
        Self::new(KEY_OPERATOR_BG, ACCENT)
    }

    /// The equals key: amber fill with dark text.
    pub fn equals() -> Self {
        Self::new(ACCENT, ACCENT_TEXT)
    }
}

/// The card around the expression and result display.
pub fn display_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(0x22, 0x27, 0x30))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(12)
}

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
