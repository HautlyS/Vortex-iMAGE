//! Transfer management for file uploads/downloads
//! Supports chunked transfers with 64KB segments for resumable downloads
use serde::{Deserialize, Serialize};

/// Chunk size for segmented transfers (64KB)
pub const CHUNK_SIZE: usize = 64 * 1024;

/// Transfer status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,
    Active,
    Paused,
    Complete,
    Error(String),
}

/// Transfer direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransferDirection {
    Upload,
    Download,
}

/// A file transfer with chunking support
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub id: String,
    pub name: String,
    pub total_bytes: u64,
    pub bytes_transferred: u64,
    pub status: TransferStatus,
    pub direction: TransferDirection,
    pub ticket: Option<String>,
    pub hash: Option<[u8; 32]>,  // BLAKE3 hash for integrity
    pub chunks_completed: Vec<bool>,  // Track completed chunks for resumable transfers
    pub chunk_tickets: Vec<String>,  // Ticket per chunk for parallel download
}

impl Transfer {
    pub fn new_upload(id: String, name: String, size: u64) -> Self {
        let num_chunks = ((size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE).max(1);
        Self {
            id,
            name,
            total_bytes: size,
            bytes_transferred: 0,
            status: TransferStatus::Pending,
            direction: TransferDirection::Upload,
            ticket: None,
            hash: None,
            chunks_completed: vec![false; num_chunks],
            chunk_tickets: Vec::new(),
        }
    }

    pub fn new_download(id: String, ticket: String, total_bytes: u64) -> Self {
        let num_chunks = ((total_bytes as usize + CHUNK_SIZE - 1) / CHUNK_SIZE).max(1);
        Self {
            id,
            name: "Downloading...".into(),
            total_bytes,
            bytes_transferred: 0,
            status: TransferStatus::Pending,
            direction: TransferDirection::Download,
            ticket: Some(ticket),
            hash: None,
            chunks_completed: vec![false; num_chunks],
            chunk_tickets: Vec::new(),
        }
    }

    pub fn progress(&self) -> u8 {
        if self.total_bytes == 0 { return 0; }
        ((self.bytes_transferred * 100) / self.total_bytes) as u8
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TransferStatus::Active)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, TransferStatus::Complete)
    }

    /// Get number of chunks
    pub fn num_chunks(&self) -> usize {
        self.chunks_completed.len()
    }

    /// Mark a chunk as completed
    pub fn complete_chunk(&mut self, chunk_index: usize) {
        if chunk_index < self.chunks_completed.len() {
            self.chunks_completed[chunk_index] = true;
            self.bytes_transferred = self.completed_bytes();
        }
    }

    /// Get number of completed chunks
    pub fn completed_chunks(&self) -> usize {
        self.chunks_completed.iter().filter(|&&c| c).count()
    }

    /// Calculate bytes transferred based on completed chunks
    pub fn completed_bytes(&self) -> u64 {
        let completed = self.completed_chunks();
        let full_chunks = completed.saturating_sub(1);
        let last_chunk_size = if completed > 0 && completed == self.num_chunks() {
            // Last chunk may be smaller
            (self.total_bytes as usize % CHUNK_SIZE).max(1)
        } else if completed > 0 {
            CHUNK_SIZE
        } else {
            0
        };
        (full_chunks * CHUNK_SIZE + last_chunk_size) as u64
    }

    /// Get indices of incomplete chunks (for resumable download)
    pub fn incomplete_chunks(&self) -> Vec<usize> {
        self.chunks_completed
            .iter()
            .enumerate()
            .filter(|(_, completed)| !**completed)
            .map(|(i, _)| i)
            .collect()
    }

    /// Check if transfer can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(self.status, TransferStatus::Paused | TransferStatus::Error(_))
            && self.completed_chunks() > 0
            && self.completed_chunks() < self.num_chunks()
    }

    /// Set hash for integrity verification
    pub fn set_hash(&mut self, hash: [u8; 32]) {
        self.hash = Some(hash);
    }

    /// Verify hash matches
    pub fn verify_hash(&self, computed_hash: &[u8; 32]) -> bool {
        self.hash.as_ref().map(|h| h == computed_hash).unwrap_or(true)
    }
}

/// Split data into 64KB chunks
pub fn split_into_chunks(data: &[u8]) -> Vec<&[u8]> {
    data.chunks(CHUNK_SIZE).collect()
}

/// Reassemble chunks into original data
pub fn reassemble_chunks(chunks: Vec<Vec<u8>>) -> Vec<u8> {
    chunks.into_iter().flatten().collect()
}

/// Compute BLAKE3 hash of data
pub fn compute_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Verify data integrity with BLAKE3
pub fn verify_integrity(data: &[u8], expected_hash: &[u8; 32]) -> bool {
    &compute_hash(data) == expected_hash
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// Transfer manager for handling multiple concurrent transfers
pub struct TransferManager {
    transfers: Vec<Transfer>,
    max_concurrent: usize,
}

impl TransferManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            transfers: Vec::new(),
            max_concurrent,
        }
    }

    pub fn add(&mut self, transfer: Transfer) -> &Transfer {
        self.transfers.push(transfer);
        self.transfers.last().unwrap()
    }

    pub fn get(&self, id: &str) -> Option<&Transfer> {
        self.transfers.iter().find(|t| t.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Transfer> {
        self.transfers.iter_mut().find(|t| t.id == id)
    }

    pub fn remove(&mut self, id: &str) -> Option<Transfer> {
        if let Some(idx) = self.transfers.iter().position(|t| t.id == id) {
            Some(self.transfers.remove(idx))
        } else {
            None
        }
    }

    pub fn active_count(&self) -> usize {
        self.transfers.iter().filter(|t| t.is_active()).count()
    }

    pub fn can_start_new(&self) -> bool {
        self.active_count() < self.max_concurrent
    }

    pub fn pending(&self) -> impl Iterator<Item = &Transfer> {
        self.transfers.iter().filter(|t| matches!(t.status, TransferStatus::Pending))
    }

    pub fn all(&self) -> &[Transfer] {
        &self.transfers
    }

    pub fn update_progress(&mut self, id: &str, bytes: u64, total: u64) {
        if let Some(t) = self.get_mut(id) {
            t.bytes_transferred = bytes;
            t.total_bytes = total;
        }
    }

    pub fn set_status(&mut self, id: &str, status: TransferStatus) {
        if let Some(t) = self.get_mut(id) {
            t.status = status;
        }
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_chunk_calculation() {
        // 100KB file = 2 chunks (64KB + 36KB)
        let transfer = Transfer::new_upload("t1".into(), "file.txt".into(), 100 * 1024);
        assert_eq!(transfer.num_chunks(), 2);
        
        // 64KB file = 1 chunk
        let transfer = Transfer::new_upload("t2".into(), "file.txt".into(), 64 * 1024);
        assert_eq!(transfer.num_chunks(), 1);
        
        // 1 byte file = 1 chunk
        let transfer = Transfer::new_upload("t3".into(), "file.txt".into(), 1);
        assert_eq!(transfer.num_chunks(), 1);
    }

    #[test]
    fn test_chunk_completion() {
        let mut transfer = Transfer::new_upload("t1".into(), "file.txt".into(), 100 * 1024);
        
        assert_eq!(transfer.completed_chunks(), 0);
        
        transfer.complete_chunk(0);
        assert_eq!(transfer.completed_chunks(), 1);
        
        transfer.complete_chunk(1);
        assert_eq!(transfer.completed_chunks(), 2);
        assert_eq!(transfer.progress(), 100);
    }

    #[test]
    fn test_incomplete_chunks() {
        let mut transfer = Transfer::new_upload("t1".into(), "file.txt".into(), 200 * 1024);
        // 200KB = 4 chunks (64 + 64 + 64 + 8)
        
        transfer.complete_chunk(0);
        transfer.complete_chunk(2);
        
        let incomplete = transfer.incomplete_chunks();
        assert_eq!(incomplete, vec![1, 3]);
    }

    #[test]
    fn test_can_resume() {
        let mut transfer = Transfer::new_upload("t1".into(), "file.txt".into(), 200 * 1024);
        
        // Can't resume if no chunks completed
        transfer.status = TransferStatus::Paused;
        assert!(!transfer.can_resume());
        
        // Can resume if some chunks completed
        transfer.complete_chunk(0);
        assert!(transfer.can_resume());
        
        // Can't resume if all chunks completed
        transfer.complete_chunk(1);
        transfer.complete_chunk(2);
        transfer.complete_chunk(3);
        assert!(!transfer.can_resume());
    }

    #[test]
    fn test_hash_verification() {
        let data = b"test data for hashing";
        let hash = compute_hash(data);
        
        assert!(verify_integrity(data, &hash));
        assert!(!verify_integrity(b"different data", &hash));
    }

    #[test]
    fn test_split_and_reassemble() {
        let data: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();
        let chunks = split_into_chunks(&data);
        
        assert_eq!(chunks.len(), 4);  // 200KB / 64KB = 4 chunks
        
        let reassembled = reassemble_chunks(chunks.into_iter().map(|c| c.to_vec()).collect());
        assert_eq!(data, reassembled);
    }

    #[test]
    fn test_transfer_manager_concurrent_limit() {
        let mut manager = TransferManager::new(3);
        
        // Add 5 transfers
        for i in 0..5 {
            let mut t = Transfer::new_upload(format!("t{}", i), "file.txt".into(), 1000);
            if i < 3 {
                t.status = TransferStatus::Active;
            }
            manager.add(t);
        }
        
        assert_eq!(manager.active_count(), 3);
        assert!(!manager.can_start_new());
        
        // Complete one
        manager.set_status("t0", TransferStatus::Complete);
        assert_eq!(manager.active_count(), 2);
        assert!(manager.can_start_new());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 10: File Sharing Round-Trip (hash verification)
        #[test]
        fn prop_file_hash_roundtrip(
            data in prop::collection::vec(any::<u8>(), 1..10000),
        ) {
            let hash = compute_hash(&data);
            prop_assert!(verify_integrity(&data, &hash));
        }

        /// Property 11: Hash Verification Integrity
        #[test]
        fn prop_hash_verification_integrity(
            data in prop::collection::vec(any::<u8>(), 1..1000),
            tamper_pos in 0usize..1000,
        ) {
            let hash = compute_hash(&data);
            
            // Original data should verify
            prop_assert!(verify_integrity(&data, &hash));
            
            // Tampered data should not verify
            if !data.is_empty() {
                let mut tampered = data.clone();
                let pos = tamper_pos % tampered.len();
                tampered[pos] ^= 0xFF;
                prop_assert!(!verify_integrity(&tampered, &hash));
            }
        }

        /// Property 12: Concurrent Transfer Limit
        #[test]
        fn prop_concurrent_transfer_limit(
            num_transfers in 1usize..20,
            max_concurrent in 1usize..5,
        ) {
            let mut manager = TransferManager::new(max_concurrent);
            
            for i in 0..num_transfers {
                let mut t = Transfer::new_upload(format!("t{}", i), "file.txt".into(), 1000);
                if i < max_concurrent {
                    t.status = TransferStatus::Active;
                }
                manager.add(t);
            }
            
            prop_assert!(manager.active_count() <= max_concurrent);
        }

        /// Property: Chunk split and reassemble
        #[test]
        fn prop_chunk_split_reassemble(
            data in prop::collection::vec(any::<u8>(), 0..500000),
        ) {
            let chunks = split_into_chunks(&data);
            let reassembled = reassemble_chunks(chunks.into_iter().map(|c| c.to_vec()).collect());
            prop_assert_eq!(data, reassembled);
        }
    }
}
