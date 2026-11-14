use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
struct Entry<T> {
    token: String,
    source: String,
    expires: Instant,
    value: T,
}
/// Bounded, expiring provider values. A token selects held data; it never authorizes a write.
pub(crate) struct ProviderArtifacts<T> {
    epoch: String,
    next: u64,
    capacity: usize,
    lifetime: Duration,
    entries: VecDeque<Entry<T>>,
}
impl<T: Clone> ProviderArtifacts<T> {
    pub(crate) fn new(capacity: usize, lifetime: Duration) -> Self {
        assert!(capacity > 0);
        static EPOCH: AtomicU64 = AtomicU64::new(1);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self {
            epoch: format!(
                "{:x}-{:x}-{:x}",
                std::process::id(),
                time,
                EPOCH.fetch_add(1, Ordering::Relaxed)
            ),
            next: 0,
            capacity,
            lifetime,
            entries: VecDeque::new(),
        }
    }
    pub(crate) fn insert(&mut self, now: Instant, source: String, value: T) -> String {
        self.entries.retain(|entry| entry.expires > now);
        while self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.next += 1;
        let token = format!("{}:{:x}", self.epoch, self.next);
        self.entries.push_back(Entry {
            token: token.clone(),
            source,
            expires: now + self.lifetime,
            value,
        });
        token
    }
    pub(crate) fn get(&mut self, now: Instant, token: &str, source: &str) -> Option<T> {
        self.entries.retain(|entry| entry.expires > now);
        let index = self.entries.iter().position(|entry| entry.token == token)?;
        if self.entries[index].source != source {
            self.entries.remove(index);
            return None;
        }
        Some(self.entries[index].value.clone())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_values_are_bounded_expiring_and_source_bound() {
        let now = Instant::now();
        let mut values = ProviderArtifacts::new(2, Duration::from_secs(10));
        let first = values.insert(now, "source".into(), "first");
        let second = values.insert(now, "source".into(), "second");
        let third = values.insert(now, "source".into(), "third");
        assert!(values.get(now, &first, "source").is_none());
        assert_eq!(values.get(now, &second, "source"), Some("second"));
        assert!(values.get(now, &third, "changed").is_none());
        assert!(
            values
                .get(now + Duration::from_secs(10), &second, "source")
                .is_none()
        );
    }
    #[test]
    fn tokens_cannot_alias_another_store_and_opaque_data_is_held_verbatim() {
        let now = Instant::now();
        let mut first = ProviderArtifacts::new(2, Duration::from_secs(10));
        let mut second = ProviderArtifacts::new(2, Duration::from_secs(10));
        let original = serde_json::json!({"data":{"uri":"file:///provider-owned","nested":[1,2]},"title":"Fix"});
        let token = first.insert(now, "source".into(), original.clone());
        let other = second.insert(now, "source".into(), original.clone());
        assert_ne!(token, other);
        assert!(second.get(now, &token, "source").is_none());
        let mut returned = first.get(now, &token, "source").unwrap();
        returned["data"] = serde_json::Value::Null;
        assert_eq!(first.get(now, &token, "source"), Some(original));
    }
}
