//! Coalesce rapid DOM diff/index requests within a 16 ms window (Phase 7.1).

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const COALESCE_MS: u64 = 16;

struct Pending {
    at: Instant,
    value: serde_json::Value,
}

static PENDING: Mutex<Option<HashMap<u64, Pending>>> = Mutex::new(None);

fn key_for(url: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}

pub async fn coalesce_dom_index<F, Fut>(url: &str, fresh: F) -> serde_json::Value
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = serde_json::Value>,
{
    let now = Instant::now();
    let key = key_for(url);
    {
        let mut guard = PENDING.lock().unwrap();
        if guard.is_none() {
            *guard = Some(HashMap::new());
        }
        let map = guard.as_mut().unwrap();
        map.retain(|_, p| now.duration_since(p.at) <= Duration::from_millis(COALESCE_MS * 4));
        if let Some(pending) = map.get(&key) {
            if now.duration_since(pending.at) <= Duration::from_millis(COALESCE_MS) {
                return pending.value.clone();
            }
        }
    }
    let value = fresh().await;
    let mut guard = PENDING.lock().unwrap();
    if let Some(map) = guard.as_mut() {
        map.insert(key, Pending { at: now, value: value.clone() });
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn coalesces_within_window() {
        let v1 = coalesce_dom_index("http://localhost/a", || async {
            serde_json::json!({"n": 1})
        })
        .await;
        let v2 = coalesce_dom_index("http://localhost/a", || async {
            serde_json::json!({"n": 2})
        })
        .await;
        assert_eq!(v1, v2);
    }
}
