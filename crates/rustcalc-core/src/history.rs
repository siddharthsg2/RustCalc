//! Calculation history and its persistence format.

use crate::format;

/// Maximum number of entries kept in the history.
pub const HISTORY_CAPACITY: usize = 100;

/// One completed calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    /// The calculation as displayed, e.g. `"12.5 + 3"`.
    pub expression: String,
    /// The formatted result as displayed, e.g. `"15.5"`.
    pub result: String,
    /// The exact result value, used when recalling the entry.
    pub value: f64,
}

/// Serialize history to a newline-separated text format.
///
/// Each line is `value<TAB>expression`. The result string is not stored: it
/// is recomputed from the value when loading, so the file only contains data
/// that cannot be derived.
pub fn serialize_history(entries: &[HistoryEntry]) -> String {
    let mut output = String::new();
    for entry in entries {
        // `{:?}` on f64 is the shortest representation that round-trips
        // exactly through `str::parse`, and stays compact for huge values.
        output.push_str(&format!("{:?}\t{}\n", entry.value, entry.expression));
    }
    output
}

/// Parse text produced by [`serialize_history`].
///
/// Malformed lines are skipped, and at most the newest [`HISTORY_CAPACITY`]
/// entries are kept.
pub fn deserialize_history(text: &str) -> Vec<HistoryEntry> {
    let mut entries: Vec<HistoryEntry> = text.lines().filter_map(parse_entry).collect();
    let excess = entries.len().saturating_sub(HISTORY_CAPACITY);
    entries.drain(..excess);
    entries
}

fn parse_entry(line: &str) -> Option<HistoryEntry> {
    let (value_text, expression) = line.split_once('\t')?;
    let value: f64 = value_text.parse().ok()?;
    if expression.is_empty() || !value.is_finite() {
        return None;
    }
    Some(HistoryEntry {
        expression: expression.to_string(),
        result: format::group_digits(&format::format_number(value)),
        value,
    })
}

#[derive(Debug, Default)]
pub(crate) struct History {
    entries: Vec<HistoryEntry>,
}

impl History {
    pub(crate) fn push(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
        if self.entries.len() > HISTORY_CAPACITY {
            self.entries.remove(0);
        }
    }

    pub(crate) fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub(crate) fn replace(&mut self, mut entries: Vec<HistoryEntry>) {
        let excess = entries.len().saturating_sub(HISTORY_CAPACITY);
        entries.drain(..excess);
        self.entries = entries;
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}
