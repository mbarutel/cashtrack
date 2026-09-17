use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Cell, Row, Table},
    Frame,
};
use rust_decimal::Decimal;

use crate::tui::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let header = Row::new(["Date", "Amount", "Category", "Description"])
        .style(Style::new().bold().underlined());

    let rows = app.rows.iter().map(|t| {
        let amount_style = if t.amount < Decimal::ZERO {
            Style::new().fg(Color::Red)
        } else {
            Style::new().fg(Color::Green)
        };

        Row::new([
            Cell::from(t.date.format("%Y-%m-%d").to_string()),
            Cell::from(Text::from(format!("{:.2}", t.amount)).right_aligned()).style(amount_style),
            Cell::from(t.category.as_str()),
            Cell::from(t.description.as_str()),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(Block::bordered().title(format!(" Transactions ({}) ", app.rows.len())))
    .row_highlight_style(Style::new().reversed())
    .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, &mut app.table);
}
