#![feature(asm)]

pub mod basic_block;
mod cpu;

use basic_block::BasicBlock;
use cpu::CPU;
use std::collections::HashMap;

pub struct Emulator {
    basic_blocks: HashMap<u16, BasicBlock>,
    cpu: CPU,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            cpu: Default::default(),
            basic_blocks: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut basic_block = BasicBlock::new();
            let buf: Vec<u8> = vec![0x32];
            basic_block.translate(&buf);
            //basic_block.emit_ret();
            self.basic_blocks.insert(3, basic_block);
            self.cpu.v[0] = 0x2;
            self.cpu.v[12] = 0xff;
            match self.basic_blocks.get(&3) {
                Some(block) => block.execute(&mut self.cpu),
                None => println!("couldn't find block"),
            }
            println!("v[0]={:x}", self.cpu.v[0]);
            println!("v[12]={:x}", self.cpu.v[12]);
            println!("v[0xf]={:x}", self.cpu.v[0xf]);
            break;
        }
    }
}
