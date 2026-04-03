mod alerts;
mod app;
mod backends;
mod events;
mod models;
mod ui;

use anyhow::Result;
use app::AppState;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::EventHandler;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "portscope")]
#[command(author, version, about = "A cross-platform TUI for monitoring ports and managing processes", long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 2000, help = "Auto-refresh interval in milliseconds")]
    refresh_interval: u64,

    #[arg(short, long, help = "Initial filter to apply")]
    filter: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, args);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    args: Args,
) -> Result<()> {
    let mut state = AppState::new();
    let mut event_handler = EventHandler::new();

    if let Some(filter) = args.filter {
        state.filter = filter;
    }

    state.refresh()?;

    let refresh_interval = Duration::from_millis(args.refresh_interval);
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &state, &event_handler))?;

        let timeout = refresh_interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::from_millis(100));

        let action = event_handler.next_action(timeout)?;

        let should_quit = state.apply_action(action)?;
        if should_quit {
            break;
        }

        if last_refresh.elapsed() >= refresh_interval {
            state.refresh()?;
            last_refresh = Instant::now();
        }
    }

    Ok(())
}
