use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

pub struct TarpitState {
    blocked_ips: RwLock<HashMap<String, Instant>>,
}

const BLOCK_DURATION_SECS: u64 = 3600;

impl TarpitState {
    pub fn new() -> Self {
        Self { blocked_ips: RwLock::new(HashMap::new()) }
    }

    pub fn is_blocked(&self, ip: &str) -> bool {
        let map = self.blocked_ips.read().unwrap();
        match map.get(ip) {
            Some(t) => t.elapsed().as_secs() < BLOCK_DURATION_SECS,
            None => false,
        }
    }

    pub fn block(&self, ip: &str) {
        let mut map = self.blocked_ips.write().unwrap();
        map.retain(|_, t| t.elapsed().as_secs() < BLOCK_DURATION_SECS);
        map.insert(ip.to_string(), Instant::now());
    }
}