//! Calculation history.

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

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}
