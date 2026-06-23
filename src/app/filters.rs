//! Unit type filter tabs shared between the app state and the UI.

use crate::systemd::Unit;

/// Unit type filter tabs. `None` means "all unit types".
pub const FILTERS: &[(&str, Option<&str>)] = &[
    ("All", None),
    ("service", Some("service")),
    ("socket", Some("socket")),
    ("timer", Some("timer")),
    ("target", Some("target")),
    ("mount", Some("mount")),
];

/// Whether `u` is included by the filter at `filter_idx`.
pub fn matches_filter(u: &Unit, filter_idx: usize) -> bool {
    match FILTERS[filter_idx].1 {
        None => true,
        Some(t) => u.name.ends_with(&format!(".{t}")),
    }
}