use std::{
    error::Error,
    io::{Stdout, Write},
    time::{Duration, SystemTime},
};

use crossterm::{
    cursor::MoveToNextLine,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

use crate::chip::state::ChipState;

pub struct Terminal {
    stdout: Stdout,
    key_state: [SystemTime; 16],
}

pub enum KeyboardEvent {
    State([bool; 16]),
    Exit,
    Reset,
    Pause,
}

const KEY_REPEAT_INTERVAL: u64 = 100;

impl Terminal {
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout(),
            key_state: [SystemTime::now() - Duration::from_millis(KEY_REPEAT_INTERVAL); 16],
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        self.stdout.execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;
        self.stdout.execute(crossterm::cursor::Hide)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Box<dyn Error>> {
        self.stdout.execute(crossterm::cursor::Show)?;
        self.stdout.execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;
        self.stdout.flush()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw(&mut self, state: &ChipState) -> Result<(), Box<dyn Error>> {
        self.stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
        self.stdout.execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;
        self.stdout.flush()?;
        for y in 0..32 {
            for x in 0..64 {
                if state.display[y * 64 + x] == 0 {
                    self.stdout.execute(Print(" "))?;
                } else {
                    self.stdout.execute(Print("█"))?;
                }
            }
            self.stdout.execute(MoveToNextLine(1))?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    pub fn draw_key_state(&mut self, key_state: &[bool; 16]) -> Result<(), Box<dyn Error>> {
        self.stdout.execute(crossterm::cursor::MoveTo(0, 32))?;
        self.stdout.execute(Print("Key state: "))?;
        for (i, key) in key_state.iter().enumerate() {
            self.stdout.execute(Print(format!("{}: ", i)))?;
            self.stdout.execute(Print(if *key { "1 " } else { "0 " }))?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    pub fn draw_timers(&mut self, state: &ChipState) -> Result<(), Box<dyn Error>> {
        self.stdout.execute(crossterm::cursor::MoveTo(0, 33))?;
        self.stdout
            .execute(Print(format!("Delay timer: {} ", state.delay_timer)))?;
        self.stdout.execute(MoveToNextLine(1))?;
        self.stdout
            .execute(Print(format!("Sound timer: {} ", state.sound_timer)))?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn get_key(&mut self) -> Result<KeyboardEvent, Box<dyn Error>> {
        if crossterm::event::poll(std::time::Duration::from_nanos(10))? {
            if let crossterm::event::Event::Key(event) = crossterm::event::read()? {
                // Ctrl-C to exit
                if event.code == crossterm::event::KeyCode::Char('c')
                    && event.modifiers == crossterm::event::KeyModifiers::CONTROL
                {
                    return Ok(KeyboardEvent::Exit);
                }
                match event.code {
                    crossterm::event::KeyCode::Char('o') => return Ok(KeyboardEvent::Exit),
                    crossterm::event::KeyCode::Char('l') => return Ok(KeyboardEvent::Reset),
                    crossterm::event::KeyCode::Char('p') => return Ok(KeyboardEvent::Pause),
                    crossterm::event::KeyCode::Char(key) => {
                        if let Some(index) = "x123qweasdzc4rfv".find(key) {
                            self.key_state[index] = SystemTime::now();
                        }
                    }
                    _ => {}
                }
            }
        }
        // Update key states
        Ok(KeyboardEvent::State(self.get_key_state()))
    }

    fn get_key_state(&mut self) -> [bool; 16] {
        self.key_state
            .iter()
            .map(|t| {
                t.elapsed()
                    .unwrap_or_else(|e| panic!("Failed to get elapsed time: {}", e))
                    .as_millis()
                    < KEY_REPEAT_INTERVAL as u128
            })
            .collect::<Vec<bool>>()
            .as_slice()
            .try_into()
            .unwrap()
    }
}
