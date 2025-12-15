pub struct Display {
    buffer: [[u8; 64]; 32],
}

pub fn build_display() -> Display {
    Display {
        buffer: [[0; 64]; 32],
    }
}

impl Display {
    pub fn get_display(&self) -> [[u8; 64]; 32] {
        self.buffer
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) -> bool {
        let was_on = self.buffer[y][x] == 1;
        self.buffer[y][x] ^= 1;
        was_on
    }

    pub fn clear(&mut self) {
        self.buffer = [[0; 64]; 32]
    }
}
