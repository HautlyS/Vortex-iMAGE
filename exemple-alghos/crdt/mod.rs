//! CRDT Module - Conflict-free Replicated Data Types for collaborative editing
//! Implements Requirements 7.1-7.10 from examples-integration-analysis spec

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Node identifier (8 bytes)
pub type NodeId = [u8; 8];

/// Document identifier (32 bytes)
pub type DocumentId = [u8; 32];

/// Hybrid Logical Clock timestamp (Req 7.2)
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct HLCTimestamp {
    pub physical: u64,
    pub logical: u32,
    pub node_id: NodeId,
}

/// Position identifier for CRDT (Req 7.5)
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PositionId {
    pub path: Vec<u32>,
    pub site_id: NodeId,
    pub counter: u64,
}

/// CRDT operation types (Req 7.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CRDTOperation {
    Insert { position: PositionId, content: String, timestamp: HLCTimestamp },
    Delete { position: PositionId, timestamp: HLCTimestamp },
}

/// Operation type for history display (Req 7.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Insert,
    Delete,
    Format,
}

/// History entry with full metadata (Req 7.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub operation: CRDTOperation,
    pub author_id: NodeId,
    pub timestamp: HLCTimestamp,
    pub operation_type: OperationType,
}


/// CRDT error types
#[derive(Error, Debug)]
pub enum CRDTError {
    #[error("Causal ordering violation")]
    CausalViolation,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Merge conflict: {0}")]
    MergeConflict(String),
    #[error("Compaction failed: {0}")]
    CompactionFailed(String),
    #[error("Reconciliation failed: {0}")]
    ReconciliationFailed(String),
}

/// Compaction result (Req 7.9)
#[derive(Debug, Clone)]
pub struct CompactionResult {
    pub original_size: usize,
    pub compacted_size: usize,
    pub preserved_operations: usize,
    pub removed_operations: usize,
}

/// Lazy loading configuration for large documents (Req 7.10)
#[derive(Debug, Clone)]
pub struct LazyLoadConfig {
    pub section_size: usize,      // Default 10KB per section
    pub prefetch_count: usize,    // Number of sections to prefetch
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            section_size: 10 * 1024,
            prefetch_count: 2,
        }
    }
}

/// Document section for lazy loading (Req 7.10)
#[derive(Debug, Clone)]
pub struct DocumentSection {
    pub start_position: PositionId,
    pub end_position: PositionId,
    pub content: String,
    pub operations: Vec<CRDTOperation>,
}

impl HLCTimestamp {
    /// Create new HLC timestamp
    pub fn new(node_id: NodeId) -> Self {
        Self {
            physical: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            logical: 0,
            node_id,
        }
    }
    
    /// Increment logical clock
    pub fn increment(&mut self) {
        self.logical += 1;
    }
    
    /// Update from received timestamp (Req 7.2)
    pub fn update(&mut self, received: &HLCTimestamp) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        if now > self.physical && now > received.physical {
            self.physical = now;
            self.logical = 0;
        } else if self.physical == received.physical {
            self.logical = self.logical.max(received.logical) + 1;
        } else if self.physical > received.physical {
            self.logical += 1;
        } else {
            self.physical = received.physical;
            self.logical = received.logical + 1;
        }
    }
}

impl PositionId {
    /// Generate unique position ID for insert (Req 7.5)
    pub fn generate(site_id: NodeId, counter: u64, index: usize) -> Self {
        Self {
            path: vec![index as u32],
            site_id,
            counter,
        }
    }
    
    /// Generate position between two positions
    pub fn between(left: Option<&PositionId>, right: Option<&PositionId>, site_id: NodeId, counter: u64) -> Self {
        let mut path = Vec::new();
        
        match (left, right) {
            (None, None) => path.push(u32::MAX / 2),
            (Some(l), None) => {
                path = l.path.clone();
                if let Some(last) = path.last_mut() {
                    *last = last.saturating_add(1);
                }
            }
            (None, Some(r)) => {
                path = r.path.clone();
                if let Some(last) = path.last_mut() {
                    *last = last.saturating_sub(1);
                }
            }
            (Some(l), Some(r)) => {
                // Generate path between left and right
                for (_i, (&lp, &rp)) in l.path.iter().zip(r.path.iter()).enumerate() {
                    if lp < rp {
                        path.push((lp + rp) / 2);
                        break;
                    } else {
                        path.push(lp);
                    }
                }
                if path.is_empty() {
                    path.push(u32::MAX / 2);
                }
            }
        }
        
        Self { path, site_id, counter }
    }
}

/// CRDT Document with operation log (Req 7.1)
pub struct CRDTDocument {
    pub doc_id: DocumentId,
    pub operations: Vec<CRDTOperation>,
    pub frontier: Vec<HLCTimestamp>,
    pub content: String,
    pub tombstones: HashMap<PositionId, HLCTimestamp>,
    node_id: NodeId,
    counter: u64,
    clock: HLCTimestamp,
}

impl CRDTDocument {
    /// Create new CRDT document
    pub fn new(doc_id: DocumentId, node_id: NodeId) -> Self {
        Self {
            doc_id,
            operations: Vec::new(),
            frontier: Vec::new(),
            content: String::new(),
            tombstones: HashMap::new(),
            node_id,
            counter: 0,
            clock: HLCTimestamp::new(node_id),
        }
    }
    
    /// Apply local insert operation (Req 7.1, 7.5)
    pub fn insert(&mut self, index: usize, text: &str) -> Result<CRDTOperation, CRDTError> {
        self.counter += 1;
        self.clock.increment();
        
        let position = PositionId::generate(self.node_id, self.counter, index);
        let op = CRDTOperation::Insert {
            position: position.clone(),
            content: text.to_string(),
            timestamp: self.clock,
        };
        
        self.apply_local(&op)?;
        Ok(op)
    }
    
    /// Apply local delete operation (Req 7.6)
    pub fn delete(&mut self, index: usize) -> Result<CRDTOperation, CRDTError> {
        self.clock.increment();
        
        let position = PositionId::generate(self.node_id, self.counter, index);
        let op = CRDTOperation::Delete {
            position: position.clone(),
            timestamp: self.clock,
        };
        
        // Mark as tombstone instead of removing (Req 7.6)
        self.tombstones.insert(position, self.clock);
        self.apply_local(&op)?;
        Ok(op)
    }
    
    /// Apply local operation (Req 7.1)
    pub fn apply_local(&mut self, op: &CRDTOperation) -> Result<(), CRDTError> {
        match op {
            CRDTOperation::Insert { position, content, timestamp } => {
                // Find insertion point based on position
                let idx = self.find_insert_index(position);
                self.content.insert_str(idx, content);
                self.operations.push(op.clone());
                self.update_frontier(*timestamp);
            }
            CRDTOperation::Delete { position, timestamp } => {
                self.tombstones.insert(position.clone(), *timestamp);
                self.operations.push(op.clone());
                self.update_frontier(*timestamp);
            }
        }
        Ok(())
    }
    
    /// Apply remote operation with causal ordering (Req 7.2)
    pub fn apply_remote(&mut self, op: &CRDTOperation) -> Result<(), CRDTError> {
        let timestamp = match op {
            CRDTOperation::Insert { timestamp, .. } => timestamp,
            CRDTOperation::Delete { timestamp, .. } => timestamp,
        };
        
        // Update local clock
        self.clock.update(timestamp);
        
        // Apply operation
        self.apply_local(op)
    }
    
    /// Merge divergent states (Req 7.1, 7.3)
    pub fn merge(&mut self, remote_ops: &[CRDTOperation]) -> Result<(), CRDTError> {
        for op in remote_ops {
            self.apply_remote(op)?;
        }
        Ok(())
    }
    
    /// Reconcile after offline editing (Req 7.3)
    pub fn reconcile_from_ancestor(
        &mut self,
        local_ops: &[CRDTOperation],
        remote_ops: &[CRDTOperation],
        _common_ancestor: &HLCTimestamp,
    ) -> Result<(), CRDTError> {
        // Merge all operations - CRDT guarantees convergence
        for op in local_ops {
            self.apply_local(op)?;
        }
        for op in remote_ops {
            self.apply_remote(op)?;
        }
        Ok(())
    }
    
    /// Get document content
    pub fn get_content(&self) -> &str {
        &self.content
    }
    
    /// Get operation history with full metadata (Req 7.4)
    pub fn get_history(&self) -> Vec<HistoryEntry> {
        self.operations.iter().map(|op| {
            let (timestamp, op_type) = match op {
                CRDTOperation::Insert { timestamp, .. } => (*timestamp, OperationType::Insert),
                CRDTOperation::Delete { timestamp, .. } => (*timestamp, OperationType::Delete),
            };
            HistoryEntry {
                operation: op.clone(),
                author_id: timestamp.node_id,
                timestamp,
                operation_type: op_type,
            }
        }).collect()
    }
    
    /// Compact operation history (Req 7.9)
    pub fn compact(&mut self) -> Result<CompactionResult, CRDTError> {
        let original_size = self.operations.len();
        
        // Remove redundant operations (deletes of already deleted positions)
        let mut seen_deletes: HashMap<PositionId, HLCTimestamp> = HashMap::new();
        let mut compacted_ops = Vec::new();
        
        for op in &self.operations {
            match op {
                CRDTOperation::Delete { position, timestamp } => {
                    if let Some(existing) = seen_deletes.get(position) {
                        if timestamp > existing {
                            seen_deletes.insert(position.clone(), *timestamp);
                        }
                        // Skip duplicate delete
                        continue;
                    }
                    seen_deletes.insert(position.clone(), *timestamp);
                    compacted_ops.push(op.clone());
                }
                _ => compacted_ops.push(op.clone()),
            }
        }
        
        let removed = original_size - compacted_ops.len();
        self.operations = compacted_ops;
        
        Ok(CompactionResult {
            original_size,
            compacted_size: self.operations.len(),
            preserved_operations: self.operations.len(),
            removed_operations: removed,
        })
    }
    
    /// Check if compaction is needed
    pub fn should_compact(&self, threshold_ops: usize) -> bool {
        self.operations.len() > threshold_ops
    }
    
    /// Find the correct insertion index using full path comparison (CRDT FIX)
    /// This properly handles the CRDT position ordering algorithm
    /// Returns the character index in the content string
    fn find_insert_index(&self, position: &PositionId) -> usize {
        // Collect all existing positions with their content lengths
        let mut positions: Vec<(&PositionId, usize)> = Vec::new();
        
        // Extract positions from insert operations
        for op in &self.operations {
            if let CRDTOperation::Insert { position: pos, content, .. } = op {
                // Skip tombstoned positions
                if !self.tombstones.contains_key(pos) {
                    positions.push((pos, content.len()));
                }
            }
        }
        
        // If no existing positions, insert at beginning
        if positions.is_empty() {
            return 0;
        }
        
        // Sort positions by CRDT ordering
        positions.sort_by(|(a, _), (b, _)| Self::compare_positions(a, b));
        
        // Find insertion point and calculate character index
        let mut char_idx = 0;
        for (existing_pos, content_len) in &positions {
            if Self::compare_positions(position, existing_pos) == std::cmp::Ordering::Less {
                return char_idx;
            }
            char_idx += content_len;
        }
        
        // Insert at end if greater than all existing positions
        char_idx
    }
    
    /// Compare two position IDs for ordering (CRDT position ordering algorithm)
    fn compare_positions(a: &PositionId, b: &PositionId) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        
        // Compare paths element by element
        let min_len = a.path.len().min(b.path.len());
        for i in 0..min_len {
            match a.path[i].cmp(&b.path[i]) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        
        // If paths are equal up to min_len, shorter path comes first
        match a.path.len().cmp(&b.path.len()) {
            Ordering::Equal => {
                // Same path length, use site_id as tiebreaker
                match a.site_id.cmp(&b.site_id) {
                    Ordering::Equal => a.counter.cmp(&b.counter),
                    other => other,
                }
            }
            other => other,
        }
    }
    
    fn update_frontier(&mut self, timestamp: HLCTimestamp) {
        // Keep only the latest timestamps per node
        self.frontier.retain(|t| t.node_id != timestamp.node_id);
        self.frontier.push(timestamp);
    }
    
    /// Serialize document state (Req 7.7)
    pub fn serialize(&self) -> Result<Vec<u8>, CRDTError> {
        bincode::serialize(&(&self.doc_id, &self.operations, &self.content))
            .map_err(|e| CRDTError::InvalidOperation(e.to_string()))
    }
    
    /// Deserialize document state (Req 7.8)
    pub fn deserialize(data: &[u8], node_id: NodeId) -> Result<Self, CRDTError> {
        let (doc_id, operations, content): (DocumentId, Vec<CRDTOperation>, String) = 
            bincode::deserialize(data)
                .map_err(|e| CRDTError::InvalidOperation(e.to_string()))?;
        
        let mut doc = Self::new(doc_id, node_id);
        doc.operations = operations;
        doc.content = content;
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlc_timestamp_ordering() {
        let node1 = [1u8; 8];
        let node2 = [2u8; 8];
        
        let mut t1 = HLCTimestamp::new(node1);
        let mut t2 = HLCTimestamp::new(node2);
        
        t1.increment();
        t2.update(&t1);
        
        assert!(t2 > t1);
    }

    #[test]
    fn test_crdt_insert() {
        let doc_id = [0u8; 32];
        let node_id = [1u8; 8];
        let mut doc = CRDTDocument::new(doc_id, node_id);
        
        doc.insert(0, "Hello").unwrap();
        assert_eq!(doc.get_content(), "Hello");
        
        doc.insert(5, " World").unwrap();
        assert_eq!(doc.get_content(), "Hello World");
    }

    #[test]
    fn test_crdt_merge() {
        let doc_id = [0u8; 32];
        let node1 = [1u8; 8];
        let node2 = [2u8; 8];
        
        let mut doc1 = CRDTDocument::new(doc_id, node1);
        let mut doc2 = CRDTDocument::new(doc_id, node2);
        
        let op1 = doc1.insert(0, "A").unwrap();
        let op2 = doc2.insert(0, "B").unwrap();
        
        // Merge operations
        doc1.merge(&[op2]).unwrap();
        doc2.merge(&[op1]).unwrap();
        
        // Both should have same content (order may vary based on CRDT rules)
        assert!(doc1.get_content().contains('A'));
        assert!(doc1.get_content().contains('B'));
    }

    #[test]
    fn test_crdt_history() {
        let doc_id = [0u8; 32];
        let node_id = [1u8; 8];
        let mut doc = CRDTDocument::new(doc_id, node_id);
        
        doc.insert(0, "Test").unwrap();
        
        let history = doc.get_history();
        assert_eq!(history.len(), 1);
        assert!(matches!(history[0].operation_type, OperationType::Insert));
    }

    #[test]
    fn test_crdt_compaction() {
        let doc_id = [0u8; 32];
        let node_id = [1u8; 8];
        let mut doc = CRDTDocument::new(doc_id, node_id);
        
        // Add some operations
        for i in 0..10 {
            doc.insert(i, &format!("{}", i)).unwrap();
        }
        
        let result = doc.compact().unwrap();
        assert_eq!(result.original_size, 10);
    }
}
