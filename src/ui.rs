use std::{io::Stdout, time::Duration};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};
use tachyonfx::EffectManager;

use crate::{AppState, View};

pub fn draw(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<&mut Stdout>>,
    effects: &mut EffectManager<()>,
    elapsed: Duration,
) {
    const MONEYBAG: &str = "💰 ";

    let _ = terminal.draw(|frame| {
        // Layout with table and help bar
        let frame_area = frame.area();

        match app.view {
            View::Accounts => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(frame_area);

                // Create header row
                let header = Row::new(vec!["Account Name", "Balance", "Account Number", "Owner"])
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );

                // Create table rows from accounts
                let rows: Vec<Row> = app
                    .accounts
                    .iter()
                    .filter(|acc| {
                        // Filter out credit cards if show_credit_card is false
                        app.show_credit_card || acc.type_field != "CREDITCARD"
                    })
                    .map(|acc| {
                        // Only allocate balance string when showing it
                        let balance = if app.show_balance {
                            format!("{:.2}", acc.balance)
                        } else {
                            String::new()
                        };

                        // Use borrowed data where possible, owned for local data
                        Row::new(vec![
                            Cell::from(acc.name.as_str()),
                            Cell::from(balance),
                            Cell::from(acc.account_number.as_str()),
                            Cell::from(
                                acc.owner.as_ref().map(|o| o.name.as_str()).unwrap_or("N/A"),
                            ),
                        ])
                    })
                    .collect();

                // Define column widths
                let widths = [
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ];

                // Create the Table widget
                let table = Table::new(rows, widths)
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title("Accounts"))
                    .row_highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(MONEYBAG);

                frame.render_stateful_widget(table, chunks[0], &mut app.account_index);

                // Help bar with commands
                let help_text = "Commands: [Ctrl+C] Quit | [b] Toggle Balance | [↑/↓] Navigate";
                let help = Paragraph::new(help_text)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                frame.render_widget(help, chunks[1]);
                effects.process_effects(elapsed.into(), frame.buffer_mut(), frame_area);
            }

            View::Menu => {
                let cancel_text = menu_text("Cancel", "esc");
                let transaction_text = menu_text("Transactions", "T");

                let menu_items = vec![ListItem::new(transaction_text), ListItem::new(cancel_text)];

                let list = List::new(menu_items)
                    .block(Block::bordered().title("Actions"))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(MONEYBAG);

                let menu_area = popup_area(frame_area, 60, 20);
                let clear_area = popup_area(frame_area, 65, 25);
                frame.render_widget(Clear, clear_area);
                frame.render_stateful_widget(list, menu_area, &mut app.menu_index);
            }
            View::Transactions => {
                // Fullscreen layout for transactions
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(frame_area);

                // Create header row
                let header = Row::new(vec!["Date", "Description", "Amount", "Type"]).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

                // Create table rows from transactions
                let rows: Vec<Row> = app
                    .transactions
                    .iter()
                    .map(|tx| {
                        // Format date from Unix timestamp (milliseconds)
                        let date_str = format_timestamp(tx.date);

                        // Use cleaned_description if available, otherwise description
                        let desc = tx
                            .cleaned_description
                            .as_ref()
                            .or(tx.description.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("N/A");

                        // Format amount with currency
                        let amount_str = format!("{:.2} {}", tx.amount, tx.currency_code);

                        // Determine color based on amount (positive = green, negative = red)
                        let amount_cell = if tx.amount >= 0.0 {
                            Cell::from(amount_str).style(Style::default().fg(Color::Green))
                        } else {
                            Cell::from(amount_str).style(Style::default().fg(Color::Red))
                        };

                        Row::new(vec![
                            Cell::from(date_str),
                            Cell::from(desc),
                            amount_cell,
                            Cell::from(tx.type_text.as_str()),
                        ])
                    })
                    .collect();

                // Define column widths
                let widths = [
                    Constraint::Percentage(15), // Date
                    Constraint::Percentage(45), // Description
                    Constraint::Percentage(20), // Amount
                    Constraint::Percentage(20), // Type
                ];

                // Create the Table widget
                let table = Table::new(rows, widths)
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title("Transactions"))
                    .row_highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(MONEYBAG);

                frame.render_widget(Clear, frame_area);
                frame.render_stateful_widget(table, chunks[0], &mut app.transaction_index);

                // Help bar for transactions view
                let help_text = "Commands: [Ctrl+C] Quit | [esc] Back to Accounts | [↑/↓] Navigate";
                let help = Paragraph::new(help_text)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                frame.render_widget(help, chunks[1]);
            }
        }
    });
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn menu_text<'a>(option: &'a str, shortcut: &'a str) -> ratatui::prelude::Line<'a> {
    Line::from(vec![
        option.white(),
        Span::raw(" [").gray().dim(),
        shortcut.gray().dim(),
        Span::raw("]").gray().dim(),
    ])
}

/// Formats a Unix timestamp (in milliseconds) to a readable date string
fn format_timestamp(timestamp_ms: i64) -> String {
    use chrono::{DateTime, Local};

    // Convert milliseconds to seconds
    let timestamp_secs = timestamp_ms / 1000;

    // Create DateTime from timestamp
    match DateTime::from_timestamp(timestamp_secs, 0) {
        Some(dt) => {
            // Convert to local time and format
            let local: DateTime<Local> = dt.into();
            local.format("%Y-%m-%d").to_string()
        }
        None => "Invalid date".to_string(),
    }
}
