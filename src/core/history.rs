//! Calculation history log and paper tape data structures.

use crate::core::types::{BinaryOp, UnaryOp};

/// A single recorded calculation event on the paper tape.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    /// Incremental identifier of the operation.
    pub id: usize,
    /// First operand or input value.
    pub operand1: f64,
    /// Operator string representation (e.g. "+", "−", "×", "÷", "√", "%", "±").
    pub operator: String,
    /// Second operand for binary operations, or None for unary operations.
    pub operand2: Option<f64>,
    /// Resulting value.
    pub result: f64,
    /// Formatted calculation string (e.g. "12 + 34 = 46" or "√144 = 12").
    pub expression: String,
}

/// In-memory ring buffer tracking previous calculations.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryLog {
    entries: Vec<HistoryEntry>,
    max_capacity: usize,
    counter: usize,
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
            entries: Vec::with_capacity(max_capacity),
            max_capacity,
            counter: 0,
        }
    }

    /// Records a binary calculation.
    pub fn record_binary(&mut self, a: f64, op: BinaryOp, b: f64, result: f64) {
        self.counter += 1;
        let op_str = op.to_string();
        let expression = format!("{} {} {} = {}", a, op_str, b, result);

        let entry = HistoryEntry {
            id: self.counter,
            operand1: a,
            operator: op_str,
            operand2: Some(b),
            result,
            expression,
        };

        if self.entries.len() >= self.max_capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Records a unary calculation.
    pub fn record_unary(&mut self, val: f64, op: UnaryOp, result: f64) {
        self.counter += 1;
        let (op_str, expression) = match op {
            UnaryOp::SquareRoot => ("√".to_string(), format!("√({}) = {}", val, result)),
            UnaryOp::Percentage => ("%".to_string(), format!("{}% = {}", val, result)),
            UnaryOp::Negate => ("±".to_string(), format!("±({}) = {}", val, result)),
        };

        let entry = HistoryEntry {
            id: self.counter,
            operand1: val,
            operator: op_str,
            operand2: None,
            result,
            expression,
        };

        if self.entries.len() >= self.max_capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Returns a slice of all recorded history entries (oldest to newest).
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Returns the number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no history has been recorded yet.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clears all recorded history.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Formats the history as a classic continuous monospace receipt paper roll.
    pub fn format_paper_tape(&self) -> String {
        if self.entries.is_empty() {
            return "*** PAPER TAPE EMPTY ***\n".to_string();
        }

        let mut tape = String::from("═══ HEPTASEG PAPER TAPE ═══\n\n");
        for entry in &self.entries {
            tape.push_str(&format!("[#{:03}] {}\n", entry.id, entry.expression));
        }
        tape.push_str("\n═══════════════════════════\n");
        tape
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_binary_and_tape_format() {
        let mut log = HistoryLog::new(5);
        assert!(log.is_empty());

        log.record_binary(12.0, BinaryOp::Add, 34.0, 46.0);
        assert_eq!(log.len(), 1);
        assert_eq!(log.entries()[0].expression, "12 + 34 = 46");

        log.record_unary(144.0, UnaryOp::SquareRoot, 12.0);
        assert_eq!(log.len(), 2);
        assert_eq!(log.entries()[1].expression, "√(144) = 12");

        let tape = log.format_paper_tape();
        assert!(tape.contains("[#001] 12 + 34 = 46"));
        assert!(tape.contains("[#002] √(144) = 12"));

        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn test_history_capacity_limit() {
        let mut log = HistoryLog::new(3);
        for i in 1..=5 {
            log.record_binary(i as f64, BinaryOp::Add, 1.0, (i + 1) as f64);
        }
        assert_eq!(log.len(), 3);
        assert_eq!(log.entries()[0].id, 3);
        assert_eq!(log.entries()[2].id, 5);
    }
}
