use std::time::{Duration, Instant};

/// Lets an event through at most once per interval, so a fast producer (a download reporting every
/// chunk) does not flood a slow consumer (a terminal, or UI events).
#[derive(Debug)]
pub struct Throttle {
    every: Duration,
    last: Option<Instant>,
}

impl Throttle {
    pub fn new(every: Duration) -> Throttle {
        Throttle { every, last: None }
    }

    /// True when enough time has passed since the last time this returned true. The first call
    /// always returns true.
    pub fn ready(&mut self) -> bool {
        let now = Instant::now();
        if self
            .last
            .is_some_and(|last| now.duration_since(last) < self.every)
        {
            return false;
        }
        self.last = Some(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_call_passes_then_calls_inside_the_interval_are_dropped() {
        let mut throttle = Throttle::new(Duration::from_secs(60));
        assert!(throttle.ready());
        assert!(!throttle.ready());
        assert!(!throttle.ready());
    }

    #[test]
    fn passes_again_once_the_interval_has_elapsed() {
        let mut throttle = Throttle::new(Duration::ZERO);
        assert!(throttle.ready());
        assert!(throttle.ready());
    }
}
