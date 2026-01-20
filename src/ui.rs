use std::{io::Stdout, time::Duration};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};
use tachyonfx::EffectManager;

use crate::{AppState, View};

pub const MENU_ITEMS: &[(&str, &str, View)] = &[
    ("Transactions", "T", View::Transactions),
    ("Transfer from", "F", View::TransferSelect),
    ("Cancel", "esc", View::Accounts),
];

const MONEYBAG: &str = "💰  ";
const ARROW: &str = "💰➡️";
pub fn draw(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<&mut Stdout>>,
    effects: &mut EffectManager<()>,
    elapsed: Duration,
    q_progress: Option<f32>,
) {
    let _ = terminal.draw(|frame| {
        // Layout with table and help bar
        let frame_area = frame.area();

        match app.view_stack.last() {
            Some(&View::Accounts) => {
                draw_account_view(app, frame, frame_area, "Accounts", MONEYBAG, q_progress);
            }
            Some(&View::Menu) => {
                //we still draw the account view in order to keep it in the background of the menu
                draw_account_view(app, frame, frame_area, "Accounts", MONEYBAG, q_progress);
                draw_menu(app, frame, frame_area);
            }
            Some(&View::Transactions) => {
                draw_transactions_view(app, frame, frame_area, q_progress);
            }
            Some(&View::TransferSelect) => {
                draw_account_view(app, frame, frame_area, "Select target account", ARROW, q_progress);
            }
            Some(&View::TransferModal) => {
                draw_account_view(app, frame, frame_area, "Select target account", ARROW, q_progress);
                draw_transfer_modal(app, frame, frame_area);
            }
            None => {}
        }

        effects.process_effects(elapsed.into(), frame.buffer_mut(), frame_area);
    });
}

fn draw_account_view(
    app: &mut AppState,
    frame: &mut Frame<'_>,
    frame_area: Rect,
    title: &str,
    icon: &str,
    q_progress: Option<f32>,
) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(frame_area);

    // Create header row
    let header = Row::new(vec!["Account Name", "Balance", "Account Number", "Owner"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    // Create table rows from accounts
    let rows: Vec<Row> = app
        .accounts
        .iter()
        .filter(|acc| app.show_credit_card || acc.type_field != "CREDITCARD")
        .map(|acc| {
            let balance = if app.show_balance {
                format!("{:.2}", acc.balance)
            } else {
                String::new()
            };

            Row::new(vec![
                Cell::from(acc.name.as_str()),
                Cell::from(balance),
                Cell::from(acc.account_number.as_str()),
                Cell::from(acc.owner.as_ref().map(|o| o.name.as_str()).unwrap_or("N/A")),
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
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(icon);

    frame.render_stateful_widget(table, chunks[0], &mut app.account_index);

    // Help bar with commands
    let help =
        help_bar("Commands: [Ctrl+C] Quit | [esc] Back | [b] Toggle Balance | [↑/↓] Navigate", q_progress);
    frame.render_widget(help, chunks[1]);
}

fn draw_menu(app: &mut AppState, frame: &mut Frame<'_>, frame_area: Rect) {
    let menu_items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .map(|(label, shortcut, _)| ListItem::new(menu_text(label, shortcut)))
        .collect();

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

fn draw_transfer_modal(app: &mut AppState, frame: &mut Frame<'_>, frame_area: Rect) {
    let block_area = popup_area(frame_area, 60, 45);
    let clear_area = popup_area(frame_area, 65, 50);

    let block = Block::bordered().title("Transfer");

    frame.render_widget(Clear, clear_area);
    frame.render_widget(block.clone(), block_area);

    // Get inner area for content
    let inner_area = block.inner(block_area);

    // Create vertical layout: first row for To/From, second row for Amount input, third row for Message input
    let rows = Layout::vertical([
        Constraint::Length(1), // To/From labels
        Constraint::Length(3), // Amount input
        Constraint::Length(3), // Message input
    ])
    .split(inner_area);

    // First row: To and From labels side by side
    let to_from_chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);

    let from_name = app
        .from_account
        .and_then(|idx| app.accounts.get(idx))
        .map(|acc| acc.name.as_str())
        .unwrap_or("N/A");
    let to_name = app
        .to_account
        .and_then(|idx| app.accounts.get(idx))
        .map(|acc| acc.name.as_str())
        .unwrap_or("N/A");

    let from_label = Paragraph::new(format!("From: {}", from_name));
    let to_label = Paragraph::new(format!("To: {}", to_name));
    frame.render_widget(from_label, to_from_chunks[0]);
    frame.render_widget(to_label, to_from_chunks[1]);

    // Second row: Amount label and input field
    let amount_chunks =
        Layout::horizontal([Constraint::Length(8), Constraint::Min(0)]).split(rows[1]);

    // Vertically center the "Amount:" label with the input box
    let label_rows =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(amount_chunks[0]);

    let amount_label = Paragraph::new("Amount:");
    frame.render_widget(amount_label, label_rows[1]);

    // Render bordered input field for amount
    let width = amount_chunks[1].width.saturating_sub(2);
    let scroll = app.amount_input.visual_scroll(width as usize);
    let amount_style = if app.active_input == crate::TransferInput::Amount {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    let input_widget = Paragraph::new(app.amount_input.value())
        .block(Block::default().borders(Borders::ALL))
        .style(amount_style)
        .scroll((0, scroll as u16));

    frame.render_widget(input_widget, amount_chunks[1]);

    // Third row: Message label and input field
    let message_chunks =
        Layout::horizontal([Constraint::Length(10), Constraint::Min(0)]).split(rows[2]);

    // Vertically center the "Message:" label with the input box
    let message_label_rows =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(message_chunks[0]);

    let message_label = Paragraph::new("Message:");
    frame.render_widget(message_label, message_label_rows[1]);

    // Render bordered input field for message
    let msg_width = message_chunks[1].width.saturating_sub(2);
    let msg_scroll = app.message_input.visual_scroll(msg_width as usize);
    let message_style = if app.active_input == crate::TransferInput::Message {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    let message_widget = Paragraph::new(app.message_input.value())
        .block(Block::default().borders(Borders::ALL))
        .style(message_style)
        .scroll((0, msg_scroll as u16));

    frame.render_widget(message_widget, message_chunks[1]);
}

fn draw_transactions_view(app: &mut AppState, frame: &mut Frame<'_>, frame_area: Rect, q_progress: Option<f32>) {
    // Fullscreen layout for transactions
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(frame_area);

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

    let widths = [
        Constraint::Percentage(15), // Date
        Constraint::Percentage(45), // Description
        Constraint::Percentage(20), // Amount
        Constraint::Percentage(20), // Type
    ];

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

    let help = help_bar("Commands: [Ctrl+C] Quit | [esc] Back | [↑/↓] Navigate", q_progress);
    frame.render_widget(help, chunks[1]);
}

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


fn help_bar(text: &str, q_progress: Option<f32>) -> Paragraph<'_> {
    let display_text = if let Some(progress) = q_progress {
        let bar_width = 20;
        let filled = (bar_width as f32 * progress) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        format!("Hold Q to quit: [{}] {:.1}s / 1.0s", bar, progress * 1.0)
    } else {
        text.to_string()
    };

    Paragraph::new(display_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
}

fn format_timestamp(timestamp_ms: i64) -> String {
    use chrono::{DateTime, Local};

    let timestamp_secs = timestamp_ms / 1000;

    match DateTime::from_timestamp(timestamp_secs, 0) {
        Some(dt) => {
            let local: DateTime<Local> = dt.into();
            local.format("%Y-%m-%d").to_string()
        }
        None => "Invalid date".to_string(),
    }
}
