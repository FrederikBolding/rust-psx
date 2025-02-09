use std::fs::read;

use cpu::CPU;
use mmu::MMU;

mod cpu;
mod mmu;

const BIOS_PATH: &str = "./static/bios/PSXBIOS.bin";

fn main() {
    let bios = read(BIOS_PATH).ok().unwrap();
    let mmu = MMU::new(bios);
    let mut cpu = CPU::new(mmu);

    for _ in 0..10 {
        cpu.step();
    }
}
