//! Offline message queue for delivery when peers reconnect
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::chat::{Message, MessageId};
use crate::net::PeerId;

/// Queued message for offline delivery
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub message: Message,
    pub recipient_id: PeerId,
    pub queued_at: u64,
    pub attempts: u32,
    pub last_attempt: Option<u64>,
}

impl QueuedMessage {
    pub fn new(message: Message, recipient_id: PeerId) -> Self {
        Self {
            message,
            recipient_id,
            queued_at: now(),
            attempts: 0,
            last_attempt: None,
        }
    }

    pub fn record_attempt(&mut self) {
        self.attempts += 1;
        self.last_attempt = Some(now());
    }
}

/// Message queue for offline delivery
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessageQueue {
    /// Messages queued per recipient
    queued: HashMap<String, Vec<QueuedMessage>>,
    /// Maximum retry attempts
    max_attempts: u32,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queued: HashMap::new(),
            max_attempts: 5,
        }
    }

    pub fn with_max_attempts(max_attempts: u32) -> Self {
        Self {
            queued: HashMap::new(),
            max_attempts,
        }
    }

    /// Queue a message for offline delivery
    pub fn enqueue(&mut self, message: Message, recipient_id: PeerId) {
        let queued = QueuedMessage::new(message, recipient_id.clone());
        self.queued
            .entry(recipient_id.0)
            .or_default()
            .push(queued);
    }

    /// Get all messages queued for a recipient
    pub fn get_for_recipient(&self, recipient_id: &PeerId) -> Vec<&QueuedMessage> {
        self.queued
            .get(&recipient_id.0)
            .map(|msgs| msgs.iter().collect())
            .unwrap_or_default()
    }

    /// Get mutable messages for a recipient
    pub fn get_for_recipient_mut(&mut self, recipient_id: &PeerId) -> Vec<&mut QueuedMessage> {
        self.queued
            .get_mut(&recipient_id.0)
            .map(|msgs| msgs.iter_mut().collect())
            .unwrap_or_default()
    }

    /// Remove a delivered message from the queue
    pub fn remove(&mut self, recipient_id: &PeerId, message_id: &MessageId) -> Option<QueuedMessage> {
        if let Some(msgs) = self.queued.get_mut(&recipient_id.0) {
            if let Some(idx) = msgs.iter().position(|m| m.message.id == *message_id) {
                return Some(msgs.remove(idx));
            }
        }
        None
    }

    /// Remove all messages for a recipient (after successful delivery)
    pub fn clear_for_recipient(&mut self, recipient_id: &PeerId) -> Vec<QueuedMessage> {
        self.queued.remove(&recipient_id.0).unwrap_or_default()
    }

    /// Get total queued message count
    pub fn total_count(&self) -> usize {
        self.queued.values().map(|v| v.len()).sum()
    }

    /// Get count for a specific recipient
    pub fn count_for_recipient(&self, recipient_id: &PeerId) -> usize {
        self.queued.get(&recipient_id.0).map(|v| v.len()).unwrap_or(0)
    }

    /// Check if queue has messages for recipient
    pub fn has_messages_for(&self, recipient_id: &PeerId) -> bool {
        self.count_for_recipient(recipient_id) > 0
    }

    /// Remove messages that exceeded max attempts
    pub fn prune_failed(&mut self) -> Vec<QueuedMessage> {
        let mut failed = Vec::new();
        for msgs in self.queued.values_mut() {
            let (keep, remove): (Vec<_>, Vec<_>) = msgs
                .drain(..)
                .partition(|m| m.attempts < self.max_attempts);
            failed.extend(remove);
            *msgs = keep;
        }
        // Remove empty entries
        self.queued.retain(|_, v| !v.is_empty());
        failed
    }

    /// Serialize queue to bytes for persistence
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize queue from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

/// History reconciliation for merging local and remote message histories
#[derive(Clone, Debug, Default)]
pub struct HistoryReconciler {
    /// Vector clock for ordering
    vector_clock: HashMap<String, u64>,
}

impl HistoryReconciler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update vector clock for a peer
    pub fn update_clock(&mut self, peer_id: &str, timestamp: u64) {
        let entry = self.vector_clock.entry(peer_id.to_string()).or_insert(0);
        *entry = (*entry).max(timestamp);
    }

    /// Get current clock value for a peer
    pub fn get_clock(&self, peer_id: &str) -> u64 {
        self.vector_clock.get(peer_id).copied().unwrap_or(0)
    }

    /// Merge two message histories, removing duplicates
    pub fn merge_histories(&self, local: Vec<Message>, remote: Vec<Message>) -> Vec<Message> {
        let mut merged: HashMap<MessageId, Message> = HashMap::new();
        
        // Add local messages
        for msg in local {
            merged.insert(msg.id.clone(), msg);
        }
        
        // Add remote messages (won't overwrite existing)
        for msg in remote {
            merged.entry(msg.id.clone()).or_insert(msg);
        }
        
        // Sort by timestamp
        let mut result: Vec<_> = merged.into_values().collect();
        result.sort_by_key(|m| m.timestamp);
        result
    }

    /// Get messages newer than a given timestamp
    pub fn messages_since<'a>(&self, messages: &'a [Message], since: u64) -> Vec<&'a Message> {
        messages.iter().filter(|m| m.timestamp > since).collect()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::MessageContent;
    use proptest::prelude::*;

    fn mock_message(id: &str, room_id: &str, timestamp: u64) -> Message {
        Message {
            id: id.into(),
            room_id: room_id.into(),
            sender_id: "sender".into(),
            sender_name: "Sender".into(),
            content: MessageContent::Text("Test".into()),
            timestamp,
            signature: Vec::new(),
            encrypted: false,
            expires_at: None,
        }
    }

    #[test]
    fn test_queue_enqueue_and_retrieve() {
        let mut queue = MessageQueue::new();
        let msg = mock_message("msg1", "room1", 1000);
        let recipient = PeerId("peer1".into());
        
        queue.enqueue(msg.clone(), recipient.clone());
        
        assert_eq!(queue.count_for_recipient(&recipient), 1);
        assert!(queue.has_messages_for(&recipient));
        
        let queued = queue.get_for_recipient(&recipient);
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].message.id, "msg1");
    }

    #[test]
    fn test_queue_remove() {
        let mut queue = MessageQueue::new();
        let msg = mock_message("msg1", "room1", 1000);
        let recipient = PeerId("peer1".into());
        
        queue.enqueue(msg, recipient.clone());
        
        let removed = queue.remove(&recipient, &"msg1".into());
        assert!(removed.is_some());
        assert_eq!(queue.count_for_recipient(&recipient), 0);
    }

    #[test]
    fn test_queue_persistence() {
        let mut queue = MessageQueue::new();
        queue.enqueue(mock_message("msg1", "room1", 1000), PeerId("peer1".into()));
        queue.enqueue(mock_message("msg2", "room1", 2000), PeerId("peer2".into()));
        
        let bytes = queue.to_bytes().unwrap();
        let restored = MessageQueue::from_bytes(&bytes).unwrap();
        
        assert_eq!(restored.total_count(), 2);
    }

    #[test]
    fn test_history_merge() {
        let reconciler = HistoryReconciler::new();
        
        let local = vec![
            mock_message("msg1", "room1", 1000),
            mock_message("msg2", "room1", 2000),
        ];
        
        let remote = vec![
            mock_message("msg2", "room1", 2000),  // Duplicate
            mock_message("msg3", "room1", 3000),
        ];
        
        let merged = reconciler.merge_histories(local, remote);
        
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].id, "msg1");
        assert_eq!(merged[1].id, "msg2");
        assert_eq!(merged[2].id, "msg3");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 28: Message Queue Persistence
        #[test]
        fn prop_message_queue_persistence(
            num_messages in 1usize..20,
        ) {
            let mut queue = MessageQueue::new();
            
            for i in 0..num_messages {
                let msg = mock_message(&format!("msg{}", i), "room1", i as u64 * 1000);
                queue.enqueue(msg, PeerId(format!("peer{}", i % 3)));
            }
            
            let bytes = queue.to_bytes().unwrap();
            let restored = MessageQueue::from_bytes(&bytes).unwrap();
            
            prop_assert_eq!(queue.total_count(), restored.total_count());
        }

        /// Property 29: History Reconciliation Consistency
        #[test]
        fn prop_history_reconciliation_consistency(
            local_ids in prop::collection::vec("[a-z0-9]{8}", 1..10),
            remote_ids in prop::collection::vec("[a-z0-9]{8}", 1..10),
        ) {
            let reconciler = HistoryReconciler::new();
            
            let local: Vec<_> = local_ids.iter().enumerate()
                .map(|(i, id)| mock_message(id, "room1", i as u64 * 1000))
                .collect();
            
            let remote: Vec<_> = remote_ids.iter().enumerate()
                .map(|(i, id)| mock_message(id, "room1", (i + 100) as u64 * 1000))
                .collect();
            
            let merged = reconciler.merge_histories(local.clone(), remote.clone());
            
            // All unique messages should be present
            let unique_ids: std::collections::HashSet<_> = local_ids.iter()
                .chain(remote_ids.iter())
                .collect();
            
            prop_assert_eq!(merged.len(), unique_ids.len());
            
            // Should be sorted by timestamp
            for i in 1..merged.len() {
                prop_assert!(merged[i-1].timestamp <= merged[i].timestamp);
            }
        }
    }
}
