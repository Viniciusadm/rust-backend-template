use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::api::state::AppState;
use crate::errors::AppError;

const PRUNE_THRESHOLD: usize = 10_000;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    hits: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            hits: Arc::default(),
        }
    }

    pub fn disabled() -> Self {
        Self::new(0, Duration::ZERO)
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        if self.max_requests == 0 {
            return true;
        }
        let now = Instant::now();
        let mut hits = self
            .hits
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if hits.len() > PRUNE_THRESHOLD {
            hits.retain(|_, (started_at, _)| now.duration_since(*started_at) < self.window);
        }
        let entry = hits.entry(ip).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        entry.1 += 1;
        entry.1 <= self.max_requests
    }
}

pub async fn limit_requests(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip());
    if let Some(ip) = client_ip
        && !state.rate_limiter.allow(ip)
    {
        return AppError::TooManyRequests("too many requests, try again later".to_owned())
            .into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_limit_and_isolates_clients() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        let first: IpAddr = "10.0.0.1".parse().unwrap();
        let second: IpAddr = "10.0.0.2".parse().unwrap();

        assert!(limiter.allow(first));
        assert!(limiter.allow(first));
        assert!(!limiter.allow(first));
        assert!(limiter.allow(second));
    }

    #[test]
    fn disabled_limiter_allows_everything() {
        let limiter = RateLimiter::disabled();
        let ip: IpAddr = "10.0.0.1".parse().unwrap();

        assert!((0..100).all(|_| limiter.allow(ip)));
    }
}
