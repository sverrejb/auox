use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::debug;
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{Terminal, backend::CrosstermBackend, widgets::{ListState, TableState}};

use crate::{fileio::read_access_token_file, models::Account};

mod api;
mod auth;
mod fileio;
mod models;
mod ui;

use tachyonfx::{
    EffectManager, Interpolation,
    fx::{self},
};

pub struct AppState {
    pub account_index: TableState,
    pub menu_index: ListState,
    pub show_balance: bool,
    pub menu_open: bool,
    pub accounts: Vec<Account>
}

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


    let mut effects: EffectManager<()> = EffectManager::default();

    // Add a simple fade-in effect
    let coalesce_in = fx::coalesce((500, Interpolation::QuintIn));
    effects.add_effect(coalesce_in);


    let mut last_frame = Instant::now();
    let mut exiting = false;
    let mut exit_start_time: Option<Instant> = None;
    let exit_duration = Duration::from_millis(500);

    let mut app = AppState {
        account_index:  TableState::new().with_selected(0),
        menu_index:  ListState::default().with_selected(Some(0)),
        menu_open: false,
        show_balance: false,
        accounts: get_accounts()
    };

    let menu_length = 2; //TODO: fix

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        ui::draw(
            &mut app,
            &mut terminal,
            &mut effects,
            elapsed,
        );

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        if !exiting {
                            effects.add_effect(fx::dissolve((500, Interpolation::QuintIn)));
                            exiting = true;
                            exit_start_time = Some(Instant::now());
                        }
                    }
                    KeyCode::Down => {

                        if !app.menu_open {
                            let i = app.account_index.selected().map_or(0, |i| (i + 1) % &app.accounts.len());
                            app.account_index.select(Some(i));
                        }
                        else {
                            let i = app.menu_index.selected().map_or(0, |i| (i + 1) % menu_length);
                            app.menu_index.select(Some(i));
                        }
                        
                    }
                    KeyCode::Up => {
                        if !app.menu_open {
                            let i = app.account_index.selected().map_or(0, |i| (i + app.accounts.len() - 1) % app.accounts.len());
                            app.account_index.select(Some(i));
                        }
                        else {
                            let i = app.menu_index.selected().map_or(0, |i| (i + menu_length - 1) % menu_length);
                            app.menu_index.select(Some(i));
                        }
                    }
                    KeyCode::Enter => {app.menu_open = true},
                    KeyCode::Esc => {app.menu_open = false},
                    KeyCode::Char('b') => app.show_balance = !app.show_balance,
                    _ => {}
                }
            }
        }

        // If exiting and dissolve effect is done, break the loop
        if exiting {
            if let Some(start_time) = exit_start_time {
                if start_time.elapsed() >= exit_duration {
                    break;
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
