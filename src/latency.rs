use std::time::Instant;

pub struct Latency {
    recent: Instant,
    context: String,
}

impl Latency {
    pub fn new(context: &str) -> Latency {
        Latency {
            recent: Instant::now(),
            context: context.to_string(),
        }
    }

    pub fn tick(&mut self, description: &str) {
        let milliseconds = self.recent.elapsed().as_millis();
        self.recent = Instant::now();
        println!("⌚ {}: [{}ms] {}", self.context, milliseconds, description);
    }
}