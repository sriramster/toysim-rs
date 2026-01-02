```markdown
# toy_cpu — educational cycle-accurate toy CPU in Rust

This is a small educational CPU emulator that is cycle-accurate at the instruction level:
each instruction returns a cycle cost and the CPU ticks attached devices once per cycle.

Features
- 8-bit registers and memory (4 general registers, 256 bytes memory).
- Instruction-level cycle accounting.
- Device trait and per-cycle device tick calls (example: Timer).
- Simple instruction set (LDI, ADD, SUB, LOAD, STORE, JMP, JZ, OUT, HLT).
- CLI with `--trace` to print instruction traces.
- Unit tests and an example program.

Run
- Build and run:
  - `cargo run --release`
  - `cargo run -- --trace` (prints trace)
- Test:
  - `cargo test`

Design notes
- CPU.step_instruction() executes one instruction and returns the cycles taken.
- CPU.run() applies those cycles and calls devices' tick() once per cycle to model timed devices.
- Extend by: more instructions, memory-mapped IO, assembler/disassembler, micro-cycle modeling.

License
- Public domain / CC0 (use as you like).
```