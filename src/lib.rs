//! Heptaseg: Vintage 7-Segment LCD Pocket Calculator library.

pub mod core;

pub use core::fsm::CalculatorFsm;
pub use core::history::{HistoryEntry, HistoryLog};
pub use core::types::{BinaryOp, CalculatorError, Key, MemoryOp, StatusFlags, UnaryOp};

