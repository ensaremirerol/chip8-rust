mod functions;
mod opcode;
pub mod state;

use std::fmt::Display;

use beep::beep;
use opcode::OpCode;
use state::{ChipState, KeyboardHalt};

use crate::terminal::Terminal;

#[derive(Debug)]
enum ChipError {
    InvalidOpcode(u16),
}

impl Display for ChipError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChipError::InvalidOpcode(opcode) => write!(f, "Invalid opcode: {:#X}", opcode),
        }
    }
}

pub struct Chip<'a> {
    clock_speed: u64, // Clock speed in kHz
    state: ChipState,
    terminal: &'a mut Terminal,
    shift_quirk: bool,
}

impl<'a> Chip<'a> {
    pub fn new(clock_speed: u64, terminal: &'a mut Terminal, shift_quirk: bool) -> Self {
        Chip {
            clock_speed,
            state: ChipState::new(),
            terminal,
            shift_quirk,
        }
    }

    fn load_fonts(&mut self) {
        let fonts = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for (i, &font) in fonts.iter().enumerate() {
            self.state.memory[i] = font;
        }
    }

    pub fn reset(&mut self) {
        self.state = ChipState::new();
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.state.load_rom(rom);
        self.load_fonts();
    }

    pub fn set_key(&mut self, new_state: [bool; 16]) {
        self.state.keypad = new_state;
    }

    fn fetch_opcode(&self) -> u16 {
        let pc = self.state.pc as usize;
        let byte1 = self.state.memory[pc] as u16;
        let byte2 = self.state.memory[pc + 1] as u16;

        (byte1 << 8) | byte2
    }

    fn decode_opcode(&self, opcode: u16) -> Result<OpCode, ChipError> {
        use OpCode::*;

        match opcode {
            0x0000..=0x00FF => match opcode & 0x00FF {
                0x00E0 => Ok(CLS),
                0x00EE => Ok(RET),
                0x00C0..=0x00CF => Ok(SCD((opcode & 0x000F) as u8)),
                0x00FB => Ok(SCR),
                0x00FC => Ok(SCL),
                0x00FD => Ok(EXIT),
                0x00FE => Ok(LOW),
                0x00FF => Ok(HIGH),
                _ => Ok(SYSADDR(opcode & 0x0FFF)),
            },
            0x1000..=0x1FFF => Ok(JP(opcode & 0x0FFF)),
            0x2000..=0x2FFF => Ok(CALL(opcode & 0x0FFF)),
            0x3000..=0x3FFF => Ok(SEVxByte(
                ((opcode & 0x0F00) >> 8) as u8,
                (opcode & 0x00FF) as u8,
            )),
            0x4000..=0x4FFF => Ok(SNEVxByte(
                ((opcode & 0x0F00) >> 8) as u8,
                (opcode & 0x00FF) as u8,
            )),
            0x5000..=0x5FFF => Ok(SEVxVy(
                ((opcode & 0x0F00) >> 8) as u8,
                ((opcode & 0x00F0) >> 4) as u8,
            )),
            0x6000..=0x6FFF => Ok(LDVxByte(
                ((opcode & 0x0F00) >> 8) as u8,
                (opcode & 0x00FF) as u8,
            )),
            0x7000..=0x7FFF => Ok(ADDVxByte(
                ((opcode & 0x0F00) >> 8) as u8,
                (opcode & 0x00FF) as u8,
            )),
            0x8000..=0x8FFF => match opcode & 0x000F {
                0x0 => Ok(LDVxVy(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x1 => Ok(ORVxVy(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x2 => Ok(ANDVxVy(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x3 => Ok(XORVxVy(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x4 => Ok(ADDVxVy(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x5 => Ok(SUBVxVy(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x6 => Ok(SHRVyVx(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0x7 => Ok(SUBNVyVx(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                0xE => Ok(SHLVyVx(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                _ => Err(ChipError::InvalidOpcode(opcode)),
            },
            0x9000..=0x9FFF => Ok(SNEVxVy(
                ((opcode & 0x0F00) >> 8) as u8,
                ((opcode & 0x00F0) >> 4) as u8,
            )),
            0xA000..=0xAFFF => Ok(LDI(opcode & 0x0FFF)),
            0xB000..=0xBFFF => Ok(JP0(opcode & 0x0FFF)),
            0xC000..=0xCFFF => Ok(RND(((opcode & 0x0F00) >> 8) as u8, (opcode & 0x00FF) as u8)),
            0xD000..=0xDFFF => match opcode & 0x000F {
                0x0 => Ok(DRWVxVy0(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                )),
                _ => Ok(DRW(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                    (opcode & 0x000F) as u8,
                )),
            },
            0xE000..=0xEFFF => match opcode & 0x00FF {
                0x9E => Ok(SKPVx(((opcode & 0x0F00) >> 8) as u8)),
                0xA1 => Ok(SKNPVx(((opcode & 0x0F00) >> 8) as u8)),
                _ => Err(ChipError::InvalidOpcode(opcode)),
            },

            0xF000..=0xFFFF => match opcode & 0x00FF {
                0x07 => Ok(LDVxDT(((opcode & 0x0F00) >> 8) as u8)),
                0x0A => Ok(LDVxK(((opcode & 0x0F00) >> 8) as u8)),
                0x15 => Ok(LDDTVx(((opcode & 0x0F00) >> 8) as u8)),
                0x18 => Ok(LDSTVx(((opcode & 0x0F00) >> 8) as u8)),
                0x1E => Ok(ADDIVx(((opcode & 0x0F00) >> 8) as u8)),
                0x29 => Ok(LDFVx(((opcode & 0x0F00) >> 8) as u8)),
                0x33 => Ok(LDBVx(((opcode & 0x0F00) >> 8) as u8)),
                0x55 => Ok(LDIVx(((opcode & 0x0F00) >> 8) as u8)),
                0x65 => Ok(LDVxI(((opcode & 0x0F00) >> 8) as u8)),
                _ => Err(ChipError::InvalidOpcode(opcode)),
            },
            _ => Err(ChipError::InvalidOpcode(opcode)),
        }
    }

    fn execute_opcode(&mut self, opcode: OpCode) -> Result<(), String> {
        use OpCode::*;

        match opcode {
            CLS => functions::cls(&mut self.state),
            RET => functions::ret(&mut self.state),
            SYSADDR(addr) => functions::sysaddr(&mut self.state, addr),
            JP(addr) => functions::jp(&mut self.state, addr),
            CALL(addr) => functions::call(&mut self.state, addr),
            SEVxByte(x, byte) => functions::se_vx_byte(&mut self.state, x, byte),
            SNEVxByte(x, byte) => functions::sne_vx_byte(&mut self.state, x, byte),
            SEVxVy(x, y) => functions::se_vx_vy(&mut self.state, x, y),
            LDVxByte(x, byte) => functions::ld_vx_byte(&mut self.state, x, byte),
            ADDVxByte(x, byte) => functions::add_vx_byte(&mut self.state, x, byte),
            LDVxVy(x, y) => functions::ld_vx_vy(&mut self.state, x, y),
            ORVxVy(x, y) => functions::or_vx_vy(&mut self.state, x, y),
            ANDVxVy(x, y) => functions::and_vx_vy(&mut self.state, x, y),
            XORVxVy(x, y) => functions::xor_vx_vy(&mut self.state, x, y),
            ADDVxVy(x, y) => functions::add_vx_vy(&mut self.state, x, y),
            SUBVxVy(x, y) => functions::sub_vx_vy(&mut self.state, x, y),
            SHRVyVx(x, y) => functions::shr(&mut self.state, x, y, self.shift_quirk),
            SUBNVyVx(x, y) => functions::subn_vy_vx(&mut self.state, x, y),
            SHLVyVx(x, y) => functions::shl(&mut self.state, x, y, self.shift_quirk),
            SNEVxVy(x, y) => functions::sne_vx_vy(&mut self.state, x, y),
            LDI(addr) => functions::ld_i(&mut self.state, addr),
            JP0(addr) => functions::jp0(&mut self.state, addr),
            RND(x, byte) => functions::rnd(&mut self.state, x, byte),
            DRW(x, y, n) => functions::drw(&mut self.state, x, y, n),
            SKPVx(x) => functions::skp_vx(&mut self.state, x),
            SKNPVx(x) => functions::sknp_vx(&mut self.state, x),
            LDDTVx(x) => functions::ld_dt_vx(&mut self.state, x),
            LDVxDT(x) => functions::ld_vx_dt(&mut self.state, x),
            LDSTVx(x) => functions::ld_st_vx(&mut self.state, x),
            ADDIVx(x) => functions::add_i_vx(&mut self.state, x),
            LDFVx(x) => functions::ld_f_vx(&mut self.state, x),
            LDBVx(x) => functions::ld_b_vx(&mut self.state, x),
            LDIVx(x) => functions::ld_i_vx(&mut self.state, x),
            LDVxI(x) => functions::ld_vx_i(&mut self.state, x),
            LDRVx(x) => functions::ld_r_vx(&mut self.state, x),
            LDVxR(x) => functions::ld_vx_r(&mut self.state, x),
            LDVxK(x) => functions::ld_vx_k(&mut self.state, x),
            _ => panic!("Unknown opcode, {:?}", opcode),
        }
    }

    fn update_timers(&mut self) {
        let elapsed = self.state.last_timer_update.elapsed().unwrap().as_nanos();
        if elapsed >= 16666666 {
            if self.state.delay_timer > 0 {
                self.state.delay_timer -= 1;
            }
            if self.state.sound_timer > 0 {
                self.state.sound_timer -= 1;
            }
            self.state.last_timer_update = std::time::SystemTime::now();
        }
    }

    fn can_cycle(&self) -> bool {
        if self.state.keyboard_halt != KeyboardHalt::Resume {
            return false;
        }
        let now = std::time::SystemTime::now();
        let elapsed = now
            .duration_since(self.state.last_cycle)
            .unwrap()
            .as_secs_f32();
        elapsed >= 1.0 / self.clock_speed as f32
    }

    fn draw(&mut self) {
        self.terminal.draw(&self.state).unwrap();
    }

    pub fn cycle(&mut self) {
        // Fetch opcode
        let opcode = self.fetch_opcode();

        // Decode opcode
        let decoded = self.decode_opcode(opcode);

        if let Err(err) = decoded {
            panic!("{}", err);
        }

        let decoded = decoded.unwrap();

        // Execute opcode
        let _ = self
            .execute_opcode(decoded)
            .map_err(|err| eprintln!("Error executing opcode: {}", err));

        // Increment program counter
        if !self.state.jump_flag {
            self.state.pc += 2;
        }
        // Wrap pc around if it goes out of bounds
        if self.state.pc >= 4096 {
            self.state.pc -= 4096;
        }
        // Reset jump flag
        self.state.jump_flag = false;
    }

    pub fn run(&mut self) -> String {
        if self.can_cycle() {
            self.cycle();
            self.state.last_cycle = std::time::SystemTime::now();
        }
        if self.state.draw_flag {
            self.draw();
            self.state.draw_flag = false;
        }
        match self.terminal.get_key() {
            Ok(event) => match event {
                crate::terminal::KeyboardEvent::State(state) => self.set_key(state),
                crate::terminal::KeyboardEvent::Exit => return String::from("exit"),
                crate::terminal::KeyboardEvent::Reset => self.reset(),
                crate::terminal::KeyboardEvent::Pause => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            },
            Err(e) => eprintln!("Error getting key: {}", e),
        }

        // Wait for key press
        match self.state.keyboard_halt {
            state::KeyboardHalt::Halt(x) => {
                if self.state.keypad.iter().any(|&key| key) {
                    self.state.v[x as usize] =
                        self.state.keypad.iter().position(|&key| key).unwrap() as u8;
                    self.state.keyboard_halt = state::KeyboardHalt::WaitForRelease(x);
                }
            }
            state::KeyboardHalt::WaitForRelease(x) => {
                if !self.state.keypad[self.state.v[x as usize] as usize] {
                    self.state.keyboard_halt = state::KeyboardHalt::Resume;
                }
            }
            _ => {}
        }

        // Update timers
        self.update_timers();

        if self.state.sound_timer > 0 {
            let _ = beep(50000);
        } else {
            let _ = beep(0);
        }

        self.terminal.draw_key_state(&self.state.keypad).unwrap();
        self.terminal.draw_timers(&self.state).unwrap();

        String::from("Running")
    }
}
