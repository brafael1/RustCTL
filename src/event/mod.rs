//! Keyboard input handling.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::systemd;

/// Returns `true` if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    use KeyCode::*;
    match (key.code, key.modifiers) {
        (Char('q'), _) | (Char('c'), KeyModifiers::CONTROL) => return true,
        (Tab, _) => app.next_filter(),
        // Shift+R refreshes the unit list; plain `r` restarts the selected unit.
        (Char('R'), mods) if mods.contains(KeyModifiers::SHIFT) => {
            let _ = app.refresh();
            app.flash("refreshed".to_string());
        }
        (Char('r'), _) => app.act("restart", systemd::restart),
        (Char('s'), _) => app.act("start", systemd::start),
        (Char('S'), _) => app.act("stop", systemd::stop),
        (Char('e'), _) => app.act("enable", systemd::enable),
        (Char('E'), _) => app.act("disable", systemd::disable),
        (Char('l'), _) => app.act("reload", systemd::reload),
        (Down, _) | (Char('j'), _) => app.select_offset(1),
        (Up, _) | (Char('k'), _) => app.select_offset(-1),
        (PageDown, _) => app.select_offset(10),
        (PageUp, _) => app.select_offset(-10),
        (Home, _) => app.select_first(),
        (End, _) => app.select_last(),
        _ => {}
    }
    false
}