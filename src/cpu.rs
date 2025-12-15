use std::fs;

use rand::Rng;

use crate::display::{Display, build_display};

const FONT_SET: [u8; 80] = [
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

pub struct Cpu {
    memory: [u8; 4096],
    stack: [u16; 16],
    v_registers: [u8; 16],
    i_register: u16,
    pc: u16,
    sp: usize,
    display: Display,
    display_wait_quirk: bool,
    draw_occurred_this_frame: bool,
    keys: [u8; 16],
    key_pressed_while_waiting: Option<u8>,
    delay_timer: u8,
    sound_timer: u8,
}

pub fn build_cpu() -> Cpu {
    let mut cpu = Cpu {
        memory: [0; 4096],
        stack: [0; 16],
        v_registers: [0; 16],
        i_register: 0,
        pc: 0x200,
        sp: 0,
        display: build_display(),
        display_wait_quirk: true,
        draw_occurred_this_frame: false,
        keys: [0; 16],
        key_pressed_while_waiting: None,
        delay_timer: 0,
        sound_timer: 0,
    };
    cpu.load_font_data();
    cpu
}

impl Cpu {
    pub fn cycle(&mut self) {
        if self.display_wait_quirk && self.draw_occurred_this_frame {
            return;
        }

        let opcode = self.fetch_instruction();
        self.execute_instruction(opcode);
    }

    pub fn fetch_instruction(&self) -> u16 {
        let high_byte: u16 = self.memory[self.pc as usize] as u16;
        let low_byte: u16 = self.memory[self.pc as usize + 1] as u16;
        (high_byte << 8) | low_byte
    }

    pub fn execute_instruction(&mut self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
            (0x1, _, _, _) => self.jump(nnn),
            (0x2, _, _, _) => self.call_subroutine(nnn),
            (0x3, _, _, _) => self.skip_if_vx_equals(x, nn),
            (0x4, _, _, _) => self.skip_if_vx_not_equals(x, nn),
            (0x5, _, _, 0x0) => self.skip_if_vx_equals_vy(x, y),
            (0x6, _, _, _) => self.set_vx(x, nn),
            (0x7, _, _, _) => self.add_to_vx(x, nn),
            (0x8, _, _, 0x0) => self.set_vx_to_vy(x, y),
            (0x8, _, _, 0x1) => self.set_vx_to_vx_or_vy(x, y),
            (0x8, _, _, 0x2) => self.set_vx_to_vx_and_vy(x, y),
            (0x8, _, _, 0x3) => self.set_vx_to_vx_xor_vy(x, y),
            (0x8, _, _, 0x4) => self.add_vy_to_vx(x, y),
            (0x8, _, _, 0x5) => self.sub_vy_from_vx(x, y),
            (0x8, _, _, 0x6) => self.shift_vx_right(x, y),
            (0x8, _, _, 0x7) => self.set_vx_to_vy_minus_vx(x, y),
            (0x8, _, _, 0xE) => self.shift_vx_left(x, y),
            (0x9, _, _, 0x0) => self.skip_if_vx_not_equals_vy(x, y),
            (0xA, _, _, _) => self.set_i(nnn),
            (0xB, _, _, _) => self.jump_to_v0_plus_nnn(nnn),
            (0xC, _, _, _) => self.random(x, nn),
            (0xD, _, _, _) => self.draw_sprite(x, y, n),
            (0xE, _, 0x9, 0xE) => self.skip_if_key_pressed(x),
            (0xE, _, 0xA, 0x1) => self.skip_if_key_not_pressed(x),
            (0xF, _, 0x0, 0x7) => self.set_vx_to_delay_timer(x),
            (0xF, _, 0x0, 0xA) => self.wait_for_key(x),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer(x),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer(x),
            (0xF, _, 0x1, 0xE) => self.add_vx_to_i(x),
            (0xF, _, 0x2, 0x9) => self.set_i_to_sprite(x),
            (0xF, _, 0x3, 0x3) => self.store_bcd(x),
            (0xF, _, 0x5, 0x5) => self.store_registers(x),
            (0xF, _, 0x6, 0x5) => self.load_registers(x),
            _ => eprintln!("Unknown opcode: {:#06x}", opcode),
        }
    }

    pub fn load_rom(&mut self, location: &str) {
        let rom = fs::read(location).unwrap_or_else(|e| {
            panic!(
                "Failed to read ROM file at '{}': {}. Please ensure the file exists.",
                location, e
            );
        });
        self.memory[0x200..(rom.len() + 0x200)].copy_from_slice(&rom[..]);
    }

    pub fn load_font_data(&mut self) {
        self.memory[0x0..FONT_SET.len()].copy_from_slice(&FONT_SET[..]);
    }

    pub fn get_display(&self) -> [[u8; 64]; 32] {
        self.display.get_display()
    }

    pub fn end_frame(&mut self) {
        self.draw_occurred_this_frame = false;
    }

    pub fn key_press(&mut self, key: u8) {
        self.keys[key as usize] = 1
    }

    pub fn key_release(&mut self, key: u8) {
        self.keys[key as usize] = 0
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn clear_screen(&mut self) {
        self.display.clear();
        self.pc += 2
    }

    fn return_from_subroutine(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn jump(&mut self, address: u16) {
        self.pc = address
    }

    fn call_subroutine(&mut self, address: u16) {
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        self.pc = address;
    }

    fn skip_if_vx_equals(&mut self, register: usize, value: u8) {
        self.pc += if self.v_registers[register] == value {
            4
        } else {
            2
        };
    }

    fn skip_if_vx_not_equals(&mut self, register: usize, value: u8) {
        self.pc += if self.v_registers[register] != value {
            4
        } else {
            2
        };
    }

    fn skip_if_vx_not_equals_vy(&mut self, x: usize, y: usize) {
        self.pc += if self.v_registers[x] != self.v_registers[y] {
            4
        } else {
            2
        };
    }

    fn set_vx(&mut self, register: usize, value: u8) {
        self.v_registers[register] = value;
        self.pc += 2
    }

    fn add_to_vx(&mut self, register: usize, value: u8) {
        self.v_registers[register] = self.v_registers[register].wrapping_add(value);
        self.pc += 2
    }

    fn set_vx_to_vy(&mut self, x: usize, y: usize) {
        self.v_registers[x] = self.v_registers[y];
        self.pc += 2
    }

    fn set_vx_to_vx_or_vy(&mut self, x: usize, y: usize) {
        self.v_registers[x] |= self.v_registers[y];
        self.v_registers[0xF] = 0;
        self.pc += 2
    }

    fn set_vx_to_vx_and_vy(&mut self, x: usize, y: usize) {
        self.v_registers[x] &= self.v_registers[y];
        self.v_registers[0xF] = 0;
        self.pc += 2
    }

    fn set_vx_to_vx_xor_vy(&mut self, x: usize, y: usize) {
        self.v_registers[x] ^= self.v_registers[y];
        self.v_registers[0xF] = 0;
        self.pc += 2
    }

    fn add_vy_to_vx(&mut self, x: usize, y: usize) {
        let vx = self.v_registers[x];
        let vy = self.v_registers[y];

        let wrap = vx as u16 + vy as u16 > 255;

        self.v_registers[x] = vx.wrapping_add(vy);
        self.v_registers[0xF] = if wrap { 1 } else { 0 };

        self.pc += 2
    }

    fn sub_vy_from_vx(&mut self, x: usize, y: usize) {
        let vx = self.v_registers[x];
        let vy = self.v_registers[y];

        let no_borrow = vx >= vy;

        self.v_registers[x] = vx.wrapping_sub(vy);
        self.v_registers[0xF] = if no_borrow { 1 } else { 0 };

        self.pc += 2
    }

    fn shift_vx_right(&mut self, x: usize, y: usize) {
        let vy = self.v_registers[y];
        self.v_registers[x] = vy >> 1;
        self.v_registers[0xF] = vy & 1;

        self.pc += 2
    }

    fn set_vx_to_vy_minus_vx(&mut self, x: usize, y: usize) {
        let vx = self.v_registers[x];
        let vy = self.v_registers[y];

        let no_borrow = vy >= vx;

        self.v_registers[x] = vy.wrapping_sub(vx);
        self.v_registers[0xF] = if no_borrow { 1 } else { 0 };

        self.pc += 2
    }

    fn shift_vx_left(&mut self, x: usize, y: usize) {
        let vy = self.v_registers[y];
        self.v_registers[x] = vy << 1;
        self.v_registers[0xF] = vy >> 7;

        self.pc += 2
    }

    fn skip_if_vx_equals_vy(&mut self, x: usize, y: usize) {
        let vx = self.v_registers[x];
        let vy = self.v_registers[y];
        if vx == vy { self.pc += 4 } else { self.pc += 2 }
    }

    fn set_i(&mut self, value: u16) {
        self.i_register = value;
        self.pc += 2
    }

    fn jump_to_v0_plus_nnn(&mut self, value: u16) {
        self.pc = self.v_registers[0] as u16 + value;
    }

    fn random(&mut self, x: usize, mask: u8) {
        let mut rng = rand::rng();
        self.v_registers[x] = rng.random::<u8>() & mask;
        self.pc += 2;
    }

    fn draw_sprite(&mut self, x: usize, y: usize, height: u8) {
        let x_pos = self.v_registers[x] as usize % 64;
        let y_pos = self.v_registers[y] as usize % 32;

        let mut collision = false;

        for row in 0..height {
            let sprite = self.memory[(self.i_register + row as u16) as usize];
            let y = y_pos + row as usize;
            if y >= 32 {
                break;
            }
            for col in 0..8 {
                let x = x_pos + col;
                if x >= 64 {
                    break;
                }
                if (sprite & (0x80 >> col)) == 0 {
                    continue;
                }

                let x_coord = x_pos + col;
                let y_coord = y_pos + row as usize;

                if self.display.set_pixel(x_coord, y_coord) {
                    collision = true;
                }
            }
        }

        self.v_registers[0xF] = collision as u8;
        self.draw_occurred_this_frame = true;
        self.pc += 2
    }

    fn skip_if_key_pressed(&mut self, register: usize) {
        let key = self.v_registers[register] as usize;
        self.pc += if self.keys[key] == 1 { 4 } else { 2 };
    }

    fn skip_if_key_not_pressed(&mut self, register: usize) {
        let key = self.v_registers[register] as usize;
        self.pc += if self.keys[key] == 0 { 4 } else { 2 };
    }

    fn set_vx_to_delay_timer(&mut self, register: usize) {
        self.v_registers[register] = self.delay_timer;
        self.pc += 2;
    }

    fn wait_for_key(&mut self, x: usize) {
        match self.key_pressed_while_waiting {
            None => {
                if let Some(pressed_key) = self.keys.iter().position(|&k| k == 1) {
                    self.key_pressed_while_waiting = Some(pressed_key as u8);
                }
            }
            Some(key) => {
                if self.keys[key as usize] == 0 {
                    self.v_registers[x] = key;
                    self.key_pressed_while_waiting = None;
                    self.pc += 2;
                }
            }
        }
    }

    fn set_delay_timer(&mut self, register: usize) {
        let value = self.v_registers[register];
        self.delay_timer = value;
        self.pc += 2
    }

    fn set_sound_timer(&mut self, register: usize) {
        let value = self.v_registers[register];
        self.sound_timer = value;
        self.pc += 2
    }

    fn add_vx_to_i(&mut self, x: usize) {
        let vx = self.v_registers[x];
        self.i_register += vx as u16;
        self.pc += 2
    }

    fn set_i_to_sprite(&mut self, register: usize) {
        let value = self.v_registers[register];
        self.i_register = (value * 5) as u16;
        self.pc += 2
    }

    fn store_bcd(&mut self, x: usize) {
        let value = self.v_registers[x];

        let hundreds = value / 100;
        let tens = (value / 10) % 10;
        let ones = value % 10;

        let i = self.i_register as usize;
        self.memory[i] = hundreds;
        self.memory[i + 1] = tens;
        self.memory[i + 2] = ones;

        self.pc += 2;
    }

    fn store_registers(&mut self, x: usize) {
        let i = self.i_register as usize;

        for idx in 0..=x {
            self.memory[i + idx] = self.v_registers[idx];
        }

        self.i_register += (x as u16) + 1;

        self.pc += 2
    }

    fn load_registers(&mut self, x: usize) {
        let i = self.i_register as usize;

        for idx in 0..=x {
            self.v_registers[idx] = self.memory[i + idx];
        }

        self.i_register += (x as u16) + 1;

        self.pc += 2
    }
}
