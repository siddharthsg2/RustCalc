//! The egui application shell.
//!
//! Renders the calculator and forwards input to `rustcalc_core`.

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use eframe::egui;
use egui::{Align, Button, Color32, Key, Layout, RichText};
use rustcalc_core::{Calculator, Op, deserialize_history, serialize_history};

use crate::theme;

/// Storage key under which the history is persisted.
pub const HISTORY_STORAGE_KEY: &str = "history";

/// Everything the UI can trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Digit(u8),
    Decimal,
    Op(Op),
    Equals,
    Percent,
    ToggleSign,
    Clear,
    ClearEntry,
    Backspace,
    RecallHistory(usize),
    ClearHistory,
    ToggleHistory,
}

/// Visual class of a keypad button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyKind {
    Digit,
    Function,
    Operator,
    Equals,
}

impl KeyKind {
    fn palette(self, text: Color32) -> theme::KeyPalette {
        match self {
            Self::Digit => theme::KeyPalette::digit(text),
            Self::Function => theme::KeyPalette::function(text),
            Self::Operator => theme::KeyPalette::operator(),
            Self::Equals => theme::KeyPalette::equals(),
        }
    }
}

/// One keypad button.
#[derive(Debug, Clone, Copy)]
struct KeyDef {
    label: &'static str,
    action: Action,
    kind: KeyKind,
}

fn key(label: &'static str, action: Action, kind: KeyKind) -> KeyDef {
    KeyDef {
        label,
        action,
        kind,
    }
}

/// The RustCalc application: calculator state plus UI preferences.
#[derive(Debug)]
pub struct RustCalcApp {
    calculator: Calculator,
    show_history: bool,
    insets: Option<InsetsHandle>,
}

impl Default for RustCalcApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RustCalcApp {
    /// Create an app showing a fresh calculator with the history panel closed.
    pub fn new() -> Self {
        Self {
            calculator: Calculator::new(),
            show_history: false,
            insets: None,
        }
    }

    /// Track the system-UI-free area reported by the platform.
    pub fn set_insets(&mut self, handle: InsetsHandle) {
        self.insets = Some(handle);
    }

    /// Replace the history from previously saved text.
    pub fn restore_history(&mut self, text: &str) {
        self.calculator.import_history(deserialize_history(text));
    }

    /// The serialized history, as written by [`eframe::App::save`].
    pub fn history_text(&self) -> String {
        serialize_history(self.calculator.history())
    }

    /// The main display text (exposed for tests).
    pub fn display(&self) -> String {
        self.calculator.display()
    }

    /// The calculator core (exposed for tests).
    pub fn calculator(&self) -> &Calculator {
        &self.calculator
    }

    /// Render the UI and apply all interactions triggered this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        for action in Self::keyboard_actions(ui.ctx()) {
            self.apply(action);
        }

        let mut actions = Vec::new();
        let margin = self.inset_margin(ui.ctx());
        egui::Frame::NONE.inner_margin(margin).show(ui, |ui| {
            ui.vertical(|ui| {
                self.top_bar(ui, &mut actions);
                self.history_section(ui, &mut actions);
                let (button_size, keypad_height) = keypad_metrics(ui);
                let display_height = ui.available_height() - keypad_height;
                self.display_section(ui, display_height);
                Self::keypad(ui, &mut actions, button_size);
            });
        });

        if !actions.is_empty() {
            ui.ctx().request_repaint();
        }
        for action in actions {
            self.apply(action);
        }
    }

    /// Margin keeping the layout clear of system UI (status bar, gesture bar).
    fn inset_margin(&self, ctx: &egui::Context) -> egui::Margin {
        match &self.insets {
            Some(handle) => handle.resolve(ctx).to_margin(),
            None => egui::Margin::ZERO,
        }
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Digit(digit) => self.calculator.input_digit(digit),
            Action::Decimal => self.calculator.input_decimal(),
            Action::Op(op) => self.calculator.set_op(op),
            Action::Equals => self.calculator.equals(),
            Action::Percent => self.calculator.percent(),
            Action::ToggleSign => self.calculator.toggle_sign(),
            Action::Clear => self.calculator.clear(),
            Action::ClearEntry => self.calculator.clear_entry(),
            Action::Backspace => self.calculator.backspace(),
            Action::RecallHistory(index) => {
                self.calculator.recall_history(index);
            }
            Action::ClearHistory => self.calculator.clear_history(),
            Action::ToggleHistory => self.show_history = !self.show_history,
        }
    }

    /// Actions triggered by keyboard input this frame.
    ///
    /// Printable characters (digits, operators, decimal point) are taken from
    /// [`egui::Event::Text`], which is what a real keyboard produces and is
    /// layout-aware. Reading them from key presses instead would double-fire
    /// for shifted keys (e.g. `Shift+8` types `*` but still reports `Num8`).
    fn keyboard_actions(ctx: &egui::Context) -> Vec<Action> {
        ctx.input(|input| {
            let mut actions = Vec::new();

            for event in &input.events {
                let egui::Event::Text(text) = event else {
                    continue;
                };
                let mut chars = text.chars();
                let Some(ch) = chars.next() else { continue };
                if chars.next().is_some() {
                    continue;
                }
                if let Some(digit) = ch.to_digit(10) {
                    actions.push(Action::Digit(digit as u8));
                    continue;
                }
                match ch {
                    '.' | ',' => actions.push(Action::Decimal),
                    '+' => actions.push(Action::Op(Op::Add)),
                    '-' => actions.push(Action::Op(Op::Subtract)),
                    '*' | '×' => actions.push(Action::Op(Op::Multiply)),
                    '/' | '÷' => actions.push(Action::Op(Op::Divide)),
                    '%' => actions.push(Action::Percent),
                    _ => {}
                }
            }

            if input.key_pressed(Key::Enter) || input.key_pressed(Key::Equals) {
                actions.push(Action::Equals);
            }
            if input.key_pressed(Key::Backspace) {
                actions.push(Action::Backspace);
            }
            if input.key_pressed(Key::Delete) {
                actions.push(Action::ClearEntry);
            }
            if input.key_pressed(Key::Escape) {
                actions.push(Action::Clear);
            }

            actions
        })
    }

    fn top_bar(&self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.horizontal(|ui| {
            let label = if self.show_history {
                "Hide history"
            } else {
                "History"
            };
            if ui.button(label).clicked() {
                actions.push(Action::ToggleHistory);
            }
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                if self.show_history
                    && !self.calculator.history().is_empty()
                    && ui.button("Clear history").clicked()
                {
                    actions.push(Action::ClearHistory);
                }
            });
        });
    }

    fn history_section(&self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        if !self.show_history {
            return;
        }
        let history = self.calculator.history();
        if history.is_empty() {
            ui.label(RichText::new("No calculations yet").weak().italics());
            return;
        }
        egui::ScrollArea::vertical()
            .max_height(160.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (index, entry) in history.iter().enumerate().rev() {
                    let text = format!("{} = {}", entry.expression, entry.result);
                    let button = Button::selectable(false, RichText::new(text))
                        .min_size(egui::Vec2::new(ui.available_width(), 26.0));
                    if ui.add(button).clicked() {
                        actions.push(Action::RecallHistory(index));
                    }
                }
            });
    }

    /// The display card. `min_outer_height` lets tall screens give the card
    /// the space the keypad does not need.
    fn display_section(&self, ui: &mut egui::Ui, min_outer_height: f32) {
        let frame = theme::display_frame();
        let vertical_margin =
            f32::from(frame.inner_margin.top) + f32::from(frame.inner_margin.bottom);
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height((min_outer_height - vertical_margin).max(0.0));
            ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                let text = self.calculator.display();
                let color = if self.calculator.error().is_some() {
                    theme::ERROR
                } else {
                    ui.visuals().strong_text_color()
                };
                let size = fit_display_size(ui, &text, ui.available_width(), color);
                ui.label(RichText::new(&text).size(size).color(color).strong());

                ui.label(
                    RichText::new(self.calculator.expression())
                        .weak()
                        .size(15.0),
                );
            });
        });
    }

    fn keypad(ui: &mut egui::Ui, actions: &mut Vec<Action>, button_size: egui::Vec2) {
        let rows: [[KeyDef; 4]; 5] = [
            [
                key("%", Action::Percent, KeyKind::Function),
                key("CE", Action::ClearEntry, KeyKind::Function),
                key("C", Action::Clear, KeyKind::Function),
                key("DEL", Action::Backspace, KeyKind::Function),
            ],
            [
                key("7", Action::Digit(7), KeyKind::Digit),
                key("8", Action::Digit(8), KeyKind::Digit),
                key("9", Action::Digit(9), KeyKind::Digit),
                key("÷", Action::Op(Op::Divide), KeyKind::Operator),
            ],
            [
                key("4", Action::Digit(4), KeyKind::Digit),
                key("5", Action::Digit(5), KeyKind::Digit),
                key("6", Action::Digit(6), KeyKind::Digit),
                key("×", Action::Op(Op::Multiply), KeyKind::Operator),
            ],
            [
                key("1", Action::Digit(1), KeyKind::Digit),
                key("2", Action::Digit(2), KeyKind::Digit),
                key("3", Action::Digit(3), KeyKind::Digit),
                key("−", Action::Op(Op::Subtract), KeyKind::Operator),
            ],
            [
                key("±", Action::ToggleSign, KeyKind::Function),
                key("0", Action::Digit(0), KeyKind::Digit),
                key(".", Action::Decimal, KeyKind::Digit),
                key("+", Action::Op(Op::Add), KeyKind::Operator),
            ],
        ];

        for row in &rows {
            ui.horizontal(|ui| {
                for def in row {
                    if key_button(ui, def, button_size).clicked() {
                        actions.push(def.action);
                    }
                }
            });
        }

        let equals = key("=", Action::Equals, KeyKind::Equals);
        let equals_size = egui::vec2(ui.available_width(), button_size.y);
        if key_button(ui, &equals, equals_size).clicked() {
            actions.push(equals.action);
        }
    }
}

impl eframe::App for RustCalcApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| self.ui(ui));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(HISTORY_STORAGE_KEY, self.history_text());
        storage.flush();
    }
}

fn key_button(ui: &mut egui::Ui, def: &KeyDef, size: egui::Vec2) -> egui::Response {
    let palette = def.kind.palette(ui.visuals().text_color());
    ui.scope(|ui| {
        let widgets = &mut ui.style_mut().visuals.widgets;
        for (visuals, fill) in [
            (&mut widgets.inactive, palette.base),
            (&mut widgets.hovered, palette.hover),
            (&mut widgets.active, palette.active),
        ] {
            visuals.weak_bg_fill = fill;
            visuals.fg_stroke.color = palette.text;
        }
        ui.add(
            Button::new(RichText::new(def.label).size((size.y * 0.33).clamp(16.0, 30.0)))
                .min_size(size)
                .corner_radius(10),
        )
    })
    .inner
}

/// The largest font size (step 2, capped) at which `text` fits `available`.
fn fit_display_size(ui: &egui::Ui, text: &str, available: f32, color: Color32) -> f32 {
    const MAX: f32 = 44.0;
    const MIN: f32 = 16.0;
    let mut size = MAX;
    while size > MIN {
        let galley =
            ui.painter()
                .layout_no_wrap(text.to_owned(), egui::FontId::proportional(size), color);
        if galley.rect.width() <= available {
            break;
        }
        size -= 2.0;
    }
    size
}

/// Height always reserved for the display card, in points.
const DISPLAY_MIN_HEIGHT: f32 = 100.0;

/// Button size and total keypad height (including the equals row) for the
/// currently available space.
///
/// Buttons stretch to fill tall screens up to square, while always leaving
/// [`DISPLAY_MIN_HEIGHT`] for the display card.
fn keypad_metrics(ui: &egui::Ui) -> (egui::Vec2, f32) {
    let spacing = ui.spacing().item_spacing;
    let width = ((ui.available_width() - spacing.x * 3.0) / 4.0).max(40.0);
    let height =
        ((ui.available_height() - DISPLAY_MIN_HEIGHT - spacing.y * 5.0) / 6.0).clamp(36.0, width);
    let size = egui::vec2(width, height);
    let total = height * 6.0 + spacing.y * 5.0;
    (size, total)
}

/// The rectangle (window pixels) in which content is safe from system UI,
/// shared with whichever platform backend tracks it.
///
/// On Android the OS reports this rectangle whenever system UI (status bar,
/// gesture bar, on-screen keyboard) changes. Desktop platforms never set it,
/// so no padding is applied there.
#[derive(Debug, Clone, Default)]
pub struct InsetsHandle {
    rect: Arc<InsetsRect>,
}

#[derive(Debug, Default)]
struct InsetsRect {
    left: AtomicI32,
    top: AtomicI32,
    right: AtomicI32,
    bottom: AtomicI32,
}

impl InsetsHandle {
    /// A handle with an empty rectangle. The app uses a conservative fallback
    /// until the platform reports real values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the content rectangle, in window pixels. Returns whether the
    /// rectangle changed.
    pub fn set_rect_px(&self, left: i32, top: i32, right: i32, bottom: i32) -> bool {
        let changed = self.rect_px() != (left, top, right, bottom);
        if changed {
            self.rect.left.store(left, Ordering::Relaxed);
            self.rect.top.store(top, Ordering::Relaxed);
            self.rect.right.store(right, Ordering::Relaxed);
            self.rect.bottom.store(bottom, Ordering::Relaxed);
        }
        changed
    }

    fn rect_px(&self) -> (i32, i32, i32, i32) {
        (
            self.rect.left.load(Ordering::Relaxed),
            self.rect.top.load(Ordering::Relaxed),
            self.rect.right.load(Ordering::Relaxed),
            self.rect.bottom.load(Ordering::Relaxed),
        )
    }

    /// Resolve the rectangle into point insets for this frame.
    fn resolve(&self, ctx: &egui::Context) -> SafeInsets {
        safe_insets(
            self.rect_px(),
            ctx.pixels_per_point(),
            ctx.viewport_rect().size(),
        )
    }
}

/// Space taken by system UI on each side, in egui points.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct SafeInsets {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

/// Used until the platform reports a non-empty content rectangle.
const FALLBACK_SAFE_AREA: SafeInsets = SafeInsets {
    left: 0.0,
    top: 36.0,
    right: 0.0,
    bottom: 28.0,
};

impl SafeInsets {
    fn to_margin(self) -> egui::Margin {
        egui::Margin {
            left: points_to_i8(self.left),
            top: points_to_i8(self.top),
            right: points_to_i8(self.right),
            bottom: points_to_i8(self.bottom),
        }
    }
}

/// Rounds a point value into the `i8` range used by [`egui::Margin`].
fn points_to_i8(points: f32) -> i8 {
    points.round().clamp(0.0, 120.0) as i8
}

/// Converts a content rectangle (window pixels) into point insets, given the
/// window's pixels-per-point scale and its size in points.
fn safe_insets(
    rect: (i32, i32, i32, i32),
    pixels_per_point: f32,
    screen: egui::Vec2,
) -> SafeInsets {
    let (left, top, right, bottom) = rect;
    if right <= 0 || bottom <= 0 {
        return FALLBACK_SAFE_AREA;
    }
    SafeInsets {
        left: (left as f32 / pixels_per_point).max(0.0),
        top: (top as f32 / pixels_per_point).max(0.0),
        right: (screen.x - right as f32 / pixels_per_point).max(0.0),
        bottom: (screen.y - bottom as f32 / pixels_per_point).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rect_falls_back_to_default_insets() {
        let insets = safe_insets((0, 0, 0, 0), 1.0, egui::vec2(400.0, 800.0));
        assert_eq!(insets, FALLBACK_SAFE_AREA);
    }

    #[test]
    fn partial_rect_falls_back_to_default_insets() {
        let insets = safe_insets((0, 120, 0, 0), 1.0, egui::vec2(400.0, 800.0));
        assert_eq!(insets, FALLBACK_SAFE_AREA);
    }

    #[test]
    fn rect_converts_from_pixels_to_points() {
        let insets = safe_insets((0, 120, 1080, 2280), 3.0, egui::vec2(360.0, 780.0));
        assert_eq!(
            insets,
            SafeInsets {
                left: 0.0,
                top: 40.0,
                right: 0.0,
                bottom: 20.0,
            }
        );
    }

    #[test]
    fn rect_beyond_screen_clamps_to_zero() {
        let insets = safe_insets((0, 10, 2000, 2000), 1.0, egui::vec2(400.0, 800.0));
        assert_eq!(insets.right, 0.0);
        assert_eq!(insets.bottom, 0.0);
        assert_eq!(insets.top, 10.0);
    }

    #[test]
    fn handle_reports_only_real_changes() {
        let handle = InsetsHandle::new();
        assert!(handle.set_rect_px(1, 2, 300, 400));
        assert!(!handle.set_rect_px(1, 2, 300, 400));
        assert!(handle.set_rect_px(1, 2, 300, 500));
        assert_eq!(handle.rect_px(), (1, 2, 300, 500));
    }

    #[test]
    fn margin_conversion_clamps_and_rounds() {
        let margin = SafeInsets {
            left: -3.4,
            top: 36.6,
            right: 500.0,
            bottom: 28.2,
        }
        .to_margin();
        assert_eq!(
            margin,
            egui::Margin {
                left: 0,
                top: 37,
                right: 120,
                bottom: 28,
            }
        );
    }
}
