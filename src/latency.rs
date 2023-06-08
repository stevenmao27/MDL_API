use std::time::Instant;

struct Latency {
    now: Instant,
    recent: Instant,
}

impl Latency {
    pub fn new() -> Latency {
        Latency {
            now: Instant::now(),
            recent: Instant::now(),
        }
    }

    pub fn get(&mut self) {
        let milliseconds = self.recent.elapsed().as_millis();
        self.recent = Instant::now();
        println!("{}ms", milliseconds);
    }
}