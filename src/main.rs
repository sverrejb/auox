use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use reqwest::blocking::Client;
use std::io;

use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

use crate::{
    config::read_access_token,
    models::{Account, AccountData, accounts},
};

mod auth;
mod config;
mod models;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::get_config();

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
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        ui::draw(&mut terminal, &mut state, &accounts);

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
    let access_token = read_access_token().access_token;

    let client = Client::new();

    let account_response = client
        .get("https://api.sparebank1.no/personal/banking/accounts")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Accept",
            "application/vnd.sparebank1.v1+json; charset=utf-8",
        )
        .send();

    let data: AccountData = match account_response {
        Ok(response) => response.json().expect("Failed to parse JSON"),
        Err(err) => {
            panic!("Paniced: {}", err)
        }
    };

    data.accounts
}
