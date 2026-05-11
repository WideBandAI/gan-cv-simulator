mod app;
mod render;
mod types;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::config::configuration_builder::ConfigurationBuilder;
use app::App;
use render::draw;
use types::Page;

// ─── Event loop ───────────────────────────────────────────────────────────────

/// Run the TUI configuration wizard and return a [`ConfigurationBuilder`].
pub fn run_tui() -> anyhow::Result<ConfigurationBuilder> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> anyhow::Result<ConfigurationBuilder> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Err(anyhow::anyhow!("TUI cancelled by user"));
                }
                KeyCode::Char(' ') if app.is_toggle() => {
                    app.toggle_focused();
                }
                KeyCode::Char(c) => app.type_char(c),
                KeyCode::Backspace => app.backspace(),
                KeyCode::Tab | KeyCode::Down => app.next_field(),
                KeyCode::BackTab | KeyCode::Up => app.prev_field(),
                KeyCode::Enter | KeyCode::Right => {
                    if app.page == Page::Confirm {
                        return Ok(app.build_config());
                    }
                    app.validate_and_advance();
                }
                KeyCode::Esc | KeyCode::Left => app.go_back(),
                _ => {}
            }
        }
    }
}
