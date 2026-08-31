//! Calculation audit and paper tape history log.

/// A single recorded transaction in the paper tape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Sequential transaction identifier.
    pub id: usize,
    /// Arithmetic expression executed (e.g., "120 + 35 =" or "144 √").
    pub expression: String,
    /// Evaluated numerical result string (e.g., "155" or "12").
    pub result: String,
}

/// An in-memory append-only audit trail representing the paper tape roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryLog {
    entries: Vec<HistoryEntry>,
    next_id: usize,
    max_capacity: usize,
}

impl Default for HistoryLog {
    fn default() -> Self {
        Self::new(100)
    }
}

impl HistoryLog {
    /// Creates a new history log with a maximum capacity.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
            max_capacity,
        }
    }

    /// Records a new completed calculation into the log.
    pub fn record(&mut self, expression: impl Into<String>, result: impl Into<String>) {
        if self.entries.len() >= self.max_capacity {
            self.entries.remove(0);
        }

        let entry = HistoryEntry {
            id: self.next_id,
            expression: expression.into(),
            result: result.into(),
        };
        self.next_id += 1;
        self.entries.push(entry);
    }

    /// Clears the entire history log.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_id = 1;
    }

    /// Returns a slice of all recorded entries.
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Returns whether the history log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_recording_and_clearing() {
        let mut log = HistoryLog::new(5);
        assert!(log.is_empty());

        log.record("2 + 2 =", "4");
        log.record("10 * 3 =", "30");

        assert_eq!(log.len(), 2);
        assert_eq!(log.entries()[0].expression, "2 + 2 =");
        assert_eq!(log.entries()[0].result, "4");
        assert_eq!(log.entries()[1].expression, "10 * 3 =");
        assert_eq!(log.entries()[1].result, "30");

        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn test_history_capacity_overflow() {
        let mut log = HistoryLog::new(2);
        log.record("1 + 1 =", "2");
        log.record("2 + 2 =", "4");
        log.record("3 + 3 =", "6");

        assert_eq!(log.len(), 2);
        assert_eq!(log.entries()[0].expression, "2 + 2 =");
        assert_eq!(log.entries()[1].expression, "3 + 3 =");
    }
}
