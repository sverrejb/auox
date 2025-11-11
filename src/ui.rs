use std::{io::Stdout, time::Duration};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};
use tachyonfx::EffectManager;

use crate::{models::Account, AppState};

pub fn draw(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<&mut Stdout>>,
    accounts: &[Account],
    effects: &mut EffectManager<()>,
    elapsed: Duration,
) {
    const MONEYBAG: &str = "💰 ";

    let _ = terminal.draw(|frame| {
        // Layout with table and help bar
        let frame_area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
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
            .block(Block::default().borders(Borders::ALL).title("Accounts"))
            .row_highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(MONEYBAG);

        frame.render_stateful_widget(table, chunks[0], &mut app.account_index);

        if app.menu_open {
            let menu_items = vec![
                ListItem::new("Transaction History"),
                ListItem::new("Cancel [esc]"),
            ];

            let list = List::new(menu_items)
                .block(Block::bordered().title("Actions"))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                )
                .highlight_symbol(MONEYBAG);

            let menu_area = popup_area(frame_area, 60, 20);
            let clear_area = popup_area(frame_area, 65, 25);
            frame.render_widget(Clear, clear_area);
            frame.render_stateful_widget(list, menu_area, &mut app.menu_index);
        }

        // Help bar with commands
        let help_text = "Commands: [q] Quit | [b] Toggle Balance | [↑/↓] Navigate";
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(help, chunks[1]);
        effects.process_effects(elapsed.into(), frame.buffer_mut(), frame_area);
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
