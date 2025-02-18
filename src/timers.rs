pub struct Timers {
    timers: [Timer; 3],
}

struct Timer {
    pub sync_enabled: bool,
    pub counter: u16,
    target: u16,
    use_system_clock: bool,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            timers: [Timer::new(), Timer::new(), Timer::new()],
        }
    }

    pub fn step(&mut self, cycles: u32) {
        for timer in &mut self.timers {
            timer.step(cycles);
        }
    }

    pub fn read(&self, address: u32) -> u32 {
        panic!("TODO")
    }

    pub fn write(&mut self, address: u32, value: u32) {
        let timer_index = address >> 4;

        let timer = &mut self.timers[timer_index as usize];

        match address % 4 {
            0 => timer.counter = value as u16,
            8 => timer.target = value as u16,
            _ => panic!("Failed to write to timer {}", timer_index),
        }
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            sync_enabled: false,
            counter: 0,
            target: 0,
            use_system_clock: true,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        if self.use_system_clock {
            // self.counter += cycles as u16;
        }
    }
}
