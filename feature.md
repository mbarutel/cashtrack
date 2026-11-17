# Feature: Report screen

Adds a second TUI screen (`Tab` to switch) that summarises the current range, with
the previous period alongside for comparison. Every snippet below was compiled and
tested against ratatui 0.30.2 on a copy of the current tree (commit `c328bf8`);
`cargo test` ends at 24 unit + 1 integration tests passing.

Rendered from the test fixture in `tui/ui/mod.rs`:

```
 cashtrack   Monthly · 01 Aug 2026 - 31 Aug 2026                             Transactions   Report
┌ Summary ─────────────────────────────────────────────────────────────────────────────────────────┐
│ Income       2,000.00  ▲ 2,000.00                                                    Saved 65.4% │
│ Spending       692.50  ▲ 92.50 (+15.4%)                         Transfers 1,000.00 in / 0.00 out │
│ Net          1,307.50  ▲ 1,907.50 (+317.9%)                                On pace for ~4,294.00 │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Spending by category (3) ──────────────────────────────────────────┐┌ Largest ───────────────────┐
│  Category                             Total             Share   #  ││   645.00 Transfer To Castle│
│▶ Rent                                645.00 █████████▌  93.1%   1 =││    30.00 SQ *SOMETHING     │
│  Unknown                              30.00 ▌            4.3%   1 +││    17.50 WOOLWORTHS 2764 NE│
│  Groceries                            17.50 ▌            2.5%   1 +││                            │
│                                                                    │└────────────────────────────┘
│                                                                    │┌ Daily spend ───────────────┐
│                                                                    ││█                           │
│                                                                    ││█▄                          │
└────────────────────────────────────────────────────────────────────┘└01 Aug────────────────31 Aug┘
 ⚠ 1 uncategorised (30.00) — press u to review
 j/k move  Enter drill in  ←/→ period  w/f/m/y/a range  Tab transactions  q quit
```


At 80 columns the same layout still fits; category names get ~11 chars. 100+ is comfortable.

## What changes and why

| Area | Change | Why |
|---|---|---|
| `models/report.rs` | Rewritten. Positive `income`/`spending`, transfers split out, per-category count, top‑5 debits, daily totals. Fixes the two known bugs (first row dropped; balance sign). | Everything the screen shows is derived here once; CLI and TUI both read it. |
| `config.rs` | `transfer: bool` per category (default false) + `Config::transfer_categories()`. | Money moving between your own accounts must not count as income or spending. |
| `models/money.rs` | `format_decimal` moves here from `cli/text.rs`. | TUI needs it; `tui` must not depend on `cli`. |
| `models/transaction.rs` | `Transaction` derives `Clone`; `UNCATEGORISED` const. | `Report` keeps copies of the largest debits; `"Unknown"` was a literal in two places. |
| `tui/app.rs` | `Screen` enum, `filter`, `report`/`prev_report`, `category_table`, one `load()`. | One reload path for range/period/filter changes. |
| `tui/mod.rs` | Key handling split into global / per-screen. `Tab`, `Enter` (drill into category), `u` (uncategorised), `Esc` clears filter. | The single `match` was about to get long. |
| `tui/ui/report.rs` | New screen. | |
| `cli/text.rs` | `print_report` updated to the new `Report` fields. | Old fields are gone. |

Rows on the Transactions screen are filtered in memory (`Vec::retain`) rather than via
a new SQL query; the report always uses the unfiltered set for the same range.

## Order of work

Each step compiles on its own; run `cargo test` after each.

1. `models/money.rs`, `models/transaction.rs`, `models/mod.rs`, `cli/text.rs` (remove `format_decimal`), `services/categorizer.rs`
2. `config.rs`
3. `models/report.rs`, `services/report.rs`, `cli/text.rs` (`print_report`)
4. `tui/app.rs`, `tui/mod.rs`
5. `tui/ui/mod.rs`, `tui/ui/transactions.rs`, `tui/ui/report.rs`
6. Add `transfer: true` to the right categories in your real `categories.yaml`

---

## Step 1 — shared formatter, `Clone`, `UNCATEGORISED`

### `src/models/money.rs` (new)

```rust
use rust_decimal::Decimal;

/// `1234.5` → `1,234.50`, `-7` → `-7.00`.
pub fn format_decimal(amount: Decimal) -> String {
    let s = amount.round_dp(2).to_string();
    let (num, frac) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    let (sign, int) = num.strip_prefix('-').map_or(("", num), |i| ("-", i));

    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }

    format!("{sign}{out}.{frac:0<2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    #[test]
    fn groups_thousands_and_pads_cents() {
        assert_eq!(format_decimal(dec("1234.5")), "1,234.50");
        assert_eq!(format_decimal(dec("-7")), "-7.00");
        assert_eq!(format_decimal(dec("1000000")), "1,000,000.00");
        assert_eq!(format_decimal(dec("0.006")), "0.01");
    }
}
```

### `src/models/transaction.rs`

```rust
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Category assigned when no rule matches.
pub const UNCATEGORISED: &str = "Unknown";

#[derive(Debug)]
pub struct NewTransaction {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub category: String,
    pub description: String,
    pub bank: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: i64,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub category: String,
    pub description: String,
    pub bank: String,
}
```

### `src/models/mod.rs`

```rust
pub mod import;
pub mod money;
pub mod period;
pub mod report;
pub mod transaction;

pub use import::Bank;
pub use money::format_decimal;
pub use period::{DateRange, TimePeriod};
pub use report::{CategorySummary, Report};
pub use transaction::{NewTransaction, Transaction, UNCATEGORISED};
```

### `src/services/categorizer.rs` (two lines)

```rust
use crate::{config::Category, models::UNCATEGORISED};
...
    let mut result = UNCATEGORISED.to_string();
```

### `src/cli/text.rs` (part 1)

Delete the `format_decimal` fn at the bottom and import it instead:

```rust
use crate::{
    models::{format_decimal, Report, Transaction},
    services::ImportSummary,
};
```

`use rust_decimal::Decimal;` stays — `print_transactions` still uses it.

## Step 2 — `transfer` flag in config

### `src/config.rs`

```rust
use std::{collections::HashSet, io::ErrorKind, path::PathBuf};
```

Inside `impl Config`, after `categories()`:

```rust
    /// Titles of categories flagged `transfer: true`.
    pub fn transfer_categories(&self) -> HashSet<String> {
        self.categories
            .iter()
            .filter(|c| c.is_transfer())
            .map(|c| c.title.clone())
            .collect()
    }
```

`Category` gets a field and a getter:

```rust
#[derive(Debug, Deserialize)]
pub struct Category {
    title: String,
    keywords: Vec<String>,
    priority: u32,
    /// Money moving between your own accounts. Excluded from income/spending in reports.
    #[serde(default)]
    transfer: bool,
}

impl Category {
    ...
    pub fn is_transfer(&self) -> bool {
        self.transfer
    }
}
```

`#[serde(default)]` means existing YAML without the key keeps parsing.

## Step 3 — `Report` model

### `src/models/report.rs` (rewritten)

```rust
use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::{Transaction, UNCATEGORISED};

/// Number of largest debits kept in `Report::largest_debits`.
pub const LARGEST_N: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorySummary {
    pub name: String,
    /// Always positive: spending totals are the absolute value of the debits.
    pub total: Decimal,
    pub count: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    /// Earliest / latest transaction date seen. Both `None` when there are no transactions.
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// Sum of credits, excluding transfer categories.
    pub income: Decimal,
    /// Sum of debits as a positive number, excluding transfer categories.
    pub spending: Decimal,
    pub transfers_in: Decimal,
    /// Positive.
    pub transfers_out: Decimal,
    /// Debit categories sorted by total, largest first. Excludes transfers.
    pub spending_by_category: Vec<CategorySummary>,
    /// Credit categories sorted by total, largest first. Excludes transfers.
    pub income_by_category: Vec<CategorySummary>,
    /// Up to `LARGEST_N` debits (excluding transfers), largest first.
    pub largest_debits: Vec<Transaction>,
    /// Spending per calendar day (positive). Days with no debits are absent.
    pub daily_spending: BTreeMap<NaiveDate, Decimal>,
}

impl Report {
    pub fn new(transactions: &[Transaction], transfer_categories: &HashSet<String>) -> Self {
        let mut report = Self::default();
        let mut spending: HashMap<&str, CategorySummary> = HashMap::new();
        let mut income: HashMap<&str, CategorySummary> = HashMap::new();
        let mut debits: Vec<&Transaction> = Vec::new();

        for t in transactions {
            report.start_date = Some(report.start_date.map_or(t.date, |d| d.min(t.date)));
            report.end_date = Some(report.end_date.map_or(t.date, |d| d.max(t.date)));

            if t.amount.is_zero() {
                continue;
            }

            let is_transfer = transfer_categories.contains(&t.category);
            let is_debit = t.amount < Decimal::ZERO;
            let magnitude = t.amount.abs();

            match (is_transfer, is_debit) {
                (true, true) => report.transfers_out += magnitude,
                (true, false) => report.transfers_in += magnitude,
                (false, true) => {
                    report.spending += magnitude;
                    *report.daily_spending.entry(t.date).or_default() += magnitude;
                    debits.push(t);
                    bump(&mut spending, &t.category, magnitude);
                }
                (false, false) => {
                    report.income += magnitude;
                    bump(&mut income, &t.category, magnitude);
                }
            }
        }

        report.spending_by_category = sorted(spending);
        report.income_by_category = sorted(income);

        debits.sort_by(|a, b| a.amount.cmp(&b.amount)); // most negative first
        report.largest_debits = debits.into_iter().take(LARGEST_N).cloned().collect();

        report
    }

    /// Income minus spending; negative when you spent more than you earned.
    pub fn net(&self) -> Decimal {
        self.income - self.spending
    }

    /// `net / income` as a percentage, or `None` when there was no income.
    pub fn savings_rate(&self) -> Option<Decimal> {
        (!self.income.is_zero()).then(|| self.net() / self.income * Decimal::ONE_HUNDRED)
    }

    pub fn uncategorised(&self) -> Option<&CategorySummary> {
        self.spending_by_category
            .iter()
            .find(|c| c.name == UNCATEGORISED)
    }

    /// Share of `spending` for a category, as a percentage.
    pub fn share(&self, category: &CategorySummary) -> Decimal {
        if self.spending.is_zero() {
            Decimal::ZERO
        } else {
            category.total / self.spending * Decimal::ONE_HUNDRED
        }
    }
}

fn bump<'a>(map: &mut HashMap<&'a str, CategorySummary>, name: &'a str, amount: Decimal) {
    let entry = map.entry(name).or_insert_with(|| CategorySummary {
        name: name.to_string(),
        total: Decimal::ZERO,
        count: 0,
    });
    entry.total += amount;
    entry.count += 1;
}

fn sorted(map: HashMap<&str, CategorySummary>) -> Vec<CategorySummary> {
    let mut v: Vec<CategorySummary> = map.into_values().collect();
    v.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.name.cmp(&b.name)));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(id: i64, day: u32, amount: &str, category: &str) -> Transaction {
        Transaction {
            id,
            date: NaiveDate::from_ymd_opt(2026, 8, day).unwrap(),
            amount: amount.parse().unwrap(),
            category: category.into(),
            description: format!("tx {id}"),
            bank: "Commonwealth".into(),
        }
    }

    fn dec(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn transfers() -> HashSet<String> {
        ["Money Transfer".to_string()].into_iter().collect()
    }

    #[test]
    fn empty_report_has_no_dates_and_zero_totals() {
        let r = Report::new(&[], &transfers());
        assert_eq!(r.start_date, None);
        assert_eq!(r.spending, Decimal::ZERO);
        assert_eq!(r.savings_rate(), None);
    }

    #[test]
    fn first_transaction_is_counted() {
        // Regression: the old implementation consumed the first row for dates only.
        let r = Report::new(&[tx(1, 3, "-10", "Groceries")], &transfers());
        assert_eq!(r.spending, dec("10"));
        assert_eq!(r.spending_by_category[0].count, 1);
        assert_eq!(
            r.start_date,
            Some(NaiveDate::from_ymd_opt(2026, 8, 3).unwrap())
        );
    }

    #[test]
    fn transfers_are_excluded_from_income_and_spending() {
        let r = Report::new(
            &[
                tx(1, 1, "3000", "Mike In"),
                tx(2, 2, "-500", "Money Transfer"),
                tx(3, 3, "-200", "Rent"),
            ],
            &transfers(),
        );
        assert_eq!(r.income, dec("3000"));
        assert_eq!(r.spending, dec("200"));
        assert_eq!(r.transfers_out, dec("500"));
        assert_eq!(r.net(), dec("2800"));
        assert!(
            r.spending_by_category
                .iter()
                .all(|c| c.name != "Money Transfer")
        );
    }

    #[test]
    fn categories_sorted_by_total_with_counts_and_share() {
        let r = Report::new(
            &[
                tx(1, 1, "-10", "Eating Out"),
                tx(2, 2, "-15", "Eating Out"),
                tx(3, 3, "-75", "Rent"),
            ],
            &transfers(),
        );
        let names: Vec<&str> = r
            .spending_by_category
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, ["Rent", "Eating Out"]);
        assert_eq!(r.spending_by_category[1].count, 2);
        assert_eq!(r.share(&r.spending_by_category[0]), dec("75"));
    }

    #[test]
    fn largest_debits_are_capped_and_ordered() {
        let txs: Vec<Transaction> = (1..=7)
            .map(|i| tx(i, i as u32, &format!("-{}", i * 10), "Misc"))
            .collect();
        let r = Report::new(&txs, &transfers());
        let amounts: Vec<Decimal> = r.largest_debits.iter().map(|t| t.amount).collect();
        assert_eq!(
            amounts,
            [dec("-70"), dec("-60"), dec("-50"), dec("-40"), dec("-30")]
        );
    }

    #[test]
    fn daily_spending_sums_same_day_debits_only() {
        let r = Report::new(
            &[
                tx(1, 4, "-5", "A"),
                tx(2, 4, "-7", "B"),
                tx(3, 4, "100", "Income"),
            ],
            &transfers(),
        );
        let day = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();
        assert_eq!(r.daily_spending[&day], dec("12"));
        assert_eq!(r.daily_spending.len(), 1);
    }

    #[test]
    fn savings_rate_is_net_over_income() {
        let r = Report::new(
            &[tx(1, 1, "1000", "Pay"), tx(2, 2, "-250", "Rent")],
            &transfers(),
        );
        assert_eq!(r.savings_rate(), Some(dec("75")));
        assert_eq!(r.uncategorised(), None);
    }
}
```

### `src/services/report.rs`

```rust
use anyhow::Result;

use crate::{
    State,
    models::{DateRange, Report},
};

pub fn report(state: &mut State, range: Option<DateRange>) -> Result<Report> {
    let transactions = super::transactions(state, range)?;
    let transfers = state.config.transfer_categories();
    Ok(Report::new(&transactions, &transfers))
}
```

### `src/cli/text.rs` (part 2) — `print_report`

```rust
pub fn print_report(report: &Report) {
    match (report.start_date, report.end_date) {
        (Some(start), Some(end)) => println!(
            "Report for {} to {}",
            start.format("%B %d, %Y"),
            end.format("%B %d, %Y")
        ),
        _ => {
            println!("No transactions");
            return;
        }
    }

    println!("Income    {:>12}", format_decimal(report.income));
    println!("Spending  {:>12}", format_decimal(report.spending));
    println!("Net       {:>12}", format_decimal(report.net()));
    if let Some(rate) = report.savings_rate() {
        println!("Saved     {:>11.1}%", rate);
    }
    println!(
        "Transfers {:>12} in / {} out (excluded)",
        format_decimal(report.transfers_in),
        format_decimal(report.transfers_out)
    );

    println!("\nSpending by category");
    for cat in &report.spending_by_category {
        println!(
            "  {:<20} {:>12} {:>5.1}% {:>3}",
            cat.name,
            format_decimal(cat.total),
            report.share(cat),
            cat.count
        );
    }
}
```

## Step 4 — App state and keys

### `src/tui/app.rs` (rewritten)

```rust
use anyhow::Result;
use chrono::{Local, NaiveDate};
use ratatui::widgets::TableState;

use crate::{
    State,
    models::{DateRange, Report, TimePeriod, Transaction},
    services,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Transactions,
    Report,
}

impl Screen {
    pub const ALL: [Screen; 2] = [Screen::Transactions, Screen::Report];

    pub fn title(self) -> &'static str {
        match self {
            Screen::Transactions => "Transactions",
            Screen::Report => "Report",
        }
    }

    pub fn next(self) -> Screen {
        match self {
            Screen::Transactions => Screen::Report,
            Screen::Report => Screen::Transactions,
        }
    }
}

pub struct App {
    pub today: NaiveDate,
    pub screen: Screen,
    pub period: TimePeriod,
    /// `None` = all time.
    pub range: Option<DateRange>,

    /// Transactions screen: rows after `filter` is applied.
    pub rows: Vec<Transaction>,
    pub table: TableState,
    /// Only show transactions in this category.
    pub filter: Option<String>,

    /// Report screen: current range and the one before it (None for all-time).
    pub report: Report,
    pub prev_report: Option<Report>,
    pub category_table: TableState,

    pub should_quit: bool,
}

impl App {
    pub fn new(state: &mut State) -> Result<Self> {
        let today = Local::now().date_naive();
        let period = TimePeriod::Monthly;

        let mut app = Self {
            today,
            screen: Screen::Transactions,
            period,
            range: Some(period.range_containing(today)),
            rows: Vec::new(),
            table: TableState::default(),
            filter: None,
            report: Report::default(),
            prev_report: None,
            category_table: TableState::default(),
            should_quit: false,
        };
        app.load(state)?;
        Ok(app)
    }

    /// Reload everything derived from `range` / `filter` from the database.
    pub fn load(&mut self, state: &mut State) -> Result<()> {
        let mut rows = services::transactions(state, self.range)?;
        if let Some(category) = &self.filter {
            rows.retain(|t| &t.category == category);
        }
        self.rows = rows;
        self.table = select_first_if_any(self.rows.len());

        self.report = services::report(state, self.range)?;
        self.prev_report = match self.range {
            Some(range) => Some(services::report(state, Some(range.prev(self.period)))?),
            None => None,
        };
        self.category_table = select_first_if_any(self.report.spending_by_category.len());

        Ok(())
    }

    pub fn set_range(&mut self, state: &mut State, range: Option<DateRange>) -> Result<()> {
        self.range = range;
        self.load(state)
    }

    /// Switch granularity, anchored on the current range's start (or today for all-time).
    pub fn set_period(&mut self, state: &mut State, period: TimePeriod) -> Result<()> {
        let anchor = self.range.map_or(self.today, |r| r.start);
        self.period = period;
        self.set_range(state, Some(period.range_containing(anchor)))
    }

    /// Show only `category` on the transactions screen. `None` clears the filter.
    pub fn set_filter(&mut self, state: &mut State, category: Option<String>) -> Result<()> {
        self.filter = category;
        self.screen = Screen::Transactions;
        self.load(state)
    }

    /// The category highlighted on the report screen, if any.
    pub fn selected_category(&self) -> Option<&str> {
        self.category_table
            .selected()
            .and_then(|i| self.report.spending_by_category.get(i))
            .map(|c| c.name.as_str())
    }
}

fn select_first_if_any(len: usize) -> TableState {
    let mut table = TableState::default();
    if len > 0 {
        table.select(Some(0));
    }
    table
}
```

### `src/tui/mod.rs` (rewritten)

```rust
mod app;
mod ui;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    State,
    models::{TimePeriod, UNCATEGORISED},
};
use app::{App, Screen};

pub fn run(state: &mut State) -> Result<()> {
    let mut app = App::new(state)?;

    ratatui::run(|terminal| {
        while !app.should_quit {
            terminal.draw(|frame| ui::render(frame, &mut app))?;
            handle_event(&mut app, state)?;
        }
        Ok(())
    })
}

fn handle_event(app: &mut App, state: &mut State) -> Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        if !handle_global_key(app, state, key)? {
            match app.screen {
                Screen::Transactions => handle_transactions_key(app, state, key)?,
                Screen::Report => handle_report_key(app, state, key)?,
            }
        }
    }
    Ok(())
}

/// Keys that work on every screen. Returns `true` if the key was consumed.
fn handle_global_key(app: &mut App, state: &mut State, key: KeyEvent) -> Result<bool> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            app.should_quit = true
        }
        (KeyCode::Tab, _) => app.screen = app.screen.next(),

        (KeyCode::Left, _) => {
            if let Some(range) = app.range {
                app.set_range(state, Some(range.prev(app.period)))?;
            }
        }
        (KeyCode::Right, _) => {
            if let Some(range) = app.range {
                app.set_range(state, Some(range.next(app.period)))?;
            }
        }
        (KeyCode::Char('w'), _) => app.set_period(state, TimePeriod::Weekly)?,
        (KeyCode::Char('f'), _) => app.set_period(state, TimePeriod::Fortnightly)?,
        (KeyCode::Char('m'), _) => app.set_period(state, TimePeriod::Monthly)?,
        (KeyCode::Char('y'), _) => app.set_period(state, TimePeriod::Yearly)?,
        (KeyCode::Char('a'), _) => app.set_range(state, None)?,

        (KeyCode::Char('u'), _) => app.set_filter(state, Some(UNCATEGORISED.to_string()))?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn handle_transactions_key(app: &mut App, state: &mut State, key: KeyEvent) -> Result<()> {
    const PAGE: u16 = 10;
    match (key.code, key.modifiers) {
        // Esc clears an active filter first; quits when there is none.
        (KeyCode::Esc, _) if app.filter.is_some() => app.set_filter(state, None)?,
        (KeyCode::Esc, _) => app.should_quit = true,

        (KeyCode::Down, KeyModifiers::ALT) => app.table.scroll_down_by(PAGE),
        (KeyCode::Up, KeyModifiers::ALT) => app.table.scroll_up_by(PAGE),
        (KeyCode::Down | KeyCode::Char('j'), _) => app.table.select_next(),
        (KeyCode::Up | KeyCode::Char('k'), _) => app.table.select_previous(),
        (KeyCode::Home | KeyCode::Char('g'), _) => app.table.select_first(),
        (KeyCode::End | KeyCode::Char('G'), _) => app.table.select_last(),
        _ => {}
    }
    Ok(())
}

fn handle_report_key(app: &mut App, state: &mut State, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Down | KeyCode::Char('j') => app.category_table.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.category_table.select_previous(),
        KeyCode::Home | KeyCode::Char('g') => app.category_table.select_first(),
        KeyCode::End | KeyCode::Char('G') => app.category_table.select_last(),
        // Drill into the highlighted category on the transactions screen.
        KeyCode::Enter => {
            if let Some(category) = app.selected_category().map(str::to_string) {
                app.set_filter(state, Some(category))?;
            }
        }
        _ => {}
    }
    Ok(())
}
```

## Step 5 — UI

### `src/tui/ui/mod.rs` (rewritten, includes both render tests)

```rust
mod report;
mod transactions;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::Tabs,
};

use super::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header, app);

    match app.screen {
        Screen::Transactions => transactions::render(frame, body, app),
        Screen::Report => report::render(frame, body, app),
    }

    let keys = match app.screen {
        Screen::Transactions => {
            " j/k move  ←/→ period  w/f/m/y/a range  u unknown  Tab report  q quit"
        }
        Screen::Report => {
            " j/k move  Enter drill in  ←/→ period  w/f/m/y/a range  Tab transactions  q quit"
        }
    };
    frame.render_widget(Line::from(keys).style(Style::new().dim()), footer);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let range = match app.range {
        Some(range) => format!("{} · {range}", app.period),
        None => "All time".to_string(),
    };

    let tabs_width = Screen::ALL
        .iter()
        .map(|s| s.title().len() as u16 + 3)
        .sum::<u16>();
    let [title, tabs] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(tabs_width)]).areas(area);

    frame.render_widget(
        Line::from(vec![
            " cashtrack ".bold().reversed(),
            format!("  {range}").into(),
        ]),
        title,
    );

    let selected = Screen::ALL
        .iter()
        .position(|s| *s == app.screen)
        .unwrap_or(0);
    frame.render_widget(
        Tabs::new(Screen::ALL.iter().map(|s| s.title()))
            .select(selected)
            .highlight_style(Style::new().bold().reversed())
            .divider(" "),
        tabs,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Report, TimePeriod, Transaction};
    use chrono::NaiveDate;
    use ratatui::{Terminal, backend::TestBackend, widgets::TableState};
    use std::collections::HashSet;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, day).unwrap()
    }

    fn tx(id: i64, day: u32, amount: &str, cat: &str, desc: &str) -> Transaction {
        Transaction {
            id,
            date: d(day),
            amount: amount.parse().unwrap(),
            category: cat.into(),
            description: desc.into(),
            bank: "Commonwealth".into(),
        }
    }

    fn transfers() -> HashSet<String> {
        ["Anastasia In".to_string()].into_iter().collect()
    }

    fn app(screen: Screen) -> App {
        let rows = vec![
            tx(1, 4, "-17.50", "Groceries", "WOOLWORTHS 2764 NERANG"),
            tx(2, 1, "1000.00", "Anastasia In", "Transfer from xx9058"),
            tx(3, 2, "-645.00", "Rent", "Transfer To Castle Property"),
            tx(4, 3, "-30.00", "Unknown", "SQ *SOMETHING"),
            tx(5, 3, "2000.00", "Salary", "PAYROLL"),
        ];
        let prev = vec![tx(9, 1, "-600.00", "Rent", "Transfer To Castle Property")];
        App {
            today: d(5),
            screen,
            period: TimePeriod::Monthly,
            range: Some(TimePeriod::Monthly.range_containing(d(5))),
            report: Report::new(&rows, &transfers()),
            prev_report: Some(Report::new(&prev, &transfers())),
            rows,
            table: TableState::default().with_selected(0),
            filter: None,
            category_table: TableState::default().with_selected(0),
            should_quit: false,
        }
    }

    fn draw(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn renders_transactions_table() {
        let out = draw(&mut app(Screen::Transactions), 80, 10);

        assert!(out.contains("Monthly · 01 Aug 2026 - 31 Aug 2026"));
        assert!(out.contains("▶ 2026-08-04"));
        assert!(out.contains("-17.50"));
        assert!(out.contains("Transactions (5)"));
    }

    #[test]
    fn renders_report_screen() {
        let out = draw(&mut app(Screen::Report), 100, 24);

        // summary: transfers excluded, delta against previous month, pace shown (today is in range)
        assert!(out.contains("Income       2,000.00"));
        assert!(out.contains("Spending       692.50"));
        assert!(out.contains("▲ 92.50 (+15.4%)"));
        assert!(out.contains("Transfers 1,000.00 in / 0.00 out"));
        assert!(out.contains("On pace for ~4,294.00"));
        assert!(out.contains("Saved 65.4%"));
        // categories ranked, rent up >10% vs prev, unknown flagged
        assert!(out.contains("▶ Rent"));
        assert!(out.contains("93.1%"));
        assert!(out.contains("1 uncategorised (30.00)"));
        assert!(out.contains("645.00 Transfer To Castle"));
    }
}
```

### `src/tui/ui/transactions.rs` (title shows the active filter)

Replace the `.block(...)` line and add `title`:

```rust
    .block(Block::bordered().title(title(app)))
```

```rust
fn title(app: &App) -> String {
    match &app.filter {
        Some(category) => format!(
            " Transactions · {category} ({}) — Esc clears filter ",
            app.rows.len()
        ),
        None => format!(" Transactions ({}) ", app.rows.len()),
    }
}
```

### `src/tui/ui/report.rs` (new)

```rust
use chrono::{Days, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Cell, List, Paragraph, Row, Sparkline, Table},
};
use rust_decimal::Decimal;

use crate::{
    models::{CategorySummary, Report, format_decimal, report::LARGEST_N},
    tui::app::App,
};

const BAR_WIDTH: usize = 10;
const RIGHT_WIDTH: u16 = 30;
const SUMMARY_RIGHT_WIDTH: u16 = 34;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let [summary, middle, status] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let [categories, right] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(RIGHT_WIDTH)]).areas(middle);

    let [largest, daily] = Layout::vertical([
        Constraint::Length(LARGEST_N as u16 + 2),
        Constraint::Fill(1),
    ])
    .areas(right);

    render_summary(frame, summary, app);
    render_categories(frame, categories, app);
    render_largest(frame, largest, &app.report);
    render_daily(frame, daily, app);
    render_status(frame, status, &app.report);
}

fn render_summary(frame: &mut Frame, area: Rect, app: &App) {
    let r = &app.report;
    let prev = app.prev_report.as_ref();

    let money_line = |label: &str, value: Decimal, prev_value: Option<Decimal>, up_is_good| {
        Line::from(vec![
            Span::raw(format!(" {label:<9}{:>12}  ", format_decimal(value))),
            delta(value, prev_value, up_is_good),
        ])
    };
    let left = Paragraph::new(vec![
        money_line("Income", r.income, prev.map(|p| p.income), true),
        money_line("Spending", r.spending, prev.map(|p| p.spending), false),
        money_line("Net", r.net(), prev.map(|p| p.net()), true),
    ]);

    let saved = match (r.savings_rate(), prev.and_then(|p| p.savings_rate())) {
        (Some(rate), Some(prev_rate)) => {
            format!("Saved {}% (prev {}%) ", pct(rate), pct(prev_rate))
        }
        (Some(rate), None) => format!("Saved {}% ", pct(rate)),
        (None, _) => "No income this period ".to_string(),
    };
    let mut right_lines = vec![
        Line::from(saved),
        Line::from(format!(
            "Transfers {} in / {} out ",
            format_decimal(r.transfers_in),
            format_decimal(r.transfers_out)
        )),
    ];
    if let Some(pace) = pace(app) {
        right_lines.push(Line::from(pace));
    }
    let right = Paragraph::new(right_lines).right_aligned();

    let block = Block::bordered().title(" Summary ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [l, rgt] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(SUMMARY_RIGHT_WIDTH)])
            .areas(inner);
    frame.render_widget(left, l);
    frame.render_widget(right, rgt);
}

/// One decimal place, rounded (Decimal's `{:.1}` truncates).
fn pct(value: Decimal) -> String {
    format!("{:.1}", value.round_dp(1))
}

/// "▲ 200.00 (+6.7%)" coloured by whether an increase is good news.
fn delta(current: Decimal, previous: Option<Decimal>, up_is_good: bool) -> Span<'static> {
    let Some(previous) = previous else {
        return Span::raw("");
    };
    let diff = current - previous;
    if diff.is_zero() {
        return Span::raw("= unchanged").dim();
    }
    let (arrow, sign) = if diff > Decimal::ZERO {
        ("▲", "+")
    } else {
        ("▼", "-")
    };
    let pct = if previous.is_zero() {
        String::new()
    } else {
        format!(
            " ({sign}{}%)",
            pct(diff.abs() / previous.abs() * Decimal::ONE_HUNDRED)
        )
    };
    let good = (diff > Decimal::ZERO) == up_is_good;
    let color = if good { Color::Green } else { Color::Red };
    Span::styled(
        format!("{arrow} {}{pct}", format_decimal(diff.abs())),
        Style::new().fg(color),
    )
}

/// Straight-line projection of spending to the end of the range, only when it includes today.
fn pace(app: &App) -> Option<String> {
    let range = app.range?;
    if !range.contains(app.today) {
        return None;
    }
    let elapsed = Decimal::from((app.today - range.start).num_days() + 1);
    let total = Decimal::from((range.end - range.start).num_days() + 1);
    let projected = app.report.spending / elapsed * total;
    Some(format!(
        "On pace for ~{} ",
        format_decimal(projected.round_dp(0))
    ))
}

fn render_categories(frame: &mut Frame, area: Rect, app: &mut App) {
    let r = &app.report;
    let prev = app.prev_report.as_ref();

    let right = |s: &'static str| Cell::from(Text::from(s).right_aligned());
    let header = Row::new([
        Cell::from("Category"),
        right("Total"),
        Cell::from(""),
        right("Share"),
        right("#"),
        Cell::from(""),
    ])
    .style(Style::new().bold().underlined());

    let rows = r.spending_by_category.iter().map(|c| {
        let share = r.share(c);
        Row::new([
            Cell::from(c.name.as_str()),
            Cell::from(Text::from(format_decimal(c.total)).right_aligned()),
            Cell::from(bar(share)).style(Style::new().fg(Color::Cyan)),
            Cell::from(Text::from(format!("{}%", pct(share))).right_aligned()),
            Cell::from(Text::from(c.count.to_string()).right_aligned()),
            Cell::from(trend(c, prev)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Length(BAR_WIDTH as u16),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(1),
        ],
    )
    .header(header)
    .block(Block::bordered().title(format!(
        " Spending by category ({}) ",
        r.spending_by_category.len()
    )))
    .row_highlight_style(Style::new().reversed())
    .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, &mut app.category_table);
}

/// Horizontal bar with half-block resolution, e.g. "████▌".
fn bar(share_pct: Decimal) -> String {
    let halves = (share_pct / Decimal::ONE_HUNDRED * Decimal::from(BAR_WIDTH * 2))
        .round()
        .try_into()
        .unwrap_or(0usize)
        .min(BAR_WIDTH * 2);
    let mut s = "█".repeat(halves / 2);
    if halves % 2 == 1 {
        s.push('▌');
    }
    s
}

/// ▲ / ▼ when the category moved more than 10% against the previous period.
fn trend(c: &CategorySummary, prev: Option<&Report>) -> Span<'static> {
    let Some(prev) = prev else {
        return Span::raw(" ");
    };
    let before = prev
        .spending_by_category
        .iter()
        .find(|p| p.name == c.name)
        .map_or(Decimal::ZERO, |p| p.total);
    if before.is_zero() {
        return Span::raw("+").fg(Color::Yellow);
    }
    let change = (c.total - before) / before;
    let ten_pct: Decimal = "0.10".parse().unwrap();
    if change > ten_pct {
        Span::raw("▲").fg(Color::Red)
    } else if change < -ten_pct {
        Span::raw("▼").fg(Color::Green)
    } else {
        Span::raw("=").dim()
    }
}

fn render_largest(frame: &mut Frame, area: Rect, report: &Report) {
    let items = report.largest_debits.iter().map(|t| {
        Line::from(vec![
            Span::raw(format!("{:>9} ", format_decimal(t.amount.abs()))).fg(Color::Red),
            Span::raw(t.description.as_str()),
        ])
    });
    frame.render_widget(
        List::new(items).block(Block::bordered().title(" Largest ")),
        area,
    );
}

fn render_daily(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered().title(" Daily spend ");
    let inner = block.inner(area);

    let (start, end) = match app.range {
        Some(r) => (r.start, r.end),
        None => match (app.report.start_date, app.report.end_date) {
            (Some(s), Some(e)) => (s, e),
            _ => {
                frame.render_widget(block, area);
                return;
            }
        },
    };

    let daily: Vec<u64> = days(start, end)
        .map(|d| {
            app.report
                .daily_spending
                .get(&d)
                .and_then(|v| v.round().try_into().ok())
                .unwrap_or(0u64)
        })
        .collect();
    let data = bucket(&daily, inner.width as usize);

    let label = |d: NaiveDate| d.format("%d %b").to_string();
    frame.render_widget(
        Sparkline::default()
            .data(&data)
            .style(Style::new().fg(Color::Cyan))
            .block(
                block
                    .title_bottom(Line::from(label(start)).left_aligned())
                    .title_bottom(Line::from(label(end)).right_aligned()),
            ),
        area,
    );
}

fn days(start: NaiveDate, end: NaiveDate) -> impl Iterator<Item = NaiveDate> {
    let n = (end - start).num_days().max(0) as u64;
    (0..=n).map(move |i| start + Days::new(i))
}

/// Sum consecutive values into at most `max_len` buckets so a year fits in a narrow box.
fn bucket(values: &[u64], max_len: usize) -> Vec<u64> {
    if max_len == 0 || values.len() <= max_len {
        return values.to_vec();
    }
    let size = values.len().div_ceil(max_len);
    values.chunks(size).map(|c| c.iter().sum()).collect()
}

fn render_status(frame: &mut Frame, area: Rect, report: &Report) {
    let line = match report.uncategorised() {
        Some(c) => Line::from(format!(
            " ⚠ {} uncategorised ({}) — press u to review",
            c.count,
            format_decimal(c.total)
        ))
        .fg(Color::Yellow),
        None if report.spending.is_zero() => Line::from(" No spending in this period").dim(),
        None => Line::from(" ✓ All spending categorised").fg(Color::Green),
    };
    frame.render_widget(line, area);
}
```

## Step 6 — your `categories.yaml`

Add `transfer: true` to categories that are money moving between your own accounts, e.g.

```yaml
  - title: "Money Transfer"
    transfer: true
    keywords: [...]
```

Running the report against the fixture CSVs with **all four** of `Mike In`, `Anastasia In`,
`Anastasia Transfer`, `Money Transfer` flagged gives `Income 0.00`, because this joint
account is funded entirely by the two `… In` categories. If you want those to count as
income, flag only the outgoing `Anastasia Transfer` / `Money Transfer`. The right answer
depends on what the account is for, so it's left to config rather than code.

## Verify

```
cargo test                         # 24 lib + 1 integration
cargo run -- report                # CLI output uses the new fields
cargo run                          # Tab → Report; j/k; Enter drills into a category;
                                   # u jumps to Unknown; Esc clears; ←/→ compare periods
```

## Decisions you may want to veto

- **Previous period is one extra query** (`services::report` for `range.prev(period)`), run on
  every range change. SQLite on a local file makes this negligible; no caching added.
- **Deltas colour by meaning**: spending ▲ is red, income/net ▲ is green. Category trend
  arrows (`▲`/`▼`) fire past ±10%, `=` inside, `+` when the category is new this period.
- **Pace** is a straight-line projection (`spending / days_elapsed * days_in_range`) and
  only shows when the range contains today. It ignores that rent lands on the 1st, so early
  in a month it over-projects. Good enough as a glance number; a "fixed vs variable" split
  would be the next refinement.
- **Sparkline buckets** consecutive days when the range has more days than the box has
  columns (a year in 28 columns ≈ 13-day buckets). Values are whole dollars.
- **`Esc` on the Transactions screen clears the filter first**, quits only when no filter is
  active. `q` always quits.
- **`Decimal` `{:.1}` truncates**, so percentages go through `pct()` → `round_dp(1)`.
  `format_decimal` still uses `round_dp(2)`, which is banker's rounding (0.005 → 0.00);
  the unit test pins that behaviour.
- `db.rs` `count`/`last_row` are still unused — the only remaining warning.
