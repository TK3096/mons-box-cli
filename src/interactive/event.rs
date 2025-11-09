use anyhow::{Context, Result};
use std::{
    io::{self, StdoutLock, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::app_state::monster::Monster;

const TICK_RATE: Duration = Duration::from_millis(60);
const UI_REFRESH_RATE: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub enum GameEvent {
    Input(InputEvent),
    Tick,
}

#[derive(Debug)]
pub enum InputEvent {
    Feed,
    Play,
    Sleep,
    Status,
    Reset,
    Quit,
}

pub struct InteractiveMode {
    monster: Monster,
    should_quit: bool,
    message: Option<String>,
    message_timer: Option<Instant>,
}

impl InteractiveMode {
    pub fn new(monster: Monster) -> Self {
        Self {
            monster,
            should_quit: false,
            message: None,
            message_timer: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
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

        let (sender, receiver) = mpsc::channel();

        let input_sender = sender.clone();

        thread::spawn(move || {
            loop {
                if event::poll(UI_REFRESH_RATE).unwrap_or(false) {
                    if let Ok(event) = event::read() {
                        if let Event::Key(key_event) = event {
                            if let Some(input_event) = Self::handle_key_event(key_event) {
                                if input_sender.send(GameEvent::Input(input_event)).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        let timer_sender = sender;
        thread::spawn(move || {
            loop {
                thread::sleep(TICK_RATE);
                if timer_sender.send(GameEvent::Tick).is_err() {
                    break;
                }
            }
        });

        let mut stdout = io::stdout().lock();
        self.draw_interface(&mut stdout)?;

        while !self.should_quit {
            if let Ok(event) = receiver.recv_timeout(UI_REFRESH_RATE) {
                match event {
                    GameEvent::Tick => {
                        self.update_monster()?;
                    }
                    GameEvent::Input(input_event) => {
                        self.handle_input(input_event)?;
                    }
                }

                self.draw_interface(&mut stdout)?;
            }

            if let Some(timer) = self.message_timer {
                if timer.elapsed() > Duration::from_secs(3) {
                    self.message = None;
                    self.message_timer = None;
                    self.draw_interface(&mut stdout)?;
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
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Status),
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::Reset),
            _ => None,
        }
    }

    fn update_monster(&mut self) -> Result<()> {
        self.monster.update_from_time_passage()?;
        self.monster.save()?;

        if !self.monster.is_alive && self.message.is_none() {
            self.set_message(format!(
                "💀 {} has died! Press 'r' to start over.",
                self.monster.name
            ));
        } else if self.monster.hunger > 90 && self.message.is_none() {
            self.set_message(format!(
                "🚨 {} is starving! Feed them now!",
                self.monster.name
            ));
        } else if self.monster.health < 20 && self.monster.is_alive && self.message.is_none() {
            self.set_message(format!(
                "⚠️ {}'s health is low! Take care of them!",
                self.monster.name
            ));
        }

        Ok(())
    }

    fn handle_input(&mut self, input_event: InputEvent) -> Result<()> {
        let message = match input_event {
            InputEvent::Feed => self.monster.feed(),
            InputEvent::Play => self.monster.play(),
            InputEvent::Sleep => self.monster.toggle_sleep(),
            InputEvent::Status => "📊 Status updated!".to_string(),
            InputEvent::Reset => {
                if !self.monster.is_alive {
                    Monster::reset()?;
                    self.monster = Monster::load_or_create()?;
                    "🔄 Game has been reset! A new monster has been created.".to_string()
                } else {
                    "⚠️ Monster is still alive! Reset only works when monster has died.".to_string()
                }
            }
            InputEvent::Quit => {
                self.should_quit = true;
                return Ok(());
            }
        };

        self.set_message(message);
        self.monster.save()?;

        Ok(())
    }

    fn draw_interface(&self, stdout: &mut StdoutLock) -> Result<()> {
        self.monster.display(stdout)?;

        // Draw message if any
        if let Some(ref message) = self.message {
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            writeln!(stdout)?;
            write!(stdout, "💬 {}\r\n", message)?;
            stdout.queue(ResetColor)?;
        }

        // Draw controls at bottom
        // writeln!(stdout)?;
        write!(stdout, "╭─────────────────────────────────╮\r\n")?;
        write!(stdout, "│            CONTROLS             │\r\n")?;
        write!(stdout, "├─────────────────────────────────┤\r\n")?;
        write!(stdout, "│ [F]eed  [P]lay  [S]leep  [I]nfo │\r\n")?;
        write!(stdout, "│ [R]eset  [Q]uit                │\r\n")?;
        write!(stdout, "╰─────────────────────────────────╯\r\n")?;

        stdout.flush()?;

        Ok(())
    }

    fn set_message(&mut self, message: String) {
        self.message = Some(message);
        self.message_timer = Some(Instant::now());
    }
}
