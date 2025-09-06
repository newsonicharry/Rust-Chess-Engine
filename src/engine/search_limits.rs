use std::time::Instant;

pub struct SearchLimits{
    timer: Instant,
    soft_stop: u128,
    hard_stop: u128,
}

impl SearchLimits {
    pub fn new(soft_stop: u128, hard_stop: u128) -> SearchLimits {
        SearchLimits{
            timer: Instant::now(),
            soft_stop,
            hard_stop,
        }
    }


    pub fn is_soft_stop(&self) -> bool{
        self.soft_stop <= self.timer.elapsed().as_millis()
    }

    pub fn hard_stop(&self) -> bool{
        self.hard_stop <= self.timer.elapsed().as_millis()
    }
    
    pub fn ms_elapsed(&self) -> u128{
        self.timer.elapsed().as_millis()
    }
}