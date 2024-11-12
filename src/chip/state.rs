use std::{fmt::Display, time::SystemTime};

#[derive(Debug, PartialEq)]
pub enum KeyboardHalt {
    Halt(u8),
    WaitForRelease(u8),
    Resume,
}

#[derive(Debug)]
pub struct ChipState {
    pub memory: [u8; 4096],
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [u8; 64 * 32],
    pub keypad: [bool; 16],
    pub jump_flag: bool,
    pub draw_flag: bool,
    pub last_timer_update: SystemTime,
    pub last_cycle: SystemTime,
    pub keyboard_halt: KeyboardHalt,
}

impl ChipState {
    pub fn new() -> ChipState {
        ChipState {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            display: [0; 64 * 32],
            keypad: [false; 16],
            jump_flag: false,
            draw_flag: false,
            last_timer_update: SystemTime::now(),
            last_cycle: SystemTime::now(),
            keyboard_halt: KeyboardHalt::Resume,
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }
}

impl Display for ChipState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..16 {
            writeln!(f, "V{:X}: {:X}", i, self.v[i])?;
        }
        writeln!(f, "I: {:X}", self.i)?;
        writeln!(f, "PC: {:X}", self.pc)?;
        writeln!(f, "SP: {:X}", self.sp)?;
        writeln!(f, "DT: {:X}", self.delay_timer)?;
        writeln!(f, "ST: {:X}", self.sound_timer)?;
        writeln!(f, "Jump flag: {}", self.jump_flag)
    }
}
