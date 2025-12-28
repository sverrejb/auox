use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::debug;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{ListState, TableState},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};
use tachyonfx::{
    fx::{self},
    EffectManager, Interpolation,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::models::{Account, Transaction};

mod api;
mod auth;
mod fileio;
mod models;
mod ui;

#[derive(Clone, Copy)]
pub enum View {
    Accounts,
    Menu,
    Transactions,
    TransferSelect,
    TransferModal,
}

pub struct AppState {
    pub account_index: TableState,
    pub menu_index: ListState,
    pub transaction_index: TableState,
    pub show_balance: bool,
    pub show_credit_card: bool,
    pub accounts: Vec<Account>,
    pub view_stack: Vec<View>,
    pub transactions: Vec<Transaction>,
    pub from_account: Option<usize>,
    pub to_account: Option<usize>,
    pub amount_input: Input,
    pub message_input: Input,
    pub active_input: TransferInput,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TransferInput {
    Amount,
    Message,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // Panic hook restores terminal to working state on panic before exiting.
    set_up_panic_hook();

    let config = fileio::get_config_file();
    auth::auth(config.client_id, config.client_secret);

    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    // Effects
    let mut effects: EffectManager<()> = EffectManager::default();
    let coalesce_in = fx::coalesce((500, Interpolation::QuintIn));
    effects.add_effect(coalesce_in);

    let mut last_frame = Instant::now();
    let mut exiting = false;
    let mut exit_start_time: Option<Instant> = None;
    let exit_duration = Duration::from_millis(500);

    // State
    let mut app = AppState {
        account_index: TableState::new().with_selected(0),
        menu_index: ListState::default().with_selected(Some(0)),
        transaction_index: TableState::new().with_selected(0),
        show_balance: false,
        show_credit_card: false,
        accounts: get_accounts(),
        view_stack: vec![View::Accounts],
        transactions: vec![],
        from_account: None,
        to_account: None,
        amount_input: Input::default(),
        message_input: Input::default(),
        active_input: TransferInput::Amount,
    };

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        ui::draw(&mut app, &mut terminal, &mut effects, elapsed);

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, app.view_stack.last()) {
                    (KeyCode::Down, Some(view)) => match view {
                        View::Accounts | View::TransferSelect => {
                            let i = app
                                .account_index
                                .selected()
                                .map_or(0, |i| (i + 1) % app.accounts.len());
                            app.account_index.select(Some(i));
                        }
                        View::Menu => {
                            let i = app
                                .menu_index
                                .selected()
                                .map_or(0, |i| (i + 1) % ui::MENU_ITEMS.len());
                            app.menu_index.select(Some(i));
                        }
                        View::Transactions if !app.transactions.is_empty() => {
                            let i = app
                                .transaction_index
                                .selected()
                                .map_or(0, |i| (i + 1) % app.transactions.len());
                            app.transaction_index.select(Some(i));
                        }
                        _ => {}
                    },
                    (KeyCode::Up, Some(view)) => match view {
                        View::Accounts | View::TransferSelect => {
                            let i = app
                                .account_index
                                .selected()
                                .map_or(0, |i| (i + app.accounts.len() - 1) % app.accounts.len());
                            app.account_index.select(Some(i));
                        }
                        View::Menu => {
                            let i = app.menu_index.selected().map_or(0, |i| {
                                (i + ui::MENU_ITEMS.len() - 1) % ui::MENU_ITEMS.len()
                            });
                            app.menu_index.select(Some(i));
                        }
                        View::Transactions if !app.transactions.is_empty() => {
                            let i = app.transaction_index.selected().map_or(0, |i| {
                                (i + app.transactions.len() - 1) % app.transactions.len()
                            });
                            app.transaction_index.select(Some(i));
                        }
                        _ => {}
                    },
                    (KeyCode::Enter, Some(&View::Accounts)) => app.view_stack.push(View::Menu),
                    (KeyCode::Enter, Some(&View::Menu)) => handle_menu_select(&mut app),
                    (KeyCode::Enter, Some(&View::TransferSelect)) => {
                        app.to_account = app.account_index.selected();
                        app.view_stack.push(View::TransferModal);
                    }
                    (KeyCode::Esc, _) => {
                        if app.view_stack.len() > 1 {
                            app.view_stack.pop();
                        }
                    }
                    (KeyCode::Char('b'), Some(&View::Accounts)) => {
                        app.show_balance = !app.show_balance
                    }
                    (KeyCode::Char('m'), _) => app.show_credit_card = !app.show_credit_card,
                    // Handle input in TransferModal
                    (_, Some(&View::TransferModal)) => {
                        match key.code {
                            KeyCode::Tab => {
                                // Switch between amount and message inputs
                                app.active_input = match app.active_input {
                                    TransferInput::Amount => TransferInput::Message,
                                    TransferInput::Message => TransferInput::Amount,
                                };
                            }
                            KeyCode::Enter => {
                                api::perform_transfer(&mut app);
                            }
                            KeyCode::Char(c) => {
                                match app.active_input {
                                    TransferInput::Amount => {
                                        // Only allow digits and decimal point for amount
                                        if c.is_numeric() || c == '.' || c == ',' {
                                            app.amount_input.handle_event(&Event::Key(key));
                                        }
                                    }
                                    TransferInput::Message => {
                                        // Allow all characters for message
                                        app.message_input.handle_event(&Event::Key(key));
                                    }
                                }
                            }
                            _ => {
                                // Pass all other keys (Backspace, Delete, arrows, etc.) to active input
                                match app.active_input {
                                    TransferInput::Amount => {
                                        app.amount_input.handle_event(&Event::Key(key));
                                    }
                                    TransferInput::Message => {
                                        app.message_input.handle_event(&Event::Key(key));
                                    }
                                }
                            }
                        }
                    }

                    //exit the application
                    (KeyCode::Char('c'), _) if key.modifiers.contains(KeyModifiers::CONTROL) => {
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

pub fn get_accounts() -> Vec<Account> {
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
    //This is horrible, should probably fix.
    let new_view = ui::MENU_ITEMS
        .get(app.menu_index.selected().unwrap())
        .unwrap()
        .2;

    match new_view {
        View::Accounts => {}
        View::Transactions => {
            let account_key = &app
                .accounts
                .get(app.account_index.selected().unwrap())
                .unwrap()
                .key;
            let transactions = get_transactions(account_key);
            app.transactions = transactions;
        }
        View::TransferSelect => {
            // Save the currently selected account as the from_account
            app.from_account = app.account_index.selected();
        }
        View::TransferModal => {}
        View::Menu => {}
    }
    app.view_stack.push(new_view);
}

fn set_up_panic_hook() {
    // Setup panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        // Call the original panic hook
        original_hook(panic_info);
    }));
}
