//! End-to-end UI tests driving the real app through egui_kittest.

use std::collections::HashMap;

use eframe::Storage as _;
use eframe::egui::Key;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use rustcalc_app::app::{HISTORY_STORAGE_KEY, RustCalcApp};

fn harness_with(app: RustCalcApp) -> Harness<'static, RustCalcApp> {
    let mut harness = Harness::new_ui_state(|ui, app: &mut RustCalcApp| app.ui(ui), app);
    harness.run();
    harness
}

fn harness() -> Harness<'static, RustCalcApp> {
    harness_with(RustCalcApp::new())
}

fn click_button(harness: &mut Harness<'_, RustCalcApp>, label: &str) {
    harness.get_by_role_and_label(Role::Button, label).click();
    harness.run();
}

/// Queue a text-input event (used for `*` and `%`, which have no logical key).
fn press_text(harness: &mut Harness<'_, RustCalcApp>, text: &str) {
    harness
        .input_mut()
        .events
        .push(eframe::egui::Event::Text(text.to_owned()));
    harness.run();
}

#[test]
fn clicking_digits_computes_result() {
    let mut harness = harness();
    click_button(&mut harness, "7");
    click_button(&mut harness, "+");
    click_button(&mut harness, "3");
    click_button(&mut harness, "=");
    assert_eq!(harness.state().display(), "10");
}

#[test]
fn keyboard_computes_result() {
    let mut harness = harness();
    harness.key_press(Key::Num7);
    harness.key_press(Key::Plus);
    harness.key_press(Key::Num3);
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().display(), "10");
}

#[test]
fn escape_clears_everything() {
    let mut harness = harness();
    harness.key_press(Key::Num9);
    harness.key_press(Key::Num9);
    harness.run();
    assert_eq!(harness.state().display(), "99");
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().display(), "0");
}

#[test]
fn backspace_removes_last_digit() {
    let mut harness = harness();
    click_button(&mut harness, "1");
    click_button(&mut harness, "2");
    click_button(&mut harness, "DEL");
    assert_eq!(harness.state().display(), "1");
}

#[test]
fn decimal_and_toggle_sign_and_ce() {
    let mut harness = harness();
    click_button(&mut harness, "1");
    click_button(&mut harness, ".");
    click_button(&mut harness, "5");
    click_button(&mut harness, "±");
    assert_eq!(harness.state().display(), "-1.5");
    click_button(&mut harness, "CE");
    assert_eq!(harness.state().display(), "0");
}

#[test]
fn division_by_zero_shows_error_and_c_recovers() {
    let mut harness = harness();
    click_button(&mut harness, "5");
    click_button(&mut harness, "÷");
    click_button(&mut harness, "0");
    click_button(&mut harness, "=");
    assert_eq!(harness.state().display(), "Cannot divide by zero");
    click_button(&mut harness, "C");
    assert_eq!(harness.state().display(), "0");
}

#[test]
fn percent_uses_windows_semantics() {
    let mut harness = harness();
    harness.key_press(Key::Num5);
    harness.key_press(Key::Num0);
    harness.key_press(Key::Plus);
    harness.key_press(Key::Num1);
    harness.key_press(Key::Num0);
    harness.run();
    press_text(&mut harness, "%");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().display(), "55");
}

#[test]
fn star_text_event_multiplies() {
    let mut harness = harness();
    harness.key_press(Key::Num6);
    harness.key_press(Key::Num7);
    harness.run();
    press_text(&mut harness, "*");
    harness.key_press(Key::Num1);
    harness.key_press(Key::Num0);
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().display(), "670");
}

#[test]
fn history_panel_lists_and_recalls() {
    let mut harness = harness();
    click_button(&mut harness, "2");
    click_button(&mut harness, "+");
    click_button(&mut harness, "3");
    click_button(&mut harness, "=");
    click_button(&mut harness, "History");

    harness.get_by_label("2 + 3 = 5").click();
    harness.run();
    assert_eq!(harness.state().display(), "5");
}

#[test]
fn clear_history_empties_panel() {
    let mut harness = harness();
    click_button(&mut harness, "1");
    click_button(&mut harness, "+");
    click_button(&mut harness, "1");
    click_button(&mut harness, "=");
    click_button(&mut harness, "History");
    click_button(&mut harness, "Clear history");
    assert_eq!(harness.state().calculator().history().len(), 0);
    assert_eq!(harness.state().display(), "2");
}

/// In-memory [`eframe::Storage`] for testing persistence without touching disk.
#[derive(Default)]
struct MockStorage {
    values: HashMap<String, String>,
}

impl eframe::Storage for MockStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.values.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.values.remove(key);
    }

    fn flush(&mut self) {}
}

#[test]
fn save_writes_history_to_storage() {
    let mut harness = harness();
    click_button(&mut harness, "5");
    click_button(&mut harness, "×");
    click_button(&mut harness, "4");
    click_button(&mut harness, "=");

    let mut storage = MockStorage::default();
    eframe::App::save(harness.state_mut(), &mut storage);

    let saved = storage
        .get_string(HISTORY_STORAGE_KEY)
        .expect("save must write the history key");
    let entries = rustcalc_core::deserialize_history(&saved);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].expression, "5 × 4");
    assert_eq!(entries[0].result, "20");
    assert_eq!(entries[0].value, 20.0);
}

#[test]
fn history_persists_across_restart() {
    let mut harness = harness();
    click_button(&mut harness, "2");
    click_button(&mut harness, "+");
    click_button(&mut harness, "3");
    click_button(&mut harness, "=");
    let saved = harness.state().history_text();

    let mut app = RustCalcApp::new();
    app.restore_history(&saved);
    let mut restarted = harness_with(app);

    assert_eq!(restarted.state().display(), "0");
    click_button(&mut restarted, "History");
    restarted.get_by_label("2 + 3 = 5");

    let history = restarted.state().calculator().history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].value, 5.0);
}
