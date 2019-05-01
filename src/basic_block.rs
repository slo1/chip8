extern crate assembler;

use crate::cpu::CPU;
use assembler::mnemonic_parameter_types::immediates::Immediate32Bit;
use assembler::mnemonic_parameter_types::immediates::Immediate8Bit;
use assembler::mnemonic_parameter_types::memory::Memory;
use assembler::mnemonic_parameter_types::registers::Register64Bit;
use assembler::mnemonic_parameter_types::registers::Register8Bit;
use assembler::{
    ExecutableAnonymousMemoryMap, InstructionStream, InstructionStreamHints,
};

static CHIP8_REG_TO_X64_REG: [Register8Bit; 11] = [
    Register8Bit::AL,
    Register8Bit::BL,
    Register8Bit::CL,
    Register8Bit::DL,
    Register8Bit::R8B,
    Register8Bit::R9B,
    Register8Bit::R10B,
    Register8Bit::R11B,
    Register8Bit::R12B,
    Register8Bit::R13B,
    Register8Bit::R14B,
];

static mut NUM_SCRATCH_REGISTERS_USED: usize = 0;
static SCRATCH_REGISTERS: [Register8Bit; 3] = [
    Register8Bit::SIL,
    Register8Bit::BPL,
    Register8Bit::DIL,
];

pub struct BasicBlock {
    memory_map: ExecutableAnonymousMemoryMap,
    function: Option<unsafe extern "C" fn()>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self {
            memory_map: ExecutableAnonymousMemoryMap::new(4096, true, true)
                .unwrap(),
            function: None,
        }
    }

    #[cfg(any(target_arch = "x86_64"))]
    pub fn execute(&self, cpu: &mut CPU) {
        if self.function.is_none() {
            panic!("instruction stream should be finalized first")
        }

        unsafe {
            asm!("call *$12"

            : "={al}"(cpu.v[0])
              "={bl}"(cpu.v[1])
              "={cl}"(cpu.v[2])
              "={dl}"(cpu.v[3])
              "={r8b}"(cpu.v[4])
              "={r9b}"(cpu.v[5])
              "={r10b}"(cpu.v[6])
              "={r11b}"(cpu.v[7])
              "={r12b}"(cpu.v[8])
              "={r13b}"(cpu.v[9])
              "={r14b}"(cpu.v[10])
              "={r15}"(&cpu.v[11])

            : "r"(self.function)
              "{al}"(cpu.v[0])
              "{bl}"(cpu.v[1])
              "{cl}"(cpu.v[2])
              "{dl}"(cpu.v[3])
              "{r8b}"(cpu.v[4])
              "{r9b}"(cpu.v[5])
              "{r10b}"(cpu.v[6])
              "{r11b}"(cpu.v[7])
              "{r12b}"(cpu.v[8])
              "{r13b}"(cpu.v[9])
              "{r14b}"(cpu.v[10])
              "{r15}"(&cpu.v[11])
            );
        }
    }

    fn get_x64_reg_from_cpu_struct(
        instruction_stream: &mut InstructionStream,
        reg_num: u8,
    ) -> Register8Bit {
        match reg_num {
            reg_num if reg_num < 11 => CHIP8_REG_TO_X64_REG[reg_num as usize],

            reg_num => {
                let reg;
                unsafe {
                    reg = match NUM_SCRATCH_REGISTERS_USED {
                        x if x < 3 => {
                            NUM_SCRATCH_REGISTERS_USED += 1;
                            SCRATCH_REGISTERS[x]
                        },

                        _ => panic!("no more scratch registers"),
                    };
                }

                instruction_stream.sub_Register64Bit_Immediate8Bit(
                    Register64Bit::RSP,
                    Immediate8Bit::from(1_u8),
                );
                instruction_stream.mov_Any8BitMemory_Register8Bit(
                    Memory::base_64(Register64Bit::RSP),
                    reg,
                );
                let offset = reg_num - 11;
                instruction_stream.mov_Register8Bit_Any8BitMemory(
                    reg,
                    Memory::base_64_displacement(
                        Register64Bit::R15,
                        Immediate32Bit::from(offset),
                    ),
                );
                reg
            },
        }
    }

    fn set_x64_reg_into_cpu_struct(
        instruction_stream: &mut InstructionStream,
        reg_num: u8,
        reg: Register8Bit,
    ) {
        if reg_num < 11 {
            return;
        }

        let offset = reg_num - 11;
        instruction_stream.mov_Any8BitMemory_Register8Bit(
            Memory::base_64_displacement(
                Register64Bit::R15,
                Immediate32Bit::from(offset),
            ),
            reg,
        );
        instruction_stream.mov_Register8Bit_Any8BitMemory(
            reg,
            Memory::base_64(Register64Bit::RSP),
        );
        instruction_stream.add_Register64Bit_Immediate8Bit(
            Register64Bit::RSP,
            Immediate8Bit::from(1_u8),
        );
    }

    fn emit_6xkk(instruction_stream: &mut InstructionStream, x: u8, byte: u8) {
        assert!(x < 16, "x isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);

        let byte = Immediate8Bit::from(byte);
        instruction_stream.mov_Register8Bit_Immediate8Bit(reg_x, byte);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_7xkk(instruction_stream: &mut InstructionStream, x: u8, byte: u8) {
        assert!(x < 16, "x isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);

        let byte = Immediate8Bit::from(byte);
        instruction_stream.add_Register8Bit_Immediate8Bit(reg_x, byte);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy0(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        instruction_stream.mov_Register8Bit_Register8Bit(reg_x, reg_y);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy1(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        instruction_stream.or_Register8Bit_Register8Bit(reg_x, reg_y);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy2(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        instruction_stream.and_Register8Bit_Register8Bit(reg_x, reg_y);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy3(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        instruction_stream.xor_Register8Bit_Register8Bit(reg_x, reg_y);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy4(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        let reg_f = Self::get_x64_reg_from_cpu_struct(instruction_stream, 0xf);
        instruction_stream.add_Register8Bit_Register8Bit(reg_x, reg_y);
        instruction_stream.setc_Register8Bit(reg_f);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, 0xf, reg_f);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy5(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        let reg_f = Self::get_x64_reg_from_cpu_struct(instruction_stream, 0xf);
        instruction_stream.sub_Register8Bit_Register8Bit(reg_x, reg_y);
        instruction_stream.setc_Register8Bit(reg_f);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, 0xf, reg_f);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy6(instruction_stream: &mut InstructionStream, x: u8) {
        assert!(x < 16, "x isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);

        let reg_f = Self::get_x64_reg_from_cpu_struct(instruction_stream, 0xf);
        instruction_stream.shr_Register8Bit_One(reg_x);
        instruction_stream.setc_Register8Bit(reg_f);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, 0xf, reg_f);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xy7(instruction_stream: &mut InstructionStream, x: u8, y: u8) {
        assert!(x < 16, "x isn't less than 16");
        assert!(y < 16, "y isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);
        let reg_y = Self::get_x64_reg_from_cpu_struct(instruction_stream, y);

        let reg_f = Self::get_x64_reg_from_cpu_struct(instruction_stream, 0xf);
        instruction_stream.sub_Register64Bit_Immediate8Bit(
            Register64Bit::RSP,
            Immediate8Bit::from(1_u8),
        );
        instruction_stream.mov_Any8BitMemory_Register8Bit(
            Memory::base_64(Register64Bit::RSP),
            reg_y,
        );
        instruction_stream.sub_Register8Bit_Register8Bit(reg_y, reg_x);
        instruction_stream.seta_Register8Bit(reg_f);
        instruction_stream.mov_Register8Bit_Register8Bit(reg_x, reg_y);
        instruction_stream.mov_Register8Bit_Any8BitMemory(
            reg_y,
            Memory::base_64(Register64Bit::RSP),
        );
        instruction_stream.add_Register64Bit_Immediate8Bit(
            Register64Bit::RSP,
            Immediate8Bit::from(1_u8),
        );
        Self::set_x64_reg_into_cpu_struct(instruction_stream, 0xf, reg_f);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, y, reg_y);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    fn emit_8xye(instruction_stream: &mut InstructionStream, x: u8) {
        assert!(x < 16, "x isn't less than 16");

        let reg_x = Self::get_x64_reg_from_cpu_struct(instruction_stream, x);

        let reg_f = Self::get_x64_reg_from_cpu_struct(instruction_stream, 0xf);
        instruction_stream.shl_Register8Bit_One(reg_x);
        instruction_stream.setc_Register8Bit(reg_f);
        Self::set_x64_reg_into_cpu_struct(instruction_stream, 0xf, reg_f);

        Self::set_x64_reg_into_cpu_struct(instruction_stream, x, reg_x);
    }

    pub fn recompile(&mut self, cpu: &mut CPU) -> usize {
        let hints: InstructionStreamHints = Default::default();
        let mut instruction_stream = self.memory_map.instruction_stream(&hints);
        let func_ptr = instruction_stream.nullary_function_pointer::<()>();

        Self::emit_7xkk(&mut instruction_stream, 0, 0x66);
        instruction_stream.ret();
        instruction_stream.finish();
        self.function = Some(func_ptr);
        3
    }
}
