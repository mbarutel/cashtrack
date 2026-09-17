mod transactions;

use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    Frame,
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let range = match app.range {
        Some(range) => format!("{} · {range}", app.period),
        None => "All time".to_string(),
    };
    frame.render_widget(
        Line::from(vec![
            " cashtrack ".bold().reversed(),
            format!("  {range}").into(),
        ]),
        header,
    );

    transactions::render(frame, body, app);

    frame.render_widget(Line::from(" q quit").style(Style::new().dim()), footer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TimePeriod, Transaction};
    use chrono::NaiveDate;
    use ratatui::{backend::TestBackend, widgets::TableState, Terminal};
    use rust_decimal::Decimal;

    #[test]
    fn renders_transactions_table() {
        let d = |day| NaiveDate::from_ymd_opt(2026, 8, day).unwrap();
        let tx = |id, day, amount: &str, cat: &str, desc: &str| Transaction {
            id,
            date: d(day),
            amount: amount.parse::<Decimal>().unwrap(),
            category: cat.into(),
            description: desc.into(),
            bank: "Commonwealth".into(),
        };

        let mut app = App {
            today: d(5),
            period: TimePeriod::Monthly,
            range: Some(TimePeriod::Monthly.range_containing(d(5))),
            rows: vec![
                tx(1, 4, "-17.50", "Groceries", "WOOLWOTHS 2764 NERANG"),
                tx(2, 1, "1000.0", "Anastasia In", "Transfer from xx9058"),
            ],
            table: TableState::default().with_selected(0),
            should_quit: false,
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 8)).unwrap();
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let out = terminal.backend().to_string();

        assert!(out.contains("Monthly · 01 Aug 2026 - 31 Aug 2026"));
        assert!(out.contains("▶ 2026-08-04"));
        assert!(out.contains("-17.50"));
        assert!(out.contains("Transactions (2)"));
    }
}
