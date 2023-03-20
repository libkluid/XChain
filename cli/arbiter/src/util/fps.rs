use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct Fps {
    quota: usize,
    queue: VecDeque<Duration>,
    instant: std::time::Instant,
}

impl Fps {
    pub fn new(quota: usize) -> Self {
        let instant = Instant::now();
        let queue = VecDeque::with_capacity(quota);
        
        Self {
            quota,
            queue,
            instant,
        }
    }

    pub fn tick(&mut self) -> f64 {
        let elapsed = self.instant.elapsed();
        self.queue.push_back(elapsed);

        let length = self.queue.len();

        let front = if length < self.quota {
            *self.queue.front().unwrap()
        } else {
            self.queue.pop_front().unwrap()
        };

        let average = (elapsed - front).as_micros() as f64 / length as f64;
        let fps = 1_000_000_f64 / average;
        fps
    }
}
