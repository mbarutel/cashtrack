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
            handle_event(&mut app)?;
        }
        Ok(())
    })
}

fn handle_event(app: &mut App) -> Result<()> {
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
            _ => {}
        }
    }
    Ok(())
}
