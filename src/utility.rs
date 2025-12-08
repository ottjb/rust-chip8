pub fn upper_nibble(byte: u8) -> u8 {
    (byte & 0xF0) >> 4
}

pub fn lower_nibble(byte: u8) -> u8 {
    byte & 0x0F
}
