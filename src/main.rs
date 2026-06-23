use std::io;
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod app;
mod event;
mod systemd;
mod ui;

use app::App;

/// Interval between input polls; also bounds how often we expire messages.
const POLL_MS: u64 = 250;

fn main() -> io::Result<()> {
    check_root_hint();

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run(&mut terminal)?;

    restore(&mut terminal)?;
    Ok(())
}

/// Main event loop: draw, poll, dispatch keys, expire messages.
fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if crossterm::event::poll(Duration::from_millis(POLL_MS))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if event::handle_key(&mut app, key) {
                    break;
                }
            }
        }

        app.tick_message();
    }

    Ok(())
}

fn restore(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Warn once if not running as root: start/init operations need root or polkit.
fn check_root_hint() {
    let uid = unsafe { getuid() };
    if uid != 0 {
        eprintln!(
            "rustctl: not running as root. Start/init operations will fail\n\
             unless you've configured polkit. Press Enter to continue anyway."
        );
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

unsafe extern "C" {
    fn getuid() -> u32;
}