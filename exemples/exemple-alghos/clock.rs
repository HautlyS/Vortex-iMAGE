//! Vector Clock for causal ordering in distributed systems
//! Based on Lamport (1978) and Fidge/Mattern (1988)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector Clock - tracks causal dependencies across peers
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock(pub HashMap<String, u64>);

impl VectorClock {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Increment clock for local peer, returns new value
    pub fn tick(&mut self, peer: &str) -> u64 {
        let v = self.0.entry(peer.to_string()).or_insert(0);
        *v += 1;
        *v
    }

    /// Get clock value for a peer
    pub fn get(&self, peer: &str) -> u64 {
        self.0.get(peer).copied().unwrap_or(0)
    }

    /// Merge with another clock (take max of each component)
    pub fn merge(&mut self, other: &Self) {
        for (peer, &val) in &other.0 {
            let entry = self.0.entry(peer.clone()).or_insert(0);
            *entry = (*entry).max(val);
        }
    }

    /// Check if self happens-before other (self < other)
    pub fn happens_before(&self, other: &Self) -> bool {
        let mut dominated = false;
        for (peer, &val) in &self.0 {
            let other_val = other.get(peer);
            if val > other_val {
                return false; // self has higher value somewhere
            }
            if val < other_val {
                dominated = true;
            }
        }
        // Check if other has peers we don't have
        for (peer, &val) in &other.0 {
            if val > 0 && self.get(peer) == 0 {
                dominated = true;
            }
        }
        dominated
    }

    /// Check if two clocks are concurrent (neither happens-before the other)
    pub fn is_concurrent(&self, other: &Self) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && self != other
    }

    /// Create clock with single peer at value 1
    pub fn init(peer: &str) -> Self {
        let mut clock = Self::new();
        clock.tick(peer);
        clock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happens_before() {
        let a = VectorClock::init("alice");
        let mut b = a.clone();
        b.tick("bob");
        
        assert!(a.happens_before(&b));
        assert!(!b.happens_before(&a));
    }

    #[test]
    fn test_concurrent() {
        let a = VectorClock::init("alice");
        let b = VectorClock::init("bob");
        
        assert!(a.is_concurrent(&b));
    }

    #[test]
    fn test_merge() {
        let mut a = VectorClock::init("alice");
        a.tick("alice"); // alice: 2
        
        let mut b = VectorClock::init("bob");
        b.tick("bob"); // bob: 2
        
        a.merge(&b);
        assert_eq!(a.get("alice"), 2);
        assert_eq!(a.get("bob"), 2);
    }
}
