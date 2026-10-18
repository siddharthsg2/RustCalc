//! The egui application shell.
//!
//! Renders the calculator and forwards input to `rustcalc_core`.

use eframe::egui;
use egui::{Align, Button, Color32, Key, Layout, RichText};
use rustcalc_core::{Calculator, Op};

use crate::theme;

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
    fn fill(self) -> Option<Color32> {
        match self {
            Self::Digit => None,
            Self::Function => Some(theme::FUNCTION_BG),
            Self::Operator => Some(theme::OPERATOR_BG),
            Self::Equals => Some(theme::ACCENT),
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
        }
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
        ui.vertical(|ui| {
            self.top_bar(ui, &mut actions);
            self.history_section(ui, &mut actions);
            self.display_section(ui);
            Self::keypad(ui, &mut actions);
        });

        if !actions.is_empty() {
            ui.ctx().request_repaint();
        }
        for action in actions {
            self.apply(action);
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
    fn keyboard_actions(ctx: &egui::Context) -> Vec<Action> {
        ctx.input(|input| {
            let mut actions = Vec::new();

            let digits = [
                (Key::Num0, 0_u8),
                (Key::Num1, 1),
                (Key::Num2, 2),
                (Key::Num3, 3),
                (Key::Num4, 4),
                (Key::Num5, 5),
                (Key::Num6, 6),
                (Key::Num7, 7),
                (Key::Num8, 8),
                (Key::Num9, 9),
            ];
            for (key, digit) in digits {
                if input.key_pressed(key) {
                    actions.push(Action::Digit(digit));
                }
            }

            let operators = [
                (Key::Plus, Op::Add),
                (Key::Minus, Op::Subtract),
                (Key::Slash, Op::Divide),
            ];
            for (key, op) in operators {
                if input.key_pressed(key) {
                    actions.push(Action::Op(op));
                }
            }

            // `*` and `%` have no logical key in egui; they arrive as text.
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    match text.as_str() {
                        "*" => actions.push(Action::Op(Op::Multiply)),
                        "%" => actions.push(Action::Percent),
                        _ => {}
                    }
                }
            }

            if input.key_pressed(Key::Period) {
                actions.push(Action::Decimal);
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

    fn display_section(&self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            ui.label(
                RichText::new(self.calculator.expression())
                    .weak()
                    .size(15.0),
            );
        });

        let text = self.calculator.display();
        let color = if self.calculator.error().is_some() {
            theme::ERROR
        } else {
            ui.visuals().strong_text_color()
        };
        let size = fit_display_size(ui, &text, ui.available_width(), color);
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            ui.label(RichText::new(&text).size(size).color(color).strong());
        });
        ui.add_space(4.0);
    }

    fn keypad(ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let spacing = ui.spacing().item_spacing;
        let button_size = egui::vec2(
            ((ui.available_width() - spacing.x * 3.0) / 4.0).max(40.0),
            ((ui.available_height() - spacing.y * 5.0) / 6.0).clamp(36.0, 64.0),
        );

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
}

fn key_button(ui: &mut egui::Ui, def: &KeyDef, size: egui::Vec2) -> egui::Response {
    let text_color = match def.kind {
        KeyKind::Equals => theme::ACCENT_TEXT,
        _ => ui.visuals().text_color(),
    };
    let mut button = Button::new(RichText::new(def.label).size(20.0).color(text_color))
        .min_size(size)
        .corner_radius(10);
    if let Some(fill) = def.kind.fill() {
        button = button.fill(fill);
    }
    ui.add(button)
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
