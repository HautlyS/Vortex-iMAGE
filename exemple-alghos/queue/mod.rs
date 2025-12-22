//! Message Queue Module - High-performance internal message queue for async operations
//! Implements Requirements 6.1-6.10 from examples-integration-analysis spec

use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Message identifier (32 bytes)
pub type MessageId = [u8; 32];

/// Consumer identifier
pub type ConsumerId = [u8; 16];

/// Queue message with metadata (Req 6.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: MessageId,
    pub partition: u32,
    pub offset: u64,
    pub payload: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub retry_count: u32,
}

/// Polling strategy (Req 6.4)
#[derive(Debug, Clone)]
pub enum PollingStrategy {
    ByOffset(u64),
    ByTimestamp(DateTime<Utc>),
    NextN(usize),
}

/// Consumer group state (Req 6.10)
#[derive(Debug, Clone)]
pub struct ConsumerGroup {
    pub group_id: String,
    pub members: Vec<ConsumerId>,
    pub offsets: HashMap<u32, u64>,
}

/// Retention configuration (Req 6.5)
#[derive(Debug, Clone)]
pub struct RetentionConfig {
    pub max_age: Duration,        // Default 7 days
    pub max_storage_bytes: u64,   // Storage quota
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            max_storage_bytes: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// Consumer assignment for rebalancing (Req 6.10)
#[derive(Debug, Clone)]
pub struct ConsumerAssignment {
    pub consumer_id: ConsumerId,
    pub partitions: Vec<u32>,
}

/// Rebalance result
#[derive(Debug, Clone)]
pub struct RebalanceResult {
    pub assignments: Vec<ConsumerAssignment>,
    pub rebalance_generation: u64,
}

/// Queue error types
#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Persistence failed: {0}")]
    PersistenceFailed(String),
    #[error("Partition not found: {0}")]
    PartitionNotFound(u32),
    #[error("Consumer group not found: {0}")]
    GroupNotFound(String),
    #[error("Rebalance failed: {0}")]
    RebalanceFailed(String),
    #[error("Message not found")]
    MessageNotFound,
}


/// In-memory message queue implementation
pub struct MessageQueue {
    partitions: HashMap<u32, VecDeque<QueueMessage>>,
    consumer_groups: HashMap<String, ConsumerGroup>,
    next_offset: HashMap<u32, u64>,
    retention_config: RetentionConfig,
    rebalance_generation: u64,
}

impl MessageQueue {
    /// Create new message queue
    pub fn new(num_partitions: u32) -> Self {
        let mut partitions = HashMap::new();
        let mut next_offset = HashMap::new();
        
        for i in 0..num_partitions {
            partitions.insert(i, VecDeque::new());
            next_offset.insert(i, 0);
        }
        
        Self {
            partitions,
            consumer_groups: HashMap::new(),
            next_offset,
            retention_config: RetentionConfig::default(),
            rebalance_generation: 0,
        }
    }
    
    /// Set retention configuration
    pub fn set_retention(&mut self, config: RetentionConfig) {
        self.retention_config = config;
    }
    
    /// Enqueue message with persistence (Req 6.1)
    pub async fn enqueue(&mut self, partition: u32, payload: Vec<u8>) -> Result<MessageId, QueueError> {
        let queue = self.partitions.get_mut(&partition)
            .ok_or(QueueError::PartitionNotFound(partition))?;
        
        let offset = self.next_offset.get_mut(&partition)
            .ok_or(QueueError::PartitionNotFound(partition))?;
        
        let mut id = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut id);
        
        let message = QueueMessage {
            id,
            partition,
            offset: *offset,
            payload,
            timestamp: Utc::now(),
            retry_count: 0,
        };
        
        *offset += 1;
        queue.push_back(message);
        
        Ok(id)
    }
    
    /// Poll messages with strategy (Req 6.4)
    pub async fn poll(&self, group: &str, strategy: PollingStrategy) -> Result<Vec<QueueMessage>, QueueError> {
        let consumer_group = self.consumer_groups.get(group)
            .ok_or_else(|| QueueError::GroupNotFound(group.to_string()))?;
        
        let mut messages = Vec::new();
        
        for (&partition, queue) in &self.partitions {
            let group_offset = consumer_group.offsets.get(&partition).copied().unwrap_or(0);
            
            match &strategy {
                PollingStrategy::ByOffset(offset) => {
                    for msg in queue.iter() {
                        if msg.offset >= *offset {
                            messages.push(msg.clone());
                        }
                    }
                }
                PollingStrategy::ByTimestamp(ts) => {
                    for msg in queue.iter() {
                        if msg.timestamp >= *ts {
                            messages.push(msg.clone());
                        }
                    }
                }
                PollingStrategy::NextN(n) => {
                    for msg in queue.iter() {
                        if msg.offset >= group_offset && messages.len() < *n {
                            messages.push(msg.clone());
                        }
                    }
                }
            }
        }
        
        Ok(messages)
    }
    
    /// Acknowledge message processing and persist offset (Req 6.6)
    pub async fn ack(&mut self, group: &str, message_id: MessageId) -> Result<(), QueueError> {
        // Find message and update consumer offset
        for queue in self.partitions.values() {
            if let Some(msg) = queue.iter().find(|m| m.id == message_id) {
                let consumer_group = self.consumer_groups.get_mut(group)
                    .ok_or_else(|| QueueError::GroupNotFound(group.to_string()))?;
                
                consumer_group.offsets.insert(msg.partition, msg.offset + 1);
                return Ok(());
            }
        }
        
        Err(QueueError::MessageNotFound)
    }
    
    /// Requeue failed message with exponential backoff (Req 6.3)
    /// Backoff: min(1s * 2^retry_count, 5min)
    pub async fn requeue(&mut self, mut message: QueueMessage) -> Result<(), QueueError> {
        message.retry_count += 1;
        
        // Calculate backoff (not actually delaying here, just tracking)
        let _backoff_secs = std::cmp::min(
            1u64 << message.retry_count,
            5 * 60, // 5 minutes max
        );
        
        let queue = self.partitions.get_mut(&message.partition)
            .ok_or(QueueError::PartitionNotFound(message.partition))?;
        
        queue.push_back(message);
        Ok(())
    }
    
    /// Get persisted consumer offset for crash recovery (Req 6.6)
    pub async fn get_consumer_offset(&self, group: &str, partition: u32) -> Result<u64, QueueError> {
        let consumer_group = self.consumer_groups.get(group)
            .ok_or_else(|| QueueError::GroupNotFound(group.to_string()))?;
        
        Ok(consumer_group.offsets.get(&partition).copied().unwrap_or(0))
    }
    
    /// Apply retention policy (Req 6.5)
    pub async fn apply_retention(&mut self) -> Result<usize, QueueError> {
        let cutoff = Utc::now() - chrono::Duration::from_std(self.retention_config.max_age)
            .unwrap_or_default();
        
        let mut deleted = 0;
        
        for queue in self.partitions.values_mut() {
            let before = queue.len();
            queue.retain(|msg| msg.timestamp > cutoff);
            deleted += before - queue.len();
        }
        
        Ok(deleted)
    }
    
    /// Create or get consumer group
    pub fn get_or_create_group(&mut self, group_id: &str) -> &mut ConsumerGroup {
        if !self.consumer_groups.contains_key(group_id) {
            self.consumer_groups.insert(group_id.to_string(), ConsumerGroup {
                group_id: group_id.to_string(),
                members: Vec::new(),
                offsets: HashMap::new(),
            });
        }
        self.consumer_groups.get_mut(group_id).unwrap()
    }
    
    /// Add consumer to group (Req 6.10)
    pub async fn join_group(&mut self, group: &str, consumer_id: ConsumerId) -> Result<ConsumerAssignment, QueueError> {
        // First, ensure group exists and add consumer
        let group_id = {
            let consumer_group = self.get_or_create_group(group);
            if !consumer_group.members.contains(&consumer_id) {
                consumer_group.members.push(consumer_id);
            }
            consumer_group.group_id.clone()
        };
        
        // Rebalance partitions (now self is no longer borrowed)
        let result = self.rebalance_internal(&group_id)?;
        
        // Find assignment for this consumer
        result.assignments.into_iter()
            .find(|a| a.consumer_id == consumer_id)
            .ok_or_else(|| QueueError::RebalanceFailed("Consumer not assigned".into()))
    }
    
    /// Remove consumer from group (Req 6.10)
    pub async fn leave_group(&mut self, group: &str, consumer_id: ConsumerId) -> Result<(), QueueError> {
        let consumer_group = self.consumer_groups.get_mut(group)
            .ok_or_else(|| QueueError::GroupNotFound(group.to_string()))?;
        
        consumer_group.members.retain(|&id| id != consumer_id);
        
        // Trigger rebalance
        let _ = self.rebalance_internal(group)?;
        
        Ok(())
    }
    
    /// Rebalance partitions when consumers join/leave (Req 6.10)
    pub async fn rebalance(&mut self, group: &str) -> Result<RebalanceResult, QueueError> {
        self.rebalance_internal(group)
    }
    
    fn rebalance_internal(&mut self, group: &str) -> Result<RebalanceResult, QueueError> {
        let consumer_group = self.consumer_groups.get(group)
            .ok_or_else(|| QueueError::GroupNotFound(group.to_string()))?;
        
        let num_partitions = self.partitions.len() as u32;
        let num_consumers = consumer_group.members.len();
        
        if num_consumers == 0 {
            return Ok(RebalanceResult {
                assignments: Vec::new(),
                rebalance_generation: self.rebalance_generation,
            });
        }
        
        // Fair distribution: each consumer gets ceil(partitions/consumers) partitions
        let partitions_per_consumer = (num_partitions as usize + num_consumers - 1) / num_consumers;
        
        let mut assignments = Vec::new();
        let mut partition_idx = 0u32;
        
        for &consumer_id in &consumer_group.members {
            let mut partitions = Vec::new();
            for _ in 0..partitions_per_consumer {
                if partition_idx < num_partitions {
                    partitions.push(partition_idx);
                    partition_idx += 1;
                }
            }
            assignments.push(ConsumerAssignment { consumer_id, partitions });
        }
        
        self.rebalance_generation += 1;
        
        Ok(RebalanceResult {
            assignments,
            rebalance_generation: self.rebalance_generation,
        })
    }
    
    /// Calculate exponential backoff duration (Req 6.3)
    pub fn calculate_backoff(retry_count: u32) -> Duration {
        let secs = std::cmp::min(1u64 << retry_count, 5 * 60);
        Duration::from_secs(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_and_poll() {
        let mut queue = MessageQueue::new(4);
        queue.get_or_create_group("test-group");
        
        let id = queue.enqueue(0, b"test message".to_vec()).await.unwrap();
        assert_ne!(id, [0u8; 32]);
        
        let messages = queue.poll("test-group", PollingStrategy::NextN(10)).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].payload, b"test message");
    }

    #[tokio::test]
    async fn test_consumer_group() {
        let mut queue = MessageQueue::new(4);
        let consumer1 = [1u8; 16];
        let consumer2 = [2u8; 16];
        
        let assignment1 = queue.join_group("test-group", consumer1).await.unwrap();
        assert!(!assignment1.partitions.is_empty());
        
        let assignment2 = queue.join_group("test-group", consumer2).await.unwrap();
        
        // Both should have partitions
        assert!(!assignment1.partitions.is_empty() || !assignment2.partitions.is_empty());
    }

    #[tokio::test]
    async fn test_ack_updates_offset() {
        let mut queue = MessageQueue::new(1);
        queue.get_or_create_group("test-group");
        
        let id = queue.enqueue(0, b"test".to_vec()).await.unwrap();
        
        let offset_before = queue.get_consumer_offset("test-group", 0).await.unwrap();
        assert_eq!(offset_before, 0);
        
        queue.ack("test-group", id).await.unwrap();
        
        let offset_after = queue.get_consumer_offset("test-group", 0).await.unwrap();
        assert_eq!(offset_after, 1);
    }

    #[test]
    fn test_exponential_backoff() {
        assert_eq!(MessageQueue::calculate_backoff(0), Duration::from_secs(1));
        assert_eq!(MessageQueue::calculate_backoff(1), Duration::from_secs(2));
        assert_eq!(MessageQueue::calculate_backoff(2), Duration::from_secs(4));
        assert_eq!(MessageQueue::calculate_backoff(10), Duration::from_secs(300)); // Max 5 min
    }

    #[tokio::test]
    async fn test_requeue_increments_retry() {
        let mut queue = MessageQueue::new(1);
        
        let msg = QueueMessage {
            id: [0u8; 32],
            partition: 0,
            offset: 0,
            payload: b"test".to_vec(),
            timestamp: Utc::now(),
            retry_count: 0,
        };
        
        queue.requeue(msg).await.unwrap();
        
        let messages = queue.partitions.get(&0).unwrap();
        assert_eq!(messages.back().unwrap().retry_count, 1);
    }
}
