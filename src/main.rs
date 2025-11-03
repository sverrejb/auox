use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io;

use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

mod auth;
mod config;
mod ui;
mod models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::get_config();

    auth::auth(config.client_id);

    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    // Our list of names
    let names: Vec<&str> = vec!["Norma", "Bob", "Charlie", "Diana", "Eve", "Frank"];

    // Track selected item
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        ui::draw(&mut terminal, &mut state, &names);

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        let i = match state.selected() {
                            Some(i) => {
                                if i >= names.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    names.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
