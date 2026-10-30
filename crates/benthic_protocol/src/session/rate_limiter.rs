use std::{sync::Arc, time::Duration};

use tokio::sync::Semaphore;

/// The rate limiter used to prevent overloading the network
pub struct RateLimiter {
    /// Semaphore for making async processes wait until they are no longer being rate limited
    sem: Arc<Semaphore>,
}

impl RateLimiter {
    /// create a new instance of the rate limiter.
    /// Defaults are set very low
    pub fn new(rate: u32, burst: usize) -> Self {
        let sem = Arc::new(Semaphore::new(0));
        let refill = sem.clone();
        let period = Duration::from_secs_f64(1.0 / rate as f64);

        tokio::spawn(async move {
            let mut tick = tokio::time::interval(period);
            loop {
                tick.tick().await;
                if refill.available_permits() < burst {
                    refill.add_permits(1);
                }
            }
        });

        Self { sem }
    }

    /// used to acquire the semaphore.
    /// this awaits until the ratelimiter is ready for a new request.
    pub async fn acquire(&self) {
        self.sem.acquire().await.unwrap().forget();
    }
}
