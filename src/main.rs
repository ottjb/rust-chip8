mod cpu;
mod display;
mod utility;

fn main() {
    let mut cpu = cpu::build_cpu();
    let rom_path = "../roms/corax.ch8";
    cpu.load_rom(rom_path);

    for _ in 0..400 {
        cpu.cycle();
    }

    //println!("{:x}", cpu.read_byte(0x019))
}
