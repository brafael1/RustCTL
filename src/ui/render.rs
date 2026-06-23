//! Individual widgets that make up the screen.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::filters::FILTERS;
use crate::app::App;
use crate::systemd::Enabled;

pub fn title(f: &mut Frame<'_>, app: &App, area: Rect) {
    let mut tabs: Vec<Span> = Vec::new();
    for (i, (name, _)) in FILTERS.iter().enumerate() {
        let (color, style_mod) = if i == app.filter {
            (Color::Black, Modifier::BOLD)
        } else {
            (Color::Reset, Modifier::empty())
        };
        tabs.push(Span::styled(
            format!(" {name} "),
            Style::default()
                .fg(color)
                .bg(if i == app.filter { Color::Cyan } else { Color::Reset })
                .add_modifier(style_mod),
        ));
        tabs.push(Span::raw(" "));
    }
    let block = Block::default()
        .title("rustctl — systemd manager")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(Line::from(tabs))
        .block(block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph, area);
}

pub fn main(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Min(1)])
        .split(area);

    unit_list(f, app, chunks[0]);
    detail(f, app, chunks[1]);
}

fn unit_list(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    let visible: Vec<_> = app.visible();

    let items: Vec<ListItem> = visible
        .iter()
        .map(|u| {
            let active_color = match u.active_state.as_str() {
                "active" => Color::Green,
                "inactive" | "failed" => Color::Red,
                "activating" | "deactivating" => Color::Yellow,
                _ => Color::Gray,
            };
            let line = Line::from(vec![
                Span::styled(state_dot(&u.active_state), Style::default().fg(active_color)),
                Span::raw(" "),
                Span::styled(u.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!("Units ({})  ⇄ Tab to filter", visible.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

pub fn detail(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Details");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let content: Vec<Line> = match app.selected_unit() {
        Some(u) => render_unit_lines(app, u, inner.height),
        None => vec![Line::from("No unit selected.")],
    };

    let paragraph = Paragraph::new(content).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_unit_lines<'a>(app: &'a App, u: &'a crate::systemd::Unit, inner_height: u16) -> Vec<Line<'a>> {
    let enabled = app
        .enabled_for_selected()
        .unwrap_or(Enabled::Other);
    let active_color = match u.active_state.as_str() {
        "active" => Color::Green,
        "inactive" | "failed" => Color::Red,
        "activating" | "deactivating" => Color::Yellow,
        _ => Color::Gray,
    };
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Unit       ", Style::default().fg(Color::Cyan)),
            Span::raw(u.name.clone()),
        ]),
        Line::from(vec![
            Span::styled("Description", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::raw(u.description.clone()),
        ]),
        Line::from(vec![
            Span::styled("Load       ", Style::default().fg(Color::Cyan)),
            Span::raw(u.load_state.clone()),
        ]),
        Line::from(vec![
            Span::styled("Active     ", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(
                u.active_state.clone(),
                Style::default().fg(active_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Sub        ", Style::default().fg(Color::Cyan)),
            Span::raw(u.sub_state.clone()),
        ]),
        Line::from(vec![
            Span::styled("Enabled    ", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(
                enabled.label(),
                Style::default().fg(match enabled {
                    Enabled::Enabled => Color::Green,
                    Enabled::Disabled => Color::Red,
                    Enabled::Masked => Color::Magenta,
                    _ => Color::DarkGray,
                }),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Status",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED),
        )),
    ];

    match app.status_for_selected() {
        Some(Ok(text)) => {
            let take = inner_height.saturating_sub(9) as usize;
            for l in text.lines().take(take) {
                lines.push(Line::from(l.to_string()));
            }
        }
        Some(Err(e)) => lines.push(Line::from(format!("(status error: {e})"))),
        None => {}
    }
    lines
}

pub fn status(f: &mut Frame<'_>, app: &App, area: Rect) {
    let help = "q quit ↑↓ move Tab filter s start S stop r restart l reload e enable E disable R refresh";
    let msg = app
        .message
        .as_ref()
        .map(|(m, _)| m.clone())
        .unwrap_or_else(|| help.to_string());

    let style = if app.message.is_some() {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default().borders(Borders::ALL);
    f.render_widget(Paragraph::new(msg).block(block).style(style), area);
}

/// Map an active-state string to a single glyph for the list rows.
fn state_dot(state: &str) -> &'static str {
    match state {
        "active" => "●",
        "inactive" => "○",
        "failed" => "✖",
        "activating" | "deactivating" => "◔",
        _ => "?",
    }
}