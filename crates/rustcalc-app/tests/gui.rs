//! End-to-end UI tests driving the real app through egui_kittest.

use eframe::egui::Key;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use rustcalc_app::app::RustCalcApp;

fn harness() -> Harness<'static, RustCalcApp> {
    let mut harness =
        Harness::new_ui_state(|ui, app: &mut RustCalcApp| app.ui(ui), RustCalcApp::new());
    harness.run();
    harness
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
