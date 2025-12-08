mod cpu;
mod utility;

fn main() {
    let byte: u8 = 0xA5;

    let mut CPU = cpu::build_cpu();
    let mut memory: [u8; 4096] = [0; 4096];

    // Fetch-Decode-Execute
    for i in 0..5 {
        CPU.cycle();
    }
}
