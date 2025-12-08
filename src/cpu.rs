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
    pub fn cycle(&self) {
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

    pub fn execute_instruction(&self, opcode: u16) {
        utility::upper_nibble(0xff);
    }

    pub fn get_register(&self, register: usize) -> u8 {
        if register <= 0xF {
            self.v_registers[register]
        } else {
            0
        }
    }

    pub fn set_register(&mut self, register: usize, value: u8) {
        if register <= 0xF {
            self.v_registers[register] = value
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
}
