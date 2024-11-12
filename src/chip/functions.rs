use crate::chip::state::ChipState;

use super::state::KeyboardHalt;

pub fn cls(state: &mut ChipState) -> Result<(), String> {
    state.display = [0; 64 * 32];
    Ok(())
}

pub fn ret(state: &mut ChipState) -> Result<(), String> {
    state.pc = state.stack[state.sp as usize];
    state.sp -= 1;
    Ok(())
}

pub fn sysaddr(state: &mut ChipState, addr: u16) -> Result<(), String> {
    state.pc = addr;
    state.jump_flag = true;
    Ok(())
}

pub fn jp(state: &mut ChipState, addr: u16) -> Result<(), String> {
    state.pc = addr;
    state.jump_flag = true;
    Ok(())
}

pub fn call(state: &mut ChipState, addr: u16) -> Result<(), String> {
    state.sp += 1;
    state.stack[state.sp as usize] = state.pc;
    state.pc = addr;
    state.jump_flag = true;
    Ok(())
}

pub fn se_vx_byte(state: &mut ChipState, x: u8, byte: u8) -> Result<(), String> {
    if state.v[x as usize] == byte {
        state.pc += 2;
    }
    Ok(())
}

pub fn sne_vx_byte(state: &mut ChipState, x: u8, byte: u8) -> Result<(), String> {
    if state.v[x as usize] != byte {
        state.pc += 2;
    }
    Ok(())
}

pub fn se_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    if state.v[x as usize] == state.v[y as usize] {
        state.pc += 2;
    }
    Ok(())
}

pub fn ld_vx_byte(state: &mut ChipState, x: u8, byte: u8) -> Result<(), String> {
    state.v[x as usize] = byte;
    Ok(())
}

pub fn add_vx_byte(state: &mut ChipState, x: u8, byte: u8) -> Result<(), String> {
    state.v[x as usize] = state.v[x as usize].wrapping_add(byte);
    Ok(())
}

pub fn ld_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    state.v[x as usize] = state.v[y as usize];
    Ok(())
}

pub fn or_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    state.v[0xF] = 0;
    state.v[x as usize] |= state.v[y as usize];
    Ok(())
}

pub fn and_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    state.v[0xF] = 0;
    state.v[x as usize] &= state.v[y as usize];
    Ok(())
}

pub fn xor_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    state.v[0xF] = 0;
    state.v[x as usize] ^= state.v[y as usize];
    Ok(())
}

pub fn add_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    let (result, overflow) = state.v[x as usize].overflowing_add(state.v[y as usize]);
    state.v[x as usize] = result;
    state.v[0xF] = overflow as u8;
    Ok(())
}

pub fn sub_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    let (result, borrow) = state.v[x as usize].overflowing_sub(state.v[y as usize]);
    state.v[x as usize] = result;
    state.v[0xF] = (!borrow) as u8;
    Ok(())
}

pub fn shr(state: &mut ChipState, x: u8, y: u8, shift_quirk: bool) -> Result<(), String> {
    if !shift_quirk {
        state.v[x as usize] = state.v[y as usize];
    }
    state.v[0xF] = state.v[x as usize] & 0x1;
    state.v[x as usize] >>= 1;
    Ok(())
}

pub fn subn_vy_vx(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    let (result, borrow) = state.v[y as usize].overflowing_sub(state.v[x as usize]);
    state.v[x as usize] = result;
    state.v[0xF] = (!borrow) as u8;
    Ok(())
}

pub fn shl(state: &mut ChipState, x: u8, y: u8, shift_quirk: bool) -> Result<(), String> {
    if !shift_quirk {
        state.v[x as usize] = state.v[y as usize];
    }
    state.v[0xF] = (state.v[x as usize] & 0x80) >> 7;
    state.v[x as usize] <<= 1;
    Ok(())
}

pub fn sne_vx_vy(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    if state.v[x as usize] != state.v[y as usize] {
        state.pc += 2;
    }
    Ok(())
}

pub fn ld_i(state: &mut ChipState, addr: u16) -> Result<(), String> {
    state.i = addr;
    Ok(())
}

pub fn jp0(state: &mut ChipState, addr: u16) -> Result<(), String> {
    state.pc = addr + state.v[0] as u16;
    state.jump_flag = true;
    Ok(())
}

pub fn rnd(state: &mut ChipState, x: u8, byte: u8) -> Result<(), String> {
    state.v[x as usize] = rand::random::<u8>() & byte;
    Ok(())
}

pub fn drw(state: &mut ChipState, x: u8, y: u8, n: u8) -> Result<(), String> {
    state.draw_flag = true;
    let x = state.v[x as usize] as usize;
    let y = state.v[y as usize] as usize;
    let n = n as usize;

    state.v[0xF] = 0;

    for yline in 0..n {
        let pixel = state.memory[state.i as usize + yline];
        for xline in 0..8 {
            if (pixel & (0x80 >> xline)) != 0 {
                let _x = (x + xline) % 64;
                let _y = (y + yline) % 32;
                if state.display[_y * 64 + _x] == 1 {
                    state.v[0xF] = 1;
                }
                state.display[_y * 64 + _x] ^= 1;
            }
        }
    }
    Ok(())
}

pub fn skp_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    if state.keypad[state.v[x as usize] as usize] {
        state.pc += 2;
    }
    Ok(())
}

pub fn sknp_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    if !state.keypad[state.v[x as usize] as usize] {
        state.pc += 2;
    }
    Ok(())
}

pub fn ld_dt_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    state.delay_timer = state.v[x as usize];
    Ok(())
}

pub fn ld_vx_dt(state: &mut ChipState, x: u8) -> Result<(), String> {
    state.v[x as usize] = state.delay_timer;
    Ok(())
}

pub fn ld_st_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    state.sound_timer = state.v[x as usize];
    Ok(())
}

pub fn add_i_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    state.i += state.v[x as usize] as u16;
    Ok(())
}

pub fn ld_f_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    state.i = state.v[x as usize] as u16 * 5;
    Ok(())
}

pub fn ld_b_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    let vx = state.v[x as usize];
    state.memory[state.i as usize] = vx / 100;
    state.memory[state.i as usize + 1] = (vx / 10) % 10;
    state.memory[state.i as usize + 2] = vx % 10;
    Ok(())
}

pub fn ld_i_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    for i in 0..=x {
        state.memory[state.i as usize] = state.v[i as usize];
        state.i += 1;
    }
    Ok(())
}

pub fn ld_vx_i(state: &mut ChipState, x: u8) -> Result<(), String> {
    for i in 0..=x {
        state.v[i as usize] = state.memory[state.i as usize];
        state.i += 1;
    }

    Ok(())
}

pub fn ld_vx_k(state: &mut ChipState, x: u8) -> Result<(), String> {
    state.keyboard_halt = KeyboardHalt::Halt(x);
    Ok(())
}

#[allow(unused_variables)]
pub fn scd(state: &mut ChipState, n: u8) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn scr(state: &mut ChipState) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn scl(state: &mut ChipState) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn exit(state: &mut ChipState) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn low(state: &mut ChipState) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn high(state: &mut ChipState) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn drw_vx_vy_0(state: &mut ChipState, x: u8, y: u8) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn ld_hf_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn ld_r_vx(state: &mut ChipState, x: u8) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}

#[allow(unused_variables)]
pub fn ld_vx_r(state: &mut ChipState, x: u8) -> Result<(), String> {
    Err("Super Chip-48 instruction not implemented".to_string())
}
