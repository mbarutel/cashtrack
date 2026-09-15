# cashtrack → ratatui

Two front-ends (CLI, TUI) over one core. Nothing below the presentation
layer touches stdout or the terminal.

```
cli/ (clap, println!)     tui/ (ratatui, crossterm)
        └──────────┬───────────────┘
              services/   returns values, never prints
              ┌────┴────┐
            db.rs     models/ (Transaction, Report, DateRange, Bank, categorizer)
```

Target module tree:

```
src/
  lib.rs                 run(): None subcommand → tui::run, Some → cli dispatch
  state.rs               State { db, config }
  config.rs
  db.rs                  queries take NaiveDate, rows carry id
  models/
    transaction.rs       Transaction { id, .. }, NewTransaction (no id)
    report.rs
    import.rs            Bank, ParsedRow
    period.rs            TimePeriod, DateRange { start, end } + for(period, today) / prev() / next()
  services/
    transactions.rs      list(state, Option<DateRange>) -> Result<Vec<Transaction>>
    report.rs            build(state, Option<DateRange>) -> Result<Report>
    import.rs            run(state, path, bank) -> Result<ImportSummary>
    categorizer.rs
  cli/
    mod.rs               Cli / Command / TimePeriod args
    commands.rs          list / report / import → services → text::*
    text.rs              print_transactions, print_report, format_decimal
  tui/
    mod.rs               run(state): ratatui::run(|term| loop { draw; poll; update })
    app.rs               App: screen, range, rows, report, TableState, status, should_quit
    event.rs             Event { Key, Tick, Resize } via crossterm::event::poll
    action.rs            Action enum + key_to_action
    update.rs            update(app, action, state) — only place that calls services
    ui/                  render(frame, &mut App); transactions.rs, report.rs, import.rs, help.rs
    theme.rs
```

Decisions: sync loop, no tokio · bare `cashtrack` opens the TUI · Elm-style
Action enum · `Transaction`/`NewTransaction` split rather than `Option<i64>` id.
Only new dependency: `ratatui = "0.30"` (re-exports crossterm).

## Phase 0 — refactor, no behaviour change

Existing `tests/cli.rs` must keep passing throughout.

- [x] `MyResult<T>` → `anyhow::Result<T>`
- [x] Move `State` to `state.rs`; lazy-load db/config per command
- [ ] `Transaction` gains `id: i64`; add `NewTransaction` for inserts; drop `TransactionDbRow` from public surface
- [ ] `models/period.rs`: move `get_dates` out of `list.rs` as `DateRange`, return `NaiveDate`; `Database::list_transactions` takes `NaiveDate`
- [ ] Wire `DateRange` into `list` and `report` (remove hard-coded `2026-08-01..2026-09-01`)
- [ ] `services/`: extract data-fetching from `commands/*`; `import` returns `ImportSummary` instead of printing every row
- [ ] `cli/text.rs`: move `print_transactions`, `print_report`, `format_decimal`
- [ ] Move `categorizer` to `services/categorizer.rs` and re-enable its unit tests

## Phase 1 — thin end-to-end slice

- [ ] Add `ratatui` dependency
- [ ] `Cli.command: Option<Command>`; `None` → `tui::run`
- [ ] `tui/mod.rs` + `app.rs` + `ui/transactions.rs`: table of current month, `q` quits
- [ ] Confirm terminal restores cleanly on quit, error, and panic

## Phase 2 — navigation

- [ ] `event.rs` / `action.rs` / `update.rs` split
- [ ] Period keys `w f m y a`, prev/next range `[ ]`
- [ ] Scrolling `j k` / arrows with `TableState`
- [ ] Report tab (`Tab` to switch) sharing the same `DateRange`
- [ ] Unit tests for `update()` (Action in → App state out)
- [ ] Render tests with `TestBackend` for transactions and report views

## Phase 3 — feedback

- [ ] Status bar: current range, row count, last message
- [ ] Service errors surface in status bar, never crash the TUI
- [ ] Help overlay `?`

## Phase 4 — import from TUI

- [ ] Path text input + bank selector
- [ ] Calls `services::import`, shows `ImportSummary` in status bar
- [ ] Reload rows/report after import

## Phase 5 — editing

- [ ] `Database::update_category(id, category)`
- [ ] Recategorise selected row
- [ ] Search / filter by description or category

## Backlog

- [ ] Background thread for large CSV imports if the UI blocks
- [ ] Edit categories.yaml from the TUI
