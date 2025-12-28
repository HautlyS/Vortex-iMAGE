//! LRU Cache with TTL for Vortex
//! Simple, efficient, async-safe
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Cache entry with timestamp
struct Entry<V> {
    value: V,
    created: Instant,
}

/// LRU Cache with TTL eviction
pub struct VortexCache<K, V> {
    map: HashMap<K, Entry<V>>,
    order: VecDeque<K>,
    max_size: usize,
    ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> VortexCache<K, V> {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            map: HashMap::with_capacity(max_size),
            order: VecDeque::with_capacity(max_size),
            max_size,
            ttl,
        }
    }

    /// Get value if exists and not expired
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.map.get(key) {
            if entry.created.elapsed() < self.ttl {
                // Move to front (most recently used)
                self.order.retain(|k| k != key);
                self.order.push_front(key.clone());
                return Some(entry.value.clone());
            } else {
                // Expired, remove
                self.map.remove(key);
                self.order.retain(|k| k != key);
            }
        }
        None
    }

    /// Insert or update value
    pub fn put(&mut self, key: K, value: V) {
        // Remove if exists
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
        }

        // Evict if full
        while self.map.len() >= self.max_size {
            if let Some(old_key) = self.order.pop_back() {
                self.map.remove(&old_key);
            }
        }

        self.map.insert(key.clone(), Entry {
            value,
            created: Instant::now(),
        });
        self.order.push_front(key);
    }

    /// Remove entry
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.order.retain(|k| k != key);
        self.map.remove(key).map(|e| e.value)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Evict expired entries
    pub fn evict_expired(&mut self) {
        let expired: Vec<K> = self.map
            .iter()
            .filter(|(_, e)| e.created.elapsed() >= self.ttl)
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in expired {
            self.map.remove(&key);
            self.order.retain(|k| k != &key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_eviction() {
        let mut cache = VortexCache::new(2, Duration::from_secs(60));
        
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3); // Should evict "a"
        
        assert!(cache.get(&"a").is_none());
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_ttl_expiration() {
        // Use a longer TTL to avoid timing issues under load
        let mut cache = VortexCache::new(10, Duration::from_secs(1));
        
        cache.put("key", "value");
        // Immediate get should work
        assert_eq!(cache.get(&"key"), Some("value"));
        
        // Sleep longer than TTL
        std::thread::sleep(Duration::from_millis(1500));
        // After TTL, should be expired
        assert!(cache.get(&"key").is_none());
    }
}
