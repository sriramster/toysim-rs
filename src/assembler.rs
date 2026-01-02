use std::collections::HashMap;

/// Assemble the toy ISA source into bytes.
/// - Two-pass assembler: first collects labels (and handles `ORG` directive), then encodes.
/// - Supports comments starting with ';' or '#' and blank lines.
/// - Registers: R0..R3
/// - Numeric formats: decimal (e.g. 42) or hex (0x2A).
pub fn assemble(src: &str) -> Result<Vec<u8>, String> {
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut lines: Vec<String> = Vec::new();
    let mut pc: usize = 0;

    // Normalize lines and collect for second pass
    for (lineno, raw) in src.lines().enumerate() {
        let mut line = raw.trim().to_string();
        // Remove comments
        if let Some(idx) = line.find(';') {
            line.truncate(idx);
            line = line.trim().to_string();
        }
        if let Some(idx) = line.find('#') {
            line.truncate(idx);
            line = line.trim().to_string();
        }
        if line.is_empty() {
            continue;
        }
        // Label
        if line.ends_with(':') {
            let label = line[..line.len()-1].trim().to_string();
            if label.is_empty() {
                return Err(format!("Empty label at line {}", lineno+1));
            }
            if labels.contains_key(&label) {
                return Err(format!("Duplicate label '{}' at line {}", label, lineno+1));
            }
            labels.insert(label, pc);
            continue;
        }

        // Directive ORG
        let up = line.to_uppercase();
        if up.starts_with("ORG ") || up == "ORG" {
            // parse operand after ORG
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(format!("ORG without address at line {}", lineno+1));
            }
            let addr = parse_number(parts[1]).map_err(|e| format!("line {}: {}", lineno+1, e))?;
            pc = addr as usize;
            continue;
        }

        // Otherwise it's an instruction line; store it and increase pc according to size
        let (mnemonic, _) = split_mnemonic_operands(&line);
        let instr_size = instruction_size(&mnemonic)
            .ok_or_else(|| format!("Unknown mnemonic '{}' at line {}", mnemonic, lineno+1))?;
        lines.push(line);
        pc = pc.wrapping_add(instr_size);
    }

    // Second pass: encode
    let mut out: Vec<u8> = Vec::new();
    let mut cur_pc = 0usize;
    for (lineno, line) in lines.into_iter().enumerate() {
        let (mnemonic, operands) = split_mnemonic_operands(&line);
        let mnemonic_upper = mnemonic.to_uppercase();
        match mnemonic_upper.as_str() {
            "LDI" => {
                let (r, imm) = parse_two_operands_reg_imm(&operands, lineno+1)?;
                if r > 3 { return Err(format!("Invalid register R{} at line {}", r, lineno+1)); }
                out.push(0x10 | (r as u8));
                out.push(imm);
                cur_pc += 2;
            }
            "ADD" => {
                let (d, s) = parse_two_operands_reg_reg(&operands, lineno+1)?;
                if d > 3 || s > 3 { return Err(format!("Invalid register at line {}", lineno+1)); }
                out.push(0x20 | (d as u8));
                out.push(s as u8);
                cur_pc += 2;
            }
            "SUB" => {
                let (d, s) = parse_two_operands_reg_reg(&operands, lineno+1)?;
                if d > 3 || s > 3 { return Err(format!("Invalid register at line {}", lineno+1)); }
                out.push(0x21 | (d as u8));
                out.push(s as u8);
                cur_pc += 2;
            }
            "LOAD" => {
                let (d, addr) = parse_two_operands_reg_addr(&operands, lineno+1, &labels)?;
                if d > 3 { return Err(format!("Invalid register R{} at line {}", d, lineno+1)); }
                out.push(0x30 | (d as u8));
                out.push(addr);
                cur_pc += 2;
            }
            "STORE" => {
                let (s, addr) = parse_two_operands_reg_addr(&operands, lineno+1, &labels)?;
                if s > 3 { return Err(format!("Invalid register R{} at line {}", s, lineno+1)); }
                out.push(0x31 | (s as u8));
                out.push(addr);
                cur_pc += 2;
            }
            "JMP" => {
                let addr = parse_addr_operand(&operands.trim(), lineno+1, &labels)?;
                out.push(0x40);
                out.push(addr);
                cur_pc += 2;
            }
            "JZ" => {
                // form: JZ Rn, addr
                let (r, addr) = parse_two_operands_reg_addr(&operands, lineno+1, &labels)?;
                if r > 3 { return Err(format!("Invalid register R{} at line {}", r, lineno+1)); }
                out.push(0x41 | (r as u8));
                out.push(addr);
                cur_pc += 2;
            }
            "OUT" => {
                let r = parse_reg_operand(&operands.trim(), lineno+1)?;
                if r > 3 { return Err(format!("Invalid register R{} at line {}", r, lineno+1)); }
                out.push(0x50 | (r as u8));
                cur_pc += 1;
            }
            "HLT" => {
                out.push(0xFF);
                cur_pc += 1;
            }
            "NOP" => {
                out.push(0x00);
                cur_pc += 1;
            }
            _ => {
                return Err(format!("Unknown mnemonic '{}' at assembly pass line {}", mnemonic, lineno+1));
            }
        }
    }

    Ok(out)
}

fn split_mnemonic_operands(line: &str) -> (String, String) {
    // Split at first whitespace
    if let Some(idx) = line.find(char::is_whitespace) {
        let (m, rest) = line.split_at(idx);
        (m.trim().to_string(), rest.trim().trim_start_matches(',').to_string())
    } else {
        (line.trim().to_string(), String::new())
    }
}

fn instruction_size(mnemonic: &str) -> Option<usize> {
    match mnemonic.to_uppercase().as_str() {
        "LDI" => Some(2),
        "ADD" => Some(2),
        "SUB" => Some(2),
        "LOAD" => Some(2),
        "STORE" => Some(2),
        "JMP" => Some(2),
        "JZ" => Some(2),
        "OUT" => Some(1),
        "HLT" => Some(1),
        "NOP" => Some(1),
        _ => None,
    }
}

fn parse_reg(s: &str) -> Result<u8, String> {
    let s = s.trim();
    if s.len() >= 2 && (s.as_bytes()[0] == b'R' || s.as_bytes()[0] == b'r') {
        let idx_str = &s[1..];
        idx_str.parse::<u8>().map_err(|_| format!("Invalid register '{}'", s))
    } else {
        Err(format!("Invalid register '{}'", s))
    }
}

fn parse_reg_operand(op: &str, lineno: usize) -> Result<u8, String> {
    parse_reg(op).map_err(|e| format!("line {}: {}", lineno, e))
}

fn parse_two_operands_reg_imm(ops: &str, lineno: usize) -> Result<(u8, u8), String> {
    let parts: Vec<&str> = ops.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.len() != 2 {
        return Err(format!("line {}: expected two operands", lineno));
    }
    let r = parse_reg(parts[0]).map_err(|e| format!("line {}: {}", lineno, e))?;
    let imm = parse_number(parts[1]).map_err(|e| format!("line {}: {}", lineno, e))?;
    Ok((r, imm))
}

fn parse_two_operands_reg_reg(ops: &str, lineno: usize) -> Result<(u8, u8), String> {
    let parts: Vec<&str> = ops.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.len() != 2 {
        return Err(format!("line {}: expected two operands", lineno));
    }
    let a = parse_reg(parts[0]).map_err(|e| format!("line {}: {}", lineno, e))?;
    let b = parse_reg(parts[1]).map_err(|e| format!("line {}: {}", lineno, e))?;
    Ok((a, b))
}

fn parse_two_operands_reg_addr(ops: &str, lineno: usize, labels: &HashMap<String, usize>) -> Result<(u8, u8), String> {
    let parts: Vec<&str> = ops.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.len() != 2 {
        return Err(format!("line {}: expected two operands", lineno));
    }
    let r = parse_reg(parts[0]).map_err(|e| format!("line {}: {}", lineno, e))?;
    let addr = parse_addr_operand(parts[1], lineno, labels)?;
    Ok((r, addr))
}

fn parse_addr_operand(op: &str, lineno: usize, labels: &HashMap<String, usize>) -> Result<u8, String> {
    let s = op.trim();
    if s.is_empty() {
        return Err(format!("line {}: expected address or label", lineno));
    }
    // label?
    if let Some(&addr) = labels.get(s) {
        return Ok(addr as u8);
    }
    // otherwise number
    parse_number(s).map_err(|e| format!("line {}: {}", lineno, e))
}

fn parse_number(s: &str) -> Result<u8, String> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        u8::from_str_radix(&s[2..], 16).map_err(|_| format!("Invalid hex number '{}'", s))
    } else {
        s.parse::<u16>()
            .map_err(|_| format!("Invalid number '{}'", s))
            .and_then(|v| {
                if v > 0xFF {
                    Err(format!("Number out of range (0..255): {}", v))
                } else {
                    Ok(v as u8)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_basic_program() {
        let src = r#"
            start:
            LDI R0, 5
            LDI R1, 10
            ADD R0, R1
            OUT R0
            HLT
        "#;
        let bytes = assemble(src).expect("assemble failed");
        let expected: Vec<u8> = vec![
            0x10, 0x05, // LDI R0,5
            0x11, 0x0A, // LDI R1,10
            0x20, 0x01, // ADD R0,R1
            0x50,       // OUT R0
            0xFF,       // HLT
        ];
        assert_eq!(bytes, expected);
    }

    #[test]
    fn assemble_labels_and_jmp() {
        let src = r#"
            ORG 0x10
            loop:
            LDI R0, 1
            JMP loop
        "#;
        let bytes = assemble(src).expect("assemble failed");
        // ORG 0x10 sets origin to 0x10 -> bytes vector will be exactly the encoded instructions (no zero padding)
        // LDI -> 2 bytes; JMP ->2 bytes
        assert_eq!(bytes.len(), 4);
    }
}
