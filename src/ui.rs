use std::{io::Stdout, time::Duration};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};
use tachyonfx::EffectManager;

use crate::models::Account;

pub fn draw(
    terminal: &mut Terminal<CrosstermBackend<&mut Stdout>>,
    state: &mut TableState,
    accounts: &Vec<Account>,
    show_balance: &bool,
    effects: &mut EffectManager<()>,
    elapsed: Duration,
) {
    let _ = terminal.draw(|frame| {
        // Layout with table and help bar
        let frame_area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(frame_area);

        // Create header row
        let header = Row::new(vec!["Account Name", "Balance", "Account Number", "Owner"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        // Create table rows from accounts
        let rows: Vec<Row> = accounts
            .iter()
            .map(|acc| {
                let mut balance = "".to_string();
                if *show_balance {
                    balance = format!("{:.2}", acc.balance);
                }

                Row::new(vec![
                    acc.name.clone(),
                    balance,
                    acc.account_number.clone(),
                    acc.owner
                        .as_ref()
                        .map_or_else(|| "N/A".to_string(), |o| o.name.clone()),
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
            .highlight_symbol("💰 ");

        frame.render_stateful_widget(table, chunks[0], state);

        // Help bar with commands
        let help_text = "Commands: [q] Quit | [b] Toggle Balance | [↑/↓] Navigate";
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(help, chunks[1]);
        effects.process_effects(elapsed.into(), frame.buffer_mut(), frame_area);
    });
}
