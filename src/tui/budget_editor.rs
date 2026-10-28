//! Free-form percentage entry for the five per-tool output budgets.
//!
//! Arrow keys walk the coarse quarter stops, which covers picking a share by feel; landing on an
//! arbitrary share needs the number typed directly. Typing is all this editor does — `automatic`
//! is a stop on the arrow-key cycle, not a word anyone has to know to spell.

use crate::control::settings::ToolBudgetLevel;

/// Editable per-tool share plus the last validation failure.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BudgetEditor {
    /// Raw text as typed, including a trailing percent sign if the user entered one.
    pub(crate) input: String,
    /// Validation failure from the most recent submission, cleared by any further edit.
    pub(crate) error: Option<ToolBudgetInputError>,
}

/// User-input failure category for the TUI's editable per-tool budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolBudgetInputError {
    /// The field was submitted with nothing in it.
    Empty,
    /// The text is not a whole number.
    NotInteger,
    /// The number parsed but falls outside `1..=100`.
    OutOfRange,
}

/// Parses a submitted share.
///
/// An empty edit is rejected rather than silently standing in for some other value: a stray Enter
/// should not discard an explicit share.
pub(crate) fn parse_input(input: &str) -> Result<ToolBudgetLevel, ToolBudgetInputError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ToolBudgetInputError::Empty);
    }
    let digits = input.strip_suffix('%').unwrap_or(input).trim();
    let percent = digits
        .parse::<i64>()
        .map_err(|_| ToolBudgetInputError::NotInteger)?;
    u8::try_from(percent)
        .ok()
        .and_then(ToolBudgetLevel::from_percent)
        .ok_or(ToolBudgetInputError::OutOfRange)
}
