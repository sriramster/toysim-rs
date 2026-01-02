use std::fmt::{Debug, Formatter};

/// Device trait for per-cycle devices. The CPU calls `tick(current_cycle)` once per cycle.
/// Devices are free to do work or produce side effects when tick() is called.
pub trait Device {
    fn tick(&mut self, current_cycle: u64);
}

impl Debug for dyn Device {
    fn fmt (&self, _: &mut Formatter::<'_>) -> Result<(), std::fmt::Error>{
        Ok(())
    }
}

/// A very small example Timer device that prints every `period` cycles.
pub struct TimerDevice {
    period: u64,
    next: u64,
}

impl TimerDevice {
    pub fn new(period: u64) -> Self {
        TimerDevice { period, next: period }
    }
}

impl Device for TimerDevice {
    fn tick(&mut self, current_cycle: u64) {
        if current_cycle >= self.next {
            println!("[device] Timer tick at cycle {}", current_cycle);
            self.next += self.period;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_runs() {
        let mut t = TimerDevice::new(2);
        // tick a few times; ensure no panic (behavior is printed)
        t.tick(1);
        t.tick(2);
        t.tick(3);
        t.tick(4);
    }
}
