use std::fmt::{Debug, Formatter};

pub struct Memory {
    mem: [u8; 256],
}

impl Memory {
    pub fn new() -> Self {
        Memory { mem: [0; 256] }
    }

    pub fn size(&self) -> usize {
        self.mem.len()
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.mem[addr % self.size()]
    }

    pub fn write(&mut self, addr: usize, val: u8) {
        let a = addr % self.size();
        self.mem[a] = val;
    }

    pub fn write_bytes(&mut self, addr: usize, bytes: &[u8]) {
        let mut a = addr % self.size();
        for b in bytes {
            self.mem[a] = *b;
            a = (a + 1) % self.size();
        }
    }
}

impl Debug for Memory {
    fn fmt (&self, _: &mut Formatter::<'_>) -> Result<(), std::fmt::Error>{
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_read_write() {
        let mut m = Memory::new();
        m.write(0x10, 0xAA);
        assert_eq!(m.read(0x10), 0xAA);
    }
}
