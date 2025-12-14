pub struct Display {
    display: [[u8; 64]; 32],
}

pub fn build_display() -> Display {
    Display {
        display: [[0; 64]; 32],
    }
}

impl Display {
    pub fn get_display(&self) -> [[u8; 64]; 32] {
        self.display
    }
    pub fn set_pixel(&mut self, x: usize, y: usize) -> bool {
        let x = x % 64;
        let y = y % 32;

        self.display[y][x] ^= 0x1;
        self.display[y][x] == 0
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> u8 {
        self.display[y][x]
    }

    pub fn clear(&mut self) {
        self.display = [[0; 64]; 32]
    }
}
