use std::fs;

use rand::Rng;

use crate::display::{Display, build_display};

pub struct Cpu {
    memory: [u8; 4096],
    stack: [u16; 16],
    v_registers: [u8; 16],
    i_register: u16,
    pc: u16,
    sp: usize,
    display: Display,
    clipping_quirk: bool,
    display_wait_quirk: bool,
    draw_occurred_this_frame: bool,
    keys: [u8; 16],
    waiting_for_key: Option<usize>,
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
        clipping_quirk: true,
        display_wait_quirk: true,
        draw_occurred_this_frame: false,
        keys: [0; 16],
        waiting_for_key: None,
        key_pressed_while_waiting: None,
        delay_timer: 0,
        sound_timer: 0,
    };
    println!("Display wait quirk: {}", cpu.display_wait_quirk);
    cpu.load_font_data();
    cpu
}

impl Cpu {
    pub fn cycle(&mut self) {
        if self.display_wait_quirk && self.draw_occurred_this_frame {
            return;
        }
        if self.waiting_for_key.is_some() {
            let opcode = self.fetch_instruction();
            self.execute_instruction(opcode);
            return;
        }

        //println!("PC: {:#05x}", self.pc);
        let opcode = self.fetch_instruction();
        //println!("Opcode: {:#06x}", opcode);
        self.execute_instruction(opcode);
        //println!("Delay: {}", self.delay_timer);
    }

    pub fn fetch_instruction(&self) -> u16 {
        let left: u16 = self.memory[self.pc as usize] as u16;
        let right: u16 = self.memory[self.pc as usize + 1] as u16;
        (left << 8) | right
    }

    pub fn execute_instruction(&mut self, opcode: u16) {
        let op = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        match op {
            0x0 => match nn {
                0xE0 => self.clear_screen(),
                0xEE => self.exit_subroutine(),
                _ => println!("Unimplemented opcode: {:#06x}", opcode),
            },
            0x1 => self.jump(nnn),
            0x2 => self.call_subroutine(nnn),
            0x3 => self.jmp_vx_nn(x as usize, nn),
            0x4 => self.jmp_vx_notnn(x as usize, nn),
            0x5 => self.jmp_vx_vy(x as usize, y as usize),
            0x6 => self.ld_vx_nn(x as usize, nn),
            0x7 => self.add_vx_nn(x as usize, nn),
            0x8 => match n {
                0x0 => self.ld_vx_vy(x as usize, y as usize),
                0x1 => self.ld_vx_or_vy(x as usize, y as usize),
                0x2 => self.ld_vx_and_vy(x as usize, y as usize),
                0x3 => self.ld_vx_xor_vy(x as usize, y as usize),
                0x4 => self.add_vx_vy(x as usize, y as usize),
                0x5 => self.sub_vx_vy(x as usize, y as usize),
                0x6 => self.ld_vx_rshift_vy(x as usize, y as usize),
                0x7 => self.sub_vy_vx(x as usize, y as usize),
                0xE => self.ld_vx_lshift_vy(x as usize, y as usize),
                _ => println!("Unimplemented opcode: {:#06x}", opcode),
            },
            0x9 => self.jmp_vx_notvy(x as usize, y as usize),
            0xA => self.ld_i(nnn),
            0xB => self.jmp_v0(nnn),
            0xC => self.rand(x as usize, nn),
            0xD => self.draw_sprite(x as usize, y as usize, n),
            0xE => match n {
                0xE => self.key_pressed(x as usize),
                0x1 => self.key_unpressed(x as usize),
                _ => println!("Unimplemented opcode: {:#06x}", opcode),
            },
            0xF => match nn {
                0x07 => self.store_delay(x as usize),
                0x0A => self.wait_for_key(x as usize),
                0x15 => self.set_delay(x as usize),
                0x18 => self.set_sound(x as usize),
                0x1E => self.add_i_vx(x as usize),
                0x29 => self.ld_i_sprite(x as usize),
                0x33 => self.bcd(x as usize),
                0x55 => self.store_reg(x as usize),
                0x65 => self.ld_reg(x as usize),
                _ => println!("Unimplemented opcode: {:#06x}", opcode),
            },
            _ => println!("Unimplemented opcode: {:#06x}", opcode),
        }
    }

    pub fn stack_push(&mut self) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
    }

    pub fn stack_pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp]
    }

    pub fn set_register(&mut self, register: usize, value: u8) {
        if register <= 0xF {
            self.v_registers[register] = value
        }
    }

    pub fn get_register(&self, register: usize) -> u8 {
        if register <= 0xF {
            self.v_registers[register]
        } else {
            0
        }
    }

    pub fn add_registers(&mut self, reg1: usize, reg2: usize, dest_reg: usize) {
        if reg1 <= 0xF && reg2 <= 0xF && dest_reg <= 0xF {
            self.v_registers[dest_reg] = self.v_registers[reg1] + self.v_registers[reg2]
        }
    }

    pub fn increment_pc(&mut self) {
        self.pc += 2;
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        if address < self.memory.len() {
            self.memory[address] = value;
        }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        if address < self.memory.len() {
            self.memory[address]
        } else {
            0
        }
    }

    pub fn load_rom(&mut self, location: &str) {
        let rom = fs::read(location).unwrap();
        self.memory[0x200..(rom.len() + 0x200)].copy_from_slice(&rom[..]);
    }

    pub fn load_font_data(&mut self) {
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

    fn exit_subroutine(&mut self) {
        self.pc = self.stack_pop();
    }

    fn jump(&mut self, address: u16) {
        self.pc = address
    }

    fn call_subroutine(&mut self, address: u16) {
        self.pc += 2;
        self.stack_push();
        self.pc = address;
    }

    fn jmp_vx_nn(&mut self, register: usize, value: u8) {
        if register <= 0xF {
            let reg_val = self.v_registers[register];
            if reg_val == value {
                self.pc += 4
            } else {
                self.pc += 2
            }
        }
    }

    fn jmp_vx_notnn(&mut self, register: usize, value: u8) {
        if register <= 0xF {
            let reg_val = self.v_registers[register];
            if reg_val != value {
                self.pc += 4
            } else {
                self.pc += 2
            }
        }
    }

    fn jmp_vx_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let x_val = self.v_registers[x];
            let y_val = self.v_registers[y];
            if x_val == y_val {
                self.pc += 4
            } else {
                self.pc += 2
            }
        }
    }

    fn ld_vx_nn(&mut self, register: usize, value: u8) {
        if register <= 0xF {
            self.v_registers[register] = value
        }
        self.pc += 2
    }

    fn add_vx_nn(&mut self, register: usize, value: u8) {
        if register <= 0xF {
            self.v_registers[register] = self.v_registers[register].wrapping_add(value);
        }
        self.pc += 2
    }

    fn ld_vx_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let new = self.v_registers[y];
            self.v_registers[x] = new;
        }
        self.pc += 2
    }

    fn ld_vx_or_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let new = self.v_registers[x] | self.v_registers[y];
            self.v_registers[x] = new;
        }
        self.v_registers[0xF] = 0;
        self.pc += 2
    }

    fn ld_vx_and_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let new = self.v_registers[x] & self.v_registers[y];
            self.v_registers[x] = new;
        }
        self.v_registers[0xF] = 0;
        self.pc += 2
    }

    fn ld_vx_xor_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let new = self.v_registers[x] ^ self.v_registers[y];
            self.v_registers[x] = new;
        }
        self.v_registers[0xF] = 0;
        self.pc += 2
    }

    fn add_vx_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let vx = self.v_registers[x];
            let vy = self.v_registers[y];

            let wrap = vx as u16 + vy as u16 > 255;

            self.v_registers[x] = vx.wrapping_add(vy);
            self.v_registers[0xF] = if wrap { 1 } else { 0 };
        }
        self.pc += 2
    }

    fn sub_vx_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let vx = self.v_registers[x];
            let vy = self.v_registers[y];

            let no_borrow = vx >= vy;

            self.v_registers[x] = vx.wrapping_sub(vy);
            self.v_registers[0xF] = if no_borrow { 1 } else { 0 };
        }
        self.pc += 2
    }

    fn ld_vx_rshift_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let vy = self.v_registers[y];
            self.v_registers[x] = vy >> 1;
            self.v_registers[0xF] = vy & 1;
        }
        self.pc += 2
    }

    fn sub_vy_vx(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let vx = self.v_registers[x];
            let vy = self.v_registers[y];

            let no_borrow = vy >= vx;

            self.v_registers[x] = vy.wrapping_sub(vx);
            self.v_registers[0xF] = if no_borrow { 1 } else { 0 };
        }
        self.pc += 2
    }

    fn ld_vx_lshift_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let vy = self.v_registers[y];
            self.v_registers[x] = vy << 1;
            self.v_registers[0xF] = vy >> 7;
        }
        self.pc += 2
    }

    fn jmp_vx_notvy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let vx = self.v_registers[x];
            let vy = self.v_registers[y];
            if vx != vy { self.pc += 4 } else { self.pc += 2 }
        }
    }

    fn ld_i(&mut self, value: u16) {
        self.i_register = value;
        self.pc += 2
    }

    fn jmp_v0(&mut self, value: u16) {
        self.pc = self.v_registers[0] as u16 + value;
    }

    fn rand(&mut self, x: usize, value: u8) {
        let mut rng = rand::rng();
        let random_byte: u8 = rng.random();
        self.v_registers[x] = random_byte & value;

        self.pc += 2;
    }

    fn draw_sprite(&mut self, x: usize, y: usize, bytes: u8) {
        let vx = self.v_registers[x] as usize % 64;
        let vy = self.v_registers[y] as usize % 32;

        let mut collision = false;

        for i in 0..bytes {
            let sprite = self.memory[(self.i_register + i as u16) as usize];
            let y = vy + i as usize;
            if y >= 32 {
                break;
            }
            for bit in 0..8 {
                let x = vx + bit;
                if x >= 64 {
                    break;
                }
                if (sprite & (0x80 >> bit)) == 0 {
                    continue;
                }

                let px = vx + bit;
                let py = vy + i as usize;
                if self.clipping_quirk && py >= 32 {
                    continue;
                }

                if self.display.set_pixel(px, py) {
                    collision = true;
                }
            }
        }

        if collision {
            self.v_registers[0xF] = 1
        } else {
            self.v_registers[0xF] = 0
        }

        self.draw_occurred_this_frame = true;
        self.pc += 2
    }

    fn key_pressed(&mut self, register: usize) {
        if register <= 0xF {
            let key = self.v_registers[register] as usize;
            if self.keys[key] == 1 {
                self.pc += 4
            } else {
                self.pc += 2
            }
        }
    }

    fn key_unpressed(&mut self, register: usize) {
        if register <= 0xF {
            let key = self.v_registers[register] as usize;
            if self.keys[key] == 0 {
                self.pc += 4
            } else {
                self.pc += 2
            }
        }
    }

    fn store_delay(&mut self, register: usize) {
        self.v_registers[register] = self.delay_timer;
        self.pc += 2;
    }

    fn wait_for_key(&mut self, x: usize) {
        match self.key_pressed_while_waiting {
            None => {
                let pressed_key = self.keys.iter().position(|&k| k == 1);
                if let Some(key) = pressed_key {
                    self.key_pressed_while_waiting = Some(key as u8);
                }
            }
            Some(key) => {
                if self.keys[key as usize] == 0 {
                    self.v_registers[x] = key;
                    self.pc += 2;
                    self.waiting_for_key = None;
                    self.key_pressed_while_waiting = None;
                }
            }
        }
    }

    fn set_delay(&mut self, register: usize) {
        if register <= 0xF {
            let value = self.v_registers[register];
            self.delay_timer = value;
        }
        self.pc += 2
    }

    fn set_sound(&mut self, register: usize) {
        if register <= 0xF {
            let value = self.v_registers[register];
            self.sound_timer = value;
        }
        self.pc += 2
    }

    fn add_i_vx(&mut self, x: usize) {
        if x <= 0xF {
            let vx = self.v_registers[x];

            self.i_register += vx as u16
        }
        self.pc += 2
    }

    fn ld_i_sprite(&mut self, register: usize) {
        if register <= 0xF {
            let value = self.v_registers[register];
            self.i_register = (value * 5) as u16;
        }
        self.pc += 2
    }

    fn bcd(&mut self, x: usize) {
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

    fn store_reg(&mut self, x: usize) {
        let i = self.i_register as usize;

        for idx in 0..=x {
            self.memory[i + idx] = self.v_registers[idx];
        }

        self.i_register += (x as u16) + 1;

        self.pc += 2
    }

    fn ld_reg(&mut self, x: usize) {
        let i = self.i_register as usize;

        for idx in 0..=x {
            self.v_registers[idx] = self.memory[i + idx];
        }

        self.i_register += (x as u16) + 1;

        self.pc += 2
    }
}
