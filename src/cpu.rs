// src/cpu.rs
use crate::device::Device;
use crate::memory::Memory;

#[derive(Debug)]
pub struct CPU {
    pub regs: [u8; 4], // R0..R3
    pub pc: usize,
    pub z: bool,
    pub mem: Memory,
    pub cycles: u64,
    pub halted: bool,
    devices: Vec<Box<dyn Device>>,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            regs: [0; 4],
            pc: 0,
            z: false,
            mem: Memory::new(),
            cycles: 0,
            halted: false,
            devices: Vec::new(),
        }
    }

    pub fn attach_device(&mut self, dev: Box<dyn Device>) {
        self.devices.push(dev);
    }

    pub fn load(&mut self, program: &[u8], addr: usize) {
        let end = addr + program.len();
        self.mem.write_bytes(addr, program);
        self.pc = addr;
    }

    fn fetch(&mut self) -> u8 {
        let b = self.mem.read(self.pc);
        self.pc = (self.pc + 1) % self.mem.size();
        b
    }

    /// Execute a single instruction (decode + execute) and return the
    /// number of cycles the instruction requires.
    /// This does NOT advance the device ticks or the CPU's cycle counter.
    pub fn step_instruction(&mut self) -> u64 {
        if self.halted {
            return 0;
        }

        let opcode = self.fetch();

        match opcode {
            // LDI reg, imm  => 0x10 | reg  imm   (2 cycles)
            op if (op & 0xF0) == 0x10 => {
                let reg = (op & 0x03) as usize;
                let imm = self.fetch();
                self.regs[reg] = imm;
                self.z = self.regs[reg] == 0;
                2
            }

            // ADD reg, reg => 0x20 | dest  src   (3 cycles)
            op if (op & 0xF0) == 0x20 => {
                let dest = (op & 0x03) as usize;
                let src = (self.fetch() & 0x03) as usize;
                let (res, _) = self.regs[dest].overflowing_add(self.regs[src]);
                self.regs[dest] = res;
                self.z = res == 0;
                3
            }

            // SUB reg, reg => 0x21 | dest src   (3 cycles)
            op if (op & 0xF0) == 0x21 => {
                let dest = (op & 0x03) as usize;
                let src = (self.fetch() & 0x03) as usize;
                let (res, _) = self.regs[dest].overflowing_sub(self.regs[src]);
                self.regs[dest] = res;
                self.z = res == 0;
                3
            }

            // LOAD dest, addr => 0x30 | dest  addr  (4 cycles)
            op if (op & 0xF0) == 0x30 => {
                let dest = (op & 0x03) as usize;
                let addr = self.fetch() as usize;
                self.regs[dest] = self.mem.read(addr);
                self.z = self.regs[dest] == 0;
                4
            }

            // STORE src, addr => 0x31 | src addr  (4 cycles)
            op if (op & 0xF0) == 0x31 => {
                let src = (op & 0x03) as usize;
                let addr = self.fetch() as usize;
                self.mem.write(addr, self.regs[src]);
                4
            }

            // JMP addr => 0x40 addr  (3 cycles)
            0x40 => {
                let addr = self.fetch() as usize;
                self.pc = addr % self.mem.size();
                3
            }

            // JZ reg, addr => 0x41 | reg addr  (3 cycles)
            op if (op & 0xF0) == 0x41 => {
                let reg = (op & 0x03) as usize;
                let addr = self.fetch() as usize;
                if self.regs[reg] == 0 {
                    self.pc = addr % self.mem.size();
                }
                3
            }

            // OUT reg => 0x50 | reg  (4 cycles) - prints decimal + newline
            op if (op & 0xF0) == 0x50 => {
                let reg = (op & 0x03) as usize;
                println!("{}", self.regs[reg]);
                4
            }

            // HLT => 0xFF  (1 cycle)
            0xFF => {
                self.halted = true;
                1
            }

            // NOP or unknown - treat as 1-cycle NOP
            _ => 1,
        }
    }

    /// Execute one instruction and perform device ticks for each consumed cycle.
    /// Returns the number of cycles consumed (0 if already halted).
    pub fn step_and_tick_instruction(&mut self) -> u64 {
        if self.halted {
            return 0;
        }
        let cycles = self.step_instruction();
        for _ in 0..cycles {
            self.cycles += 1;
            for dev in self.devices.iter_mut() {
                dev.tick(self.cycles);
            }
        }
        cycles
    }

    /// Execute up to `n` instructions (or stop sooner if halted).
    /// Returns (executed_instructions, total_cycles_consumed).
    pub fn step_n_instructions(&mut self, n: usize) -> (usize, u64) {
        let mut executed = 0usize;
        let mut cycles_consumed = 0u64;
        for _ in 0..n {
            if self.halted {
                break;
            }
            let c = self.step_and_tick_instruction();
            executed += 1;
            cycles_consumed += c;
        }
        (executed, cycles_consumed)
    }

    /// Run until HLT (or until halted). Devices are ticked once per cycle.
    pub fn run(&mut self) {
        while !self.halted {
            let cycles = self.step_instruction();
            for _ in 0..cycles {
                self.cycles += 1;
                for dev in self.devices.iter_mut() {
                    dev.tick(self.cycles);
                }
            }
        }
    }

    /// Run with a simple trace: prints an instruction summary before each instruction.
    pub fn run_with_trace(&mut self) {
        while !self.halted {
            let pc_before = self.pc;
            let opcode = self.mem.read(pc_before);
            // Print a short trace line
            println!(
                "[trace] PC={:02X} OPCODE={:02X} R=[{},{},{},{}] CYC={}",
                pc_before, opcode, self.regs[0], self.regs[1], self.regs[2], self.regs[3], self.cycles
            );
            let cycles = self.step_instruction();
            for _ in 0..cycles {
                self.cycles += 1;
                for dev in self.devices.iter_mut() {
                    dev.tick(self.cycles);
                }
            }
        }
    }

    pub fn dump_state(&self) {
        println!("--- CPU STATE ---");
        println!("PC: {:02X} Cycles: {}", self.pc, self.cycles);
        println!("Z: {}", self.z);
        for i in 0..self.regs.len() {
            println!("R{}: {:02X}", i, self.regs[i]);
        }
        println!("-----------------");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_cycles() {
        // LDI R0,5 (2)
        // LDI R1,10 (2)
        // ADD R0,R1 (3)
        // OUT R0 (4)
        // HLT (1)
        let program: &[u8] = &[
            0x10, 0x05, // LDI R0,5
            0x11, 0x0A, // LDI R1,10
            0x20, 0x01, // ADD R0,R1
            0x50,       // OUT R0
            0xFF,       // HLT
        ];

        let mut cpu = CPU::new();
        cpu.load(program, 0);
        cpu.run();
        assert_eq!(cpu.regs[0], 15);
        // expected cycles = 2 + 2 + 3 + 4 + 1 = 12
        assert_eq!(cpu.cycles, 12);
        assert!(cpu.halted);
    }
}
