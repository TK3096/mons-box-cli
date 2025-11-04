use std::{
    io::{self, StdoutLock, Write},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

#[derive(Debug)]
pub enum InputEvent {
    Feed,
    Play,
    Sleep,
    Bath,
    Status,
    Quit,
}

pub struct InteractiveMode;

impl InteractiveMode {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Running interactive mode...");

        let mut stdout = io::stdout().lock();
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode().context("Failed to enable raw mode")?;

        let result = self.run_game_loop(&mut stdout);

        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(stdout, LeaveAlternateScreen)?;

        result
    }

    pub fn run_game_loop(&mut self, stdout: &mut StdoutLock) -> Result<()> {
        stdout.queue(Clear(ClearType::All))?;
        stdout.queue(MoveTo(0, 0))?;

        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    if let Event::Key(key_event) = event {
                        if let Some(input_event) = Self::handle_key_event(key_event) {
                            if let InputEvent::Quit = input_event {
                                break;
                            } else {
                                write!(stdout, "Received input event: {:?}\r\n", input_event)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(key_event: KeyEvent) -> Option<InputEvent> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }
            | KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Quit),
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Feed),
            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Play),
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Sleep),
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Bath),
            KeyEvent {
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Status),
            _ => None,
        }
    }
}
