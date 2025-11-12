use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::path::PathBuf;

mod app;
mod file_browser;
mod metadata;
mod ui;

use app::{App, InputMode};

/// A terminal user interface for editing ID3v2 tags and audio metadata
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to start browsing from (defaults to current directory)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(args.path)?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('h') => app.toggle_help(),
                        KeyCode::Char('e') => app.enter_edit_mode(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Enter => app.select_item()?,
                        KeyCode::Backspace => app.go_back()?,
                        KeyCode::Tab => app.toggle_focus(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => app.exit_edit_mode(),
                        KeyCode::Enter => app.save_field()?,
                        KeyCode::Backspace => app.delete_char(),
                        KeyCode::Left => app.move_cursor_left(),
                        KeyCode::Right => app.move_cursor_right(),
                        KeyCode::Up => app.previous_field(),
                        KeyCode::Down => app.next_field(),
                        KeyCode::Char(c) => app.insert_char(c),
                        _ => {}
                    },
                }
            }
        }
    }
}
