//! Tests for history persistence (`serialize_history` / `deserialize_history`).

use proptest::prelude::*;
use rustcalc_core::{
    Calculator, HISTORY_CAPACITY, HistoryEntry, Op, deserialize_history, serialize_history,
};

fn entry(expression: &str, value: f64) -> HistoryEntry {
    HistoryEntry {
        expression: expression.to_string(),
        result: String::new(),
        value,
    }
}

#[test]
fn round_trips_calculator_history() {
    let mut calculator = Calculator::new();
    calculator.input_digit(1);
    calculator.input_digit(2);
    calculator.input_decimal();
    calculator.input_digit(5);
    calculator.set_op(Op::Add);
    calculator.input_digit(3);
    calculator.equals();
    calculator.set_op(Op::Multiply);
    calculator.input_digit(8);
    calculator.equals();

    let text = serialize_history(calculator.history());
    let restored = deserialize_history(&text);

    assert_eq!(restored.as_slice(), calculator.history());
}

#[test]
fn round_trips_extreme_values() {
    let entries = vec![entry("big", 1e300), entry("small", 9.9e-10)];
    let restored = deserialize_history(&serialize_history(&entries));

    assert_eq!(restored.len(), 2);
    assert_eq!(restored[0].value, 1e300);
    assert_eq!(restored[1].value, 9.9e-10);
    assert_eq!(restored[1].result, "9.9e-10");
}

#[test]
fn skips_malformed_lines() {
    let text = "garbage\n\
                1.5\n\
                abc\tbroken\ttabs\n\
                2\tvalid\n\
                NaN\tnot a number\n\
                inf\tinfinite\n\
                -0.5\tneg\n\
                1000\tgrouped\n";
    let restored = deserialize_history(text);

    assert_eq!(restored.len(), 3);
    assert_eq!(restored[0].expression, "valid");
    assert_eq!(restored[0].value, 2.0);
    assert_eq!(restored[0].result, "2");
    assert_eq!(restored[1].expression, "neg");
    assert_eq!(restored[1].value, -0.5);
    assert_eq!(restored[2].expression, "grouped");
    assert_eq!(restored[2].result, "1,000");
}

#[test]
fn empty_text_yields_empty_history() {
    assert!(deserialize_history("").is_empty());
    assert!(deserialize_history("\n\n").is_empty());
}

#[test]
fn deserialize_keeps_newest_within_capacity() {
    let entries: Vec<HistoryEntry> = (0..(HISTORY_CAPACITY + 50))
        .map(|index| entry(&format!("{index} + 1"), index as f64))
        .collect();

    let restored = deserialize_history(&serialize_history(&entries));

    assert_eq!(restored.len(), HISTORY_CAPACITY);
    assert_eq!(restored[0].value, 50.0);
    assert_eq!(
        restored[HISTORY_CAPACITY - 1].value,
        (HISTORY_CAPACITY + 49) as f64
    );
}

#[test]
fn deserialize_keeps_exactly_capacity_entries() {
    let entries: Vec<HistoryEntry> = (0..HISTORY_CAPACITY)
        .map(|index| entry(&format!("{index} + 1"), index as f64))
        .collect();

    let restored = deserialize_history(&serialize_history(&entries));

    assert_eq!(restored.len(), HISTORY_CAPACITY);
    assert_eq!(restored[0].value, 0.0);
    assert_eq!(
        restored[HISTORY_CAPACITY - 1].value,
        (HISTORY_CAPACITY - 1) as f64
    );
}

#[test]
fn import_truncates_to_capacity() {
    let entries: Vec<HistoryEntry> = (0..(HISTORY_CAPACITY + 50))
        .map(|index| entry(&format!("{index} + 1"), index as f64))
        .collect();

    let mut calculator = Calculator::new();
    calculator.import_history(entries);

    let history = calculator.history();
    assert_eq!(history.len(), HISTORY_CAPACITY);
    assert_eq!(history[0].value, 50.0);
    assert_eq!(
        history[HISTORY_CAPACITY - 1].value,
        (HISTORY_CAPACITY + 49) as f64
    );
}

#[test]
fn import_keeps_exactly_capacity_entries() {
    let entries: Vec<HistoryEntry> = (0..HISTORY_CAPACITY)
        .map(|index| entry(&format!("{index} + 1"), index as f64))
        .collect();

    let mut calculator = Calculator::new();
    calculator.import_history(entries);

    let history = calculator.history();
    assert_eq!(history.len(), HISTORY_CAPACITY);
    assert_eq!(history[0].value, 0.0);
    assert_eq!(
        history[HISTORY_CAPACITY - 1].value,
        (HISTORY_CAPACITY - 1) as f64
    );
}

#[test]
fn imported_history_is_recallable() {
    let mut calculator = Calculator::new();
    calculator.import_history(vec![entry("7 × 6", 42.0)]);

    assert!(calculator.recall_history(0));
    assert_eq!(calculator.display(), "42");
}

proptest! {
    #[test]
    fn serialization_round_trips(
        entries in prop::collection::vec(
            (-1e308f64..1e308, "[0-9 +×−÷%.]{1,24}"),
            0..50,
        ),
    ) {
        let original: Vec<HistoryEntry> =
            entries.iter().map(|(value, expression)| entry(expression, *value)).collect();

        let restored = deserialize_history(&serialize_history(&original));

        prop_assert_eq!(restored.len(), original.len());
        let pairs_match = original
            .iter()
            .zip(&restored)
            .all(|(before, after)| before.expression == after.expression && before.value == after.value);
        prop_assert!(pairs_match);
    }
}
