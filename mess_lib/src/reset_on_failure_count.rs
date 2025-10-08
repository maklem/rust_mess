
pub struct ResettingCounter<F: FnMut() -> ()> {
    reset_function: F,
    count: u32,
    max_failures: u32
}

impl<F: FnMut() -> ()> ResettingCounter<F> {
    pub fn new(callback: F, max_failures: u32) -> Self {
        ResettingCounter {
            reset_function: callback,
            count: 0,
            max_failures: max_failures,
        }
    }

    pub fn increment_failure(&mut self) {
        self.count += 1;
        if self.count >= self.max_failures {
            (self.reset_function)();
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
}