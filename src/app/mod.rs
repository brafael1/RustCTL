//! Application state and the actions the user can trigger on it.

use std::io;
use std::time::{Duration, Instant};

use ratatui::widgets::ListState;

use crate::systemd::{Unit, list_units};

pub mod filters;

/// How long a flashed message stays on the status bar.
const MESSAGE_TTL: Duration = Duration::from_secs(4);

pub struct App {
    pub units: Vec<Unit>,
    pub list_state: ListState,
    pub filter: usize,
    pub last_refresh: Instant,
    pub message: Option<(String, Instant)>,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            units: Vec::new(),
            list_state: ListState::default(),
            filter: 0,
            last_refresh: Instant::now() - Duration::from_secs(3600),
            message: None,
        };
        let _ = app.refresh();
        app.list_state.select(Some(0));
        app
    }

    /// Units that pass the active filter, in list order.
    pub fn visible(&self) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|u| filters::matches_filter(u, self.filter))
            .collect()
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        match list_units(None) {
            Ok(units) => {
                self.units = units;
                self.last_refresh = Instant::now();
            }
            Err(e) => self.flash(format!("refresh error: {e}")),
        }
        Ok(())
    }

    pub fn flash(&mut self, msg: String) {
        self.message = Some((msg, Instant::now()));
    }

    pub fn selected_unit(&self) -> Option<&Unit> {
        let idx = self.list_state.selected()?;
        self.visible().get(idx).copied()
    }

    /// Expire a flashed message once its TTL elapses.
    pub fn tick_message(&mut self) {
        if let Some((_, t)) = &self.message {
            if t.elapsed() > MESSAGE_TTL {
                self.message = None;
            }
        }
    }

    /// Perform a systemctl verb on the currently selected unit, then refresh.
    pub fn act<F>(&mut self, label: &str, f: F)
    where
        F: FnOnce(&str) -> anyhow::Result<String>,
    {
        let name = match self.selected_unit().map(|u| u.name.clone()) {
            Some(n) => n,
            None => {
                self.flash("no unit selected".to_string());
                return;
            }
        };
        match f(&name) {
            Ok(_) => self.flash(format!("{label} {name}: ok")),
            Err(e) => self.flash(format!("{label} {name} failed: {e}")),
        }
        let _ = self.refresh();
    }

    pub fn next_filter(&mut self) {
        self.filter = (self.filter + 1) % filters::FILTERS.len();
        self.clamp_selection();
    }

    pub fn select_offset(&mut self, delta: i32) {
        let len = self.visible().len();
        if len == 0 {
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as i32;
        let mut next = current + delta;
        if next < 0 {
            next = 0;
        }
        if next as usize >= len {
            next = (len - 1) as i32;
        }
        self.list_state.select(Some(next as usize));
    }

    pub fn select_first(&mut self) {
        if self.visible().is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        let last = self.visible().len().saturating_sub(1);
        if last == 0 && self.visible().is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(last));
        }
    }

    /// `ui` needs the enabled state for the detail panel; exposed here so the
    /// render module stays free of `systemd::` calls that mutate nothing.
    pub fn enabled_for_selected(&self) -> Option<crate::systemd::Enabled> {
        self.selected_unit().map(|u| crate::systemd::is_enabled(&u.name))
    }

    pub fn status_for_selected(&self) -> Option<anyhow::Result<String>> {
        self.selected_unit()
            .map(|u| crate::systemd::status(&u.name))
    }

    /// Keep the selection within bounds after the filter or list changes.
    fn clamp_selection(&mut self) {
        let max = self.visible().len().saturating_sub(1);
        if self.list_state.selected().map_or(0, |i| i) > max {
            self.list_state.select(Some(max));
        }
        if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }
}