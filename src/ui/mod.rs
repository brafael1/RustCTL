//! Rendering: turns [`App`] state into ratatui widgets.

mod render;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app::App;

/// Top-level draw dispatcher called once per frame.
pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(main_constraints())
        .split(area);

    render::title(f, app, chunks[0]);
    render::main(f, app, chunks[1]);
    render::status(f, app, chunks[2]);
}

/// Vertical layout: [tabs (3) | main (fill) | status bar (3)].
pub fn main_constraints() -> [Constraint; 3] {
    [Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)]
}