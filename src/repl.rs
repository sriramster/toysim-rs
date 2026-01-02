// src/repl.rs
use crate::assembler;
use crate::cpu::CPU;
use std::io::{self, Write};

/// Run a small interactive REPL for assembling and running code.
/// Commands:
///  - asm        : enter assembler mode (multiline), finish with a single '.' on a line to assemble & load at addr 0
///  - run        : run until HLT
///  - trace      : run with trace
///  - step [N]   : execute N instructions (default 1)
///  - dump       : print CPU state
///  - regs       : print registers
///  - mem <addr> <len> : dump memory bytes
///  - exit|quit  : exit REPL
///  - help       : show help
pub fn run_repl() {
    let mut cpu = CPU::new();
    println!("toy_cpu REPL. Type 'help' for commands. Enter 'asm' to write assembler lines (end with a single '.' line).");

    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, exiting.");
            break;
        }
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let cmd = parts.next().unwrap().to_lowercase();

        match cmd.as_str() {
            "help" => print_help(),
            "asm" => {
                println!("Entering assembler mode. End input with a single '.' on a line.");
                let mut src = String::new();
                loop {
                    let mut a = String::new();
                    let _ = io::stdout().flush();
                    if io::stdin().read_line(&mut a).is_err() {
                        println!("read error; aborting asm mode");
                        break;
                    }
                    let t = a.trim_end().to_string();
                    if t == "." {
                        break;
                    }
                    src.push_str(&t);
                    src.push('\n');
                }
                match assembler::assemble(&src) {
                    Ok(bytes) => {
                        println!("Assembled {} bytes:", bytes.len());
                        for (i, b) in bytes.iter().enumerate() {
                            if i % 16 == 0 {
                                print!("\n{:04X}: ", i);
                            }
                            print!("{:02X} ", b);
                        }
                        println!();
                        // load at 0
                        cpu.load(&bytes, 0);
                        println!("Loaded at address 0.");
                    }
                    Err(e) => {
                        println!("Assemble error: {}", e);
                    }
                }
            }
            "run" => {
                cpu.run();
                println!("Program finished. cycles={}", cpu.cycles);
            }
            "trace" => {
                cpu.run_with_trace();
                println!("Program finished. cycles={}", cpu.cycles);
            }
            "step" => {
                let n: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                let (executed, cycles) = cpu.step_n_instructions(n);
                println!("Stepped {} instruction(s) consuming {} cycles. PC={:02X} cycles={}", executed, cycles, cpu.pc, cpu.cycles);
            }
            "dump" => {
                cpu.dump_state();
            }
            "regs" => {
                println!("R: {:?}", cpu.regs);
            }
            "mem" => {
                let a = parts.next().and_then(|s| parse_num(s));
                let l = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);
                if let Some(addr) = a {
                    for i in 0..l {
                        if i % 16 == 0 {
                            print!("\n{:04X}: ", addr + i);
                        }
                        print!("{:02X} ", cpu.mem.read(addr + i));
                    }
                    println!();
                } else {
                    println!("mem requires address. Usage: mem <addr> <len>");
                }
            }
            "exit" | "quit" => {
                println!("Bye.");
                break;
            }
            _ => {
                println!("Unknown command '{}'. Type 'help' for commands.", cmd);
            }
        }
    }
}

fn print_help() {
    println!(
        r#"Commands:
  asm                Enter assembler mode (end with a single '.' line). Assembles and loads at address 0.
  run                Run until HLT.
  trace              Run with trace output.
  step [N]           Execute N instructions (default 1).
  dump               Dump CPU state.
  regs               Print registers.
  mem <addr> <len>   Dump memory starting at <addr> for <len> bytes (len defaults to 16).
  exit, quit         Exit the REPL.
  help               Show this help.
"#
    );
}

fn parse_num(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse::<usize>().ok()
    }
}
