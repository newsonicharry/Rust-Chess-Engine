use std::time::Instant;

#[derive(Copy, Clone)]
pub struct SearchLimits{
    timer: Instant,
    soft_stop: u32,
    hard_stop: u32,
}

impl SearchLimits {
    pub fn new(soft_stop: u32, hard_stop: u32) -> SearchLimits {
        SearchLimits{
            timer: Instant::now(),
            soft_stop,
            hard_stop,
        }
    }


    pub fn is_soft_stop(&self) -> bool{
        self.soft_stop <= self.timer.elapsed().as_millis() as u32
    }

    pub fn is_hard_stop(&self) -> bool{
        self.hard_stop <= self.timer.elapsed().as_millis() as u32
    }
    
    pub fn ms_elapsed(&self) -> u128{
        self.timer.elapsed().as_millis()
    }
}