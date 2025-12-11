use crate::utility;

pub struct Cpu {
    memory: [u8; 4096],
    v_registers: [u8; 16],
    pc: u16,
}

pub fn build_cpu() -> Cpu {
    Cpu {
        memory: [0; 4096],
        v_registers: [0; 16],
        pc: 0x200,
    }
}

impl Cpu {
    pub fn cycle(&mut self) {
        println!("PC: 0x{:x}", self.pc);
        let opcode = self.fetch_instruction();
        println!("Opcode: 0x{:x}", opcode);
        self.execute_instruction(opcode);
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
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        match op {
            0x6 => self.ld_vx_nn(x as usize, nn),
            0x7 => self.add_vx_nn(x as usize, nn),
            0x8 => match n {
                0x0 => self.ld_vx_vy(x as usize, y as usize),
                0x4 => self.add_vx_vy(x as usize, y as usize),
                _ => println!("Unimplemented opcode: 0x{:x}", opcode),
            },
            _ => println!("Unimplemented opcode: 0x{:x}", opcode),
        }
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

    fn add_vx_vy(&mut self, x: usize, y: usize) {
        if x <= 0xF && y <= 0xF {
            let x_val = self.v_registers[x];
            let y_val = self.v_registers[y];
            if x_val as u16 + y_val as u16 > 255 {
                self.v_registers[0xF] = 1
            }
            self.v_registers[x] = x_val.wrapping_add(y_val);
        }
        self.pc += 2
    }
}
