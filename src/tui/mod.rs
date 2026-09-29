mod app;
mod ui;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::State;
use app::App;

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

        const PAGE: u16 = 10;
        match (key.code, key.modifiers) {
            (KeyCode::Char('q') | KeyCode::Esc, _) => app.should_quit = true,
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.should_quit = true,
            (KeyCode::Down, KeyModifiers::ALT) => app.table.scroll_down_by(PAGE),
            (KeyCode::Up, KeyModifiers::ALT) => app.table.scroll_up_by(PAGE),
            (KeyCode::Down | KeyCode::Char('j'), _) => app.table.select_next(),
            (KeyCode::Up | KeyCode::Char('k'), _) => app.table.select_previous(),
            (KeyCode::Home | KeyCode::Char('g'), _) => app.table.select_first(),
            (KeyCode::End | KeyCode::Char('G'), _) => app.table.select_last(),
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
            (KeyCode::Tab, KeyModifiers::SHIFT) => match app.period {
                crate::models::TimePeriod::Weekly => {
                    app.set_period(state, crate::models::TimePeriod::Yearly)?
                }
                crate::models::TimePeriod::Fortnightly => {
                    app.set_period(state, crate::models::TimePeriod::Weekly)?
                }
                crate::models::TimePeriod::Monthly => {
                    app.set_period(state, crate::models::TimePeriod::Fortnightly)?
                }
                crate::models::TimePeriod::Yearly => {
                    app.set_period(state, crate::models::TimePeriod::Monthly)?
                }
            },
            (KeyCode::Tab, _) => match app.period {
                crate::models::TimePeriod::Weekly => {
                    app.set_period(state, crate::models::TimePeriod::Fortnightly)?
                }
                crate::models::TimePeriod::Fortnightly => {
                    app.set_period(state, crate::models::TimePeriod::Monthly)?
                }
                crate::models::TimePeriod::Monthly => {
                    app.set_period(state, crate::models::TimePeriod::Yearly)?
                }
                crate::models::TimePeriod::Yearly => {
                    app.set_period(state, crate::models::TimePeriod::Weekly)?
                }
            },
            _ => {}
        }
    }
    Ok(())
}
