const NUM_GPRS: usize = 16;
const MEMORY_MAP_SIZE: usize = 4096;

#[derive(Default)]
#[repr(C)]
pub struct CPU {
    pub v: [u8; NUM_GPRS],
    pub i: u16,
    pub pc: u16,
    mem: [u8; MEMORY_MAP_SIZE],
}

impl CPU {
    pub fn fetch_instruction(&mut self) -> u16 {
        let lo = self.mem[self.pc as usize] as u16;
        let hi = self.mem[self.pc + 1 as usize] as u16;
        self.pc += 2;
        return (hi << 8) | lo;
    }
}
