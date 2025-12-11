mod cpu;
mod utility;

fn main() {
    let byte: u8 = 0xA5;

    let mut CPU = cpu::build_cpu();

    CPU.set_register(1, 200);
    CPU.set_register(2, 100);
    CPU.write_byte(0x200, 0x81);
    CPU.write_byte(0x201, 0x24);

    // Fetch-Decode-Execute
    for _ in 0..1 {
        CPU.cycle();
    }

    println!("{} {}", CPU.get_register(1), CPU.get_register(0xF))
}
