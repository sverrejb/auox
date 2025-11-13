use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::debug;
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{ListState, TableState},
};

use crate::{
    models::{Account, Transaction},
};

mod api;
mod auth;
mod fileio;
mod models;
mod ui;

use tachyonfx::{
    EffectManager, Interpolation,
    fx::{self},
};

pub enum View {
    Accounts,
    Menu,
    Transactions,
}

pub struct AppState {
    pub account_index: TableState,
    pub menu_index: ListState,
    pub transaction_index: TableState,
    pub show_balance: bool,
    pub show_credit_card: bool,
    pub accounts: Vec<Account>,
    pub view: View,
    pub transactions: Vec<Transaction>,
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
        account_index: TableState::new().with_selected(0),
        menu_index: ListState::default().with_selected(Some(0)),
        transaction_index: TableState::new().with_selected(0),
        show_balance: false,
        show_credit_card: false,
        accounts: get_accounts(),
        view: View::Accounts,
        transactions: vec!()
    };

    let menu_length = 2; //TODO: fix

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        ui::draw(&mut app, &mut terminal, &mut effects, elapsed);

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, &app.view) {
                    (KeyCode::Down, View::Accounts) => {
                        let i = app
                            .account_index
                            .selected()
                            .map_or(0, |i| (i + 1) % &app.accounts.len());
                        app.account_index.select(Some(i));
                    }
                    (KeyCode::Down, View::Menu) => {
                        let i = app
                            .menu_index
                            .selected()
                            .map_or(0, |i| (i + 1) % menu_length);
                        app.menu_index.select(Some(i));
                    }

                    (KeyCode::Up, View::Accounts) => {
                        let i = app
                            .account_index
                            .selected()
                            .map_or(0, |i| (i + app.accounts.len() - 1) % app.accounts.len());
                        app.account_index.select(Some(i));
                    }
                    (KeyCode::Up, View::Menu) => {
                        let i = app
                            .menu_index
                            .selected()
                            .map_or(0, |i| (i + menu_length - 1) % menu_length);
                        app.menu_index.select(Some(i));
                    }
                    (KeyCode::Down, View::Transactions) => {
                        if !app.transactions.is_empty() {
                            let i = app
                                .transaction_index
                                .selected()
                                .map_or(0, |i| (i + 1) % app.transactions.len());
                            app.transaction_index.select(Some(i));
                        }
                    }
                    (KeyCode::Up, View::Transactions) => {
                        if !app.transactions.is_empty() {
                            let i = app
                                .transaction_index
                                .selected()
                                .map_or(0, |i| (i + app.transactions.len() - 1) % app.transactions.len());
                            app.transaction_index.select(Some(i));
                        }
                    }
                    (KeyCode::Enter, View::Accounts) => app.view = View::Menu,
                    (KeyCode::Enter, View::Menu) => handle_menu_select(&mut app),
                    (KeyCode::Esc, View::Menu) => app.view = View::Accounts,
                    (KeyCode::Esc, View::Transactions) => app.view = View::Accounts,
                    (KeyCode::Char('b'), View::Accounts) => app.show_balance = !app.show_balance,
                    (KeyCode::Char('m'), _) => app.show_credit_card = !app.show_credit_card,
                    //exit command
                    (KeyCode::Char('c'), _) if key.modifiers.contains(KeyModifiers::CONTROL)=> {
                        if !exiting {
                            effects.add_effect(fx::dissolve((500, Interpolation::QuintIn)));
                            exiting = true;
                            exit_start_time = Some(Instant::now());
                        }
                    }
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
    let data = api::get_accounts();
    data.accounts
}

fn get_transactions(account_key: &String) -> Vec<Transaction> {
    debug!("Fetching transactions");
    let data = api::get_transactions(account_key);
    data.transactions
}

fn handle_menu_select(app: &mut AppState) {
    match app.menu_index.selected() {
        Some(0) => {
            let account_key = &app
                .accounts
                .get(app.account_index.selected().unwrap())
                .unwrap()
                .key;
            let transactions = get_transactions(account_key);
            app.transactions = transactions;
            app.view = View::Transactions;
        }
        Some(1) => {
            app.view = View::Accounts;
        }
        _ => {}
    };
}
