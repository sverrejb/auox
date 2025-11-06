use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::debug;
use std::{io, time::Instant};

use ratatui::{Terminal, backend::CrosstermBackend, style::Color, widgets::TableState};

use crate::{fileio::read_access_token_file, models::Account};

mod api;
mod auth;
mod fileio;
mod models;
mod ui;

use tachyonfx::{EffectManager, Interpolation, fx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger (off by default, enable with RUST_LOG=debug)
    env_logger::init();

    let config = fileio::get_config_file();

    auth::auth(config.client_id, config.client_secret);

    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    // Our list of names
    //let names: Vec<&str> = vec!["Norma", "Bob", "Charlie", "Diana", "Eve", "Frank"];

    let accounts = get_accounts();

    // Track selected item
    let mut state = TableState::default();
    let mut show_balance = false;
    state.select(Some(0));

    let mut effects: EffectManager<()> = EffectManager::default();

    // Add a simple fade-in effect
    let fx = fx::fade_to(Color::Cyan, Color::Gray, (1_000, Interpolation::SineIn));
    effects.add_effect(fx);

    let mut last_frame = Instant::now();

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        ui::draw(
            &mut terminal,
            &mut state,
            &accounts,
            &show_balance,
            &mut effects,
            elapsed,
        );

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        let i = state.selected().map_or(0, |i| (i + 1) % accounts.len());
                        state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = state
                            .selected()
                            .map_or(0, |i| (i + accounts.len() - 1) % accounts.len());
                        state.select(Some(i));
                    }
                    KeyCode::Char('b') => show_balance = !show_balance,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn get_accounts() -> Vec<Account> {
    debug!("Fetching accounts");
    let access_token = read_access_token_file().unwrap().access_token;
    let data = api::get_accounts(access_token);
    data.accounts
}
