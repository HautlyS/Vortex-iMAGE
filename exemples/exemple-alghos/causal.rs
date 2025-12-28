//! Causal delivery layer for P2P messaging
//! Ensures messages are delivered in causal order
use crate::clock::VectorClock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Message with causal metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalMessage<T> {
    pub clock: VectorClock,
    pub sender: String,
    pub payload: T,
}

impl<T> CausalMessage<T> {
    pub fn new(sender: &str, clock: VectorClock, payload: T) -> Self {
        Self {
            clock,
            sender: sender.to_string(),
            payload,
        }
    }
}

/// Buffer that reorders messages for causal delivery
#[derive(Debug)]
pub struct CausalBuffer<T> {
    pending: VecDeque<CausalMessage<T>>,
    delivered: VectorClock,
    local_peer: String,
}

impl<T: Clone> CausalBuffer<T> {
    pub fn new(local_peer: &str) -> Self {
        Self {
            pending: VecDeque::new(),
            delivered: VectorClock::new(),
            local_peer: local_peer.to_string(),
        }
    }

    /// Check if message can be delivered (all dependencies satisfied)
    fn can_deliver(&self, msg: &CausalMessage<T>) -> bool {
        // Message can be delivered if for all peers except sender:
        // delivered[peer] >= msg.clock[peer]
        // And for sender: delivered[sender] == msg.clock[sender] - 1
        for (peer, &val) in &msg.clock.0 {
            let delivered_val = self.delivered.get(peer);
            if peer == &msg.sender {
                // Sender's clock should be exactly one ahead
                if val != delivered_val + 1 {
                    return false;
                }
            } else {
                // Other peers should be at or behind
                if val > delivered_val {
                    return false;
                }
            }
        }
        true
    }

    /// Receive a message, returns messages ready for delivery
    pub fn receive(&mut self, msg: CausalMessage<T>) -> Vec<CausalMessage<T>> {
        self.pending.push_back(msg);
        self.try_deliver()
    }

    /// Try to deliver pending messages in causal order
    fn try_deliver(&mut self) -> Vec<CausalMessage<T>> {
        let mut ready = Vec::new();
        let mut progress = true;

        while progress {
            progress = false;
            let mut i = 0;
            while i < self.pending.len() {
                if self.can_deliver(&self.pending[i]) {
                    let msg = self.pending.remove(i).unwrap();
                    self.delivered.merge(&msg.clock);
                    ready.push(msg);
                    progress = true;
                } else {
                    i += 1;
                }
            }
        }
        ready
    }

    /// Create a new message from local peer
    pub fn send(&mut self, payload: T) -> CausalMessage<T> {
        self.delivered.tick(&self.local_peer);
        CausalMessage::new(&self.local_peer, self.delivered.clone(), payload)
    }

    /// Current delivered clock
    pub fn clock(&self) -> &VectorClock {
        &self.delivered
    }

    /// Number of pending messages
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_order() {
        let mut alice_buf = CausalBuffer::new("alice");
        let mut bob_buf = CausalBuffer::new("bob");

        // Alice sends m1
        let m1 = alice_buf.send("hello");
        
        // Bob receives m1
        let delivered = bob_buf.receive(m1.clone());
        assert_eq!(delivered.len(), 1);

        // Bob sends m2 (depends on m1)
        let m2 = bob_buf.send("world");

        // Alice receives m2 - should work since she sent m1
        let delivered = alice_buf.receive(m2);
        assert_eq!(delivered.len(), 1);
    }

    #[test]
    fn test_out_of_order() {
        let mut alice_buf = CausalBuffer::new("alice");
        let mut bob_buf = CausalBuffer::new("bob");

        let m1 = alice_buf.send("first");
        let m2 = alice_buf.send("second");

        // Bob receives m2 first (out of order)
        let delivered = bob_buf.receive(m2);
        assert_eq!(delivered.len(), 0); // Can't deliver yet

        // Bob receives m1
        let delivered = bob_buf.receive(m1);
        assert_eq!(delivered.len(), 2); // Both delivered in order
        assert_eq!(delivered[0].payload, "first");
        assert_eq!(delivered[1].payload, "second");
    }
}
