use anyhow::Result;
use chrono::{Local, NaiveDate};
use ratatui::widgets::TableState;

use crate::{
    models::{DateRange, TimePeriod, Transaction},
    services, State,
};

pub struct App {
    pub today: NaiveDate,
    pub period: TimePeriod,
    pub range: Option<DateRange>,
    pub rows: Vec<Transaction>,
    pub table: TableState,
    pub should_quit: bool,
}

impl App {
    pub fn new(state: &mut State) -> Result<Self> {
        let today = Local::now().date_naive();
        let period = TimePeriod::Yearly;
        let range = Some(period.range_containing(today));
        let rows = services::transactions(state, range)?;

        let mut table = TableState::default();
        if !rows.is_empty() {
            table.select(Some(0));
        }

        Ok(Self {
            today,
            period,
            range,
            rows,
            table,
            should_quit: false,
        })
    }

    pub fn set_range(&mut self, state: &mut State, range: Option<DateRange>) -> Result<()> {
        self.range = range;
        self.rows = services::transactions(state, range)?;
        self.table = TableState::default();
        if !self.rows.is_empty() {
            self.table.select(Some(0));
        }
        Ok(())
    }

    pub fn set_period(&mut self, state: &mut State, period: TimePeriod) -> Result<()> {
        let anchor = self.range.map_or(self.today, |r| r.start);
        self.period = period;
        self.set_range(state, Some(period.range_containing(anchor)))
    }
}
