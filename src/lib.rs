//! Dupot Dev Tools, split along clean-architecture lines:
//! - `domain`: pure logic and types (no GTK, no I/O), unit-tested from `tests/`;
//! - `infrastructure`: GTK views, database drivers, feature registry.

pub mod domain;
pub mod infrastructure;
