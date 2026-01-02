mod cpu;
mod device;
mod memory;
mod assembler;
mod repl;

use cpu::CPU;
use device::TimerDevice;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let trace = args.iter().any(|a| a == "--trace" || a == "-t");
    let repl_mode = args.iter().any(|a| a == "--repl" || a == "-r");

    if repl_mode {
        // Start REPL (it creates its own CPU)
        repl::run_repl();
        return;
    }

    // Example program:
    let program: &[u8] = &[
        0x10, 0x05, // LDI R0,5
        0x11, 0x0A, // LDI R1,10
        0x20, 0x01, // ADD R0, R1
        0x50,       // OUT R0
        0xFF,       // HLT
    ];

    let mut cpu = CPU::new();
    // attach an example timer device (prints every 5 cycles)
    cpu.attach_device(Box::new(TimerDevice::new(5)));
    cpu.load(program, 0);

    if trace {
        cpu.run_with_trace();
    } else {
        cpu.run();
    }

    cpu.dump_state();
}
