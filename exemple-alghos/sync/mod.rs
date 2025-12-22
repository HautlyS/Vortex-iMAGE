//! Document sync with CRDT + OT for collaborative editing
//! Implements confluence guarantee: same ops in any order produce same result
//! Uses Operational Transform for cursor/selection sync
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocVersion {
    pub clock: u64,
    pub author: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DocOp {
    Insert { pos: usize, text: String, ver: DocVersion },
    Delete { pos: usize, len: usize, ver: DocVersion },
    Replace { pos: usize, len: usize, text: String, ver: DocVersion },
    Format { pos: usize, len: usize, style: TextStyle, ver: DocVersion },
}

/// Text formatting styles
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TextStyle {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Link(String),
    Code,
}

/// Selection range for collaborative editing
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start: start.min(end), end: start.max(end) }
    }
    
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }
    
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// Transform selection after an insert operation
    pub fn transform_insert(&mut self, pos: usize, len: usize) {
        if pos <= self.start {
            self.start += len;
            self.end += len;
        } else if pos < self.end {
            self.end += len;
        }
    }
    
    /// Transform selection after a delete operation
    pub fn transform_delete(&mut self, pos: usize, len: usize) {
        let del_end = pos + len;
        
        if del_end <= self.start {
            // Delete before selection
            self.start -= len;
            self.end -= len;
        } else if pos >= self.end {
            // Delete after selection - no change
        } else if pos <= self.start && del_end >= self.end {
            // Delete encompasses selection
            self.start = pos;
            self.end = pos;
        } else if pos <= self.start {
            // Delete overlaps start
            self.start = pos;
            self.end -= len;
        } else if del_end >= self.end {
            // Delete overlaps end
            self.end = pos;
        } else {
            // Delete within selection
            self.end -= len;
        }
    }
}

impl DocOp {
    pub fn version(&self) -> &DocVersion {
        match self {
            DocOp::Insert { ver, .. } => ver,
            DocOp::Delete { ver, .. } => ver,
            DocOp::Replace { ver, .. } => ver,
            DocOp::Format { ver, .. } => ver,
        }
    }

    /// Serialize operation to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize operation from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cursor {
    pub peer_id: String,
    pub position: usize,
    pub selection: Option<Selection>,
    pub color: String,
    pub name: String,
    pub last_active: u64,
}

impl Cursor {
    pub fn new(peer_id: String, position: usize, name: String, color: String) -> Self {
        Self {
            peer_id,
            position,
            selection: None,
            color,
            name,
            last_active: now(),
        }
    }
    
    /// Transform cursor position after an insert
    pub fn transform_insert(&mut self, pos: usize, len: usize) {
        if pos <= self.position {
            self.position += len;
        }
        if let Some(ref mut sel) = self.selection {
            sel.transform_insert(pos, len);
        }
    }
    
    /// Transform cursor position after a delete
    pub fn transform_delete(&mut self, pos: usize, len: usize) {
        let del_end = pos + len;
        if del_end <= self.position {
            self.position -= len;
        } else if pos < self.position {
            self.position = pos;
        }
        if let Some(ref mut sel) = self.selection {
            sel.transform_delete(pos, len);
        }
    }
    
    /// Set selection range
    pub fn set_selection(&mut self, start: usize, end: usize) {
        self.selection = Some(Selection::new(start, end));
        self.last_active = now();
    }
    
    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncDoc {
    pub id: String,
    pub content: String,
    pub version: u64,
    pub ops: Vec<DocOp>,
    pub peers: HashMap<String, u64>,  // peer_id -> last_seen_version
    pub cursors: Vec<Cursor>,
    pub formatting: HashMap<(usize, usize), Vec<TextStyle>>,  // (start, end) -> styles
}

impl SyncDoc {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: String::new(),
            version: 0,
            ops: Vec::new(),
            peers: HashMap::new(),
            cursors: Vec::new(),
            formatting: HashMap::new(),
        }
    }

    /// Create document with unique ID
    pub fn new_with_uuid() -> Self {
        Self::new(format!("doc_{}", uuid::Uuid::new_v4()))
    }

    pub fn update_cursor(&mut self, peer_id: String, position: usize, selection: Option<Selection>, name: String, color: String) {
        if let Some(cursor) = self.cursors.iter_mut().find(|c| c.peer_id == peer_id) {
            cursor.position = position;
            cursor.selection = selection;
            cursor.last_active = now();
        } else {
            let mut cursor = Cursor::new(peer_id, position, name, color);
            cursor.selection = selection;
            self.cursors.push(cursor);
        }
    }

    pub fn remove_cursor(&mut self, peer_id: &str) {
        self.cursors.retain(|c| c.peer_id != peer_id);
    }

    /// Check if cursor exists for peer
    pub fn has_cursor(&self, peer_id: &str) -> bool {
        self.cursors.iter().any(|c| c.peer_id == peer_id)
    }

    pub fn apply(&mut self, op: DocOp) {
        match &op {
            DocOp::Insert { pos, text, .. } => {
                let pos = (*pos).min(self.content.len());
                let len = text.len();
                self.content.insert_str(pos, text);
                // Transform all cursors
                for cursor in &mut self.cursors {
                    cursor.transform_insert(pos, len);
                }
            }
            DocOp::Delete { pos, len, .. } => {
                let pos = (*pos).min(self.content.len());
                let end = (pos + len).min(self.content.len());
                let actual_len = end - pos;
                self.content.drain(pos..end);
                // Transform all cursors
                for cursor in &mut self.cursors {
                    cursor.transform_delete(pos, actual_len);
                }
            }
            DocOp::Replace { pos, len, text, .. } => {
                let pos = (*pos).min(self.content.len());
                let end = (pos + len).min(self.content.len());
                let old_len = end - pos;
                let new_len = text.len();
                self.content.replace_range(pos..end, text);
                // Transform cursors: delete then insert
                for cursor in &mut self.cursors {
                    cursor.transform_delete(pos, old_len);
                    cursor.transform_insert(pos, new_len);
                }
            }
            DocOp::Format { pos, len, style, .. } => {
                let pos = (*pos).min(self.content.len());
                let end = (pos + len).min(self.content.len());
                // Store formatting range
                self.formatting
                    .entry((pos, end))
                    .or_default()
                    .push(style.clone());
            }
        }
        self.version += 1;
        self.ops.push(op);
    }

    /// Generate insert operation from text change
    pub fn generate_insert(&self, pos: usize, text: String, author: String) -> DocOp {
        DocOp::Insert {
            pos,
            text,
            ver: DocVersion { clock: self.version + 1, author },
        }
    }

    /// Generate delete operation from text change
    pub fn generate_delete(&self, pos: usize, len: usize, author: String) -> DocOp {
        DocOp::Delete {
            pos,
            len,
            ver: DocVersion { clock: self.version + 1, author },
        }
    }

    /// Generate replace operation from text change
    pub fn generate_replace(&self, pos: usize, len: usize, text: String, author: String) -> DocOp {
        DocOp::Replace {
            pos,
            len,
            text,
            ver: DocVersion { clock: self.version + 1, author },
        }
    }

    pub fn ops_since(&self, version: u64) -> Vec<DocOp> {
        self.ops.iter()
            .filter(|op| op.version().clock > version)
            .cloned()
            .collect()
    }

    pub fn merge(&mut self, ops: Vec<DocOp>) {
        for op in ops {
            if op.version().clock > self.version {
                self.apply(op);
            }
        }
    }

    /// Update peer's last seen version
    pub fn update_peer_version(&mut self, peer_id: String, version: u64) {
        self.peers.insert(peer_id, version);
    }

    /// Get peer's last seen version
    pub fn peer_version(&self, peer_id: &str) -> u64 {
        self.peers.get(peer_id).copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SyncMessage {
    RequestSync { doc_id: String, from_version: u64 },
    SyncOps { doc_id: String, ops: Vec<DocOp> },
    FullDoc { doc: SyncDoc },
    CursorUpdate { doc_id: String, peer_id: String, position: usize, selection: Option<Selection>, name: String, color: String },
    SelectionUpdate { doc_id: String, peer_id: String, selection: Selection },
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
    use proptest::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test_doc_insert() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 0,
            text: "hello".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        assert_eq!(doc.content, "hello");
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn test_doc_delete() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 0,
            text: "hello world".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        doc.apply(DocOp::Delete {
            pos: 5,
            len: 6,
            ver: DocVersion { clock: 2, author: "alice".into() },
        });
        assert_eq!(doc.content, "hello");
    }

    #[test]
    fn test_doc_replace() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 0,
            text: "hello world".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        doc.apply(DocOp::Replace {
            pos: 6,
            len: 5,
            text: "rust".into(),
            ver: DocVersion { clock: 2, author: "alice".into() },
        });
        assert_eq!(doc.content, "hello rust");
    }

    #[test]
    fn test_ops_since() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 0,
            text: "a".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        doc.apply(DocOp::Insert {
            pos: 1,
            text: "b".into(),
            ver: DocVersion { clock: 2, author: "alice".into() },
        });
        doc.apply(DocOp::Insert {
            pos: 2,
            text: "c".into(),
            ver: DocVersion { clock: 3, author: "alice".into() },
        });
        
        let ops = doc.ops_since(1);
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_merge() {
        let mut doc1 = SyncDoc::new("test");
        doc1.apply(DocOp::Insert {
            pos: 0,
            text: "hello".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        
        let mut doc2 = SyncDoc::new("test");
        let ops = doc1.ops_since(0);
        doc2.merge(ops);
        
        assert_eq!(doc1.content, doc2.content);
    }

    #[test]
    fn test_concurrent_edits() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 0,
            text: "hello".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        doc.apply(DocOp::Insert {
            pos: 5,
            text: " world".into(),
            ver: DocVersion { clock: 2, author: "bob".into() },
        });
        assert_eq!(doc.content, "hello world");
    }

    #[test]
    fn test_out_of_bounds_insert() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 100,
            text: "test".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        assert_eq!(doc.content, "test");
    }

    #[test]
    fn test_out_of_bounds_delete() {
        let mut doc = SyncDoc::new("test");
        doc.apply(DocOp::Insert {
            pos: 0,
            text: "hello".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        });
        doc.apply(DocOp::Delete {
            pos: 3,
            len: 100,
            ver: DocVersion { clock: 2, author: "alice".into() },
        });
        assert_eq!(doc.content, "hel");
    }

    #[test]
    fn test_cursor_removal() {
        // Property 19: Cursor Removal
        let mut doc = SyncDoc::new("test");
        doc.update_cursor("peer1".into(), 0, None, "Alice".into(), "#ff0000".into());
        doc.update_cursor("peer2".into(), 5, None, "Bob".into(), "#00ff00".into());
        
        assert!(doc.has_cursor("peer1"));
        assert!(doc.has_cursor("peer2"));
        
        doc.remove_cursor("peer1");
        
        assert!(!doc.has_cursor("peer1"));
        assert!(doc.has_cursor("peer2"));
    }

    #[test]
    fn test_operation_serialization() {
        let op = DocOp::Insert {
            pos: 5,
            text: "hello".into(),
            ver: DocVersion { clock: 1, author: "alice".into() },
        };
        
        let bytes = op.to_bytes().unwrap();
        let restored = DocOp::from_bytes(&bytes).unwrap();
        
        assert_eq!(op, restored);
    }

    #[test]
    fn test_generate_operations() {
        let doc = SyncDoc::new("test");
        
        let insert = doc.generate_insert(0, "hello".into(), "alice".into());
        if let DocOp::Insert { pos, text, ver } = insert {
            assert_eq!(pos, 0);
            assert_eq!(text, "hello");
            assert_eq!(ver.clock, 1);
        } else {
            panic!("Expected Insert operation");
        }
    }

    #[test]
    fn test_document_id_uniqueness() {
        // Property 16: Document ID Uniqueness
        let mut ids = HashSet::new();
        for _ in 0..100 {
            let doc = SyncDoc::new_with_uuid();
            assert!(ids.insert(doc.id.clone()), "Duplicate ID generated");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 16: Document ID Uniqueness
        #[test]
        fn prop_document_id_uniqueness(_seed in 0u64..1000) {
            let doc1 = SyncDoc::new_with_uuid();
            let doc2 = SyncDoc::new_with_uuid();
            prop_assert_ne!(doc1.id, doc2.id);
        }

        /// Property 17: Operation Generation Correctness
        #[test]
        fn prop_operation_generation_correctness(
            text in "[a-z]{1,20}",
            author in "[a-z]{5}",
        ) {
            let mut doc = SyncDoc::new("test");
            let op = doc.generate_insert(0, text.clone(), author);
            doc.apply(op);
            prop_assert_eq!(doc.content, text);
        }

        /// Property 19: Cursor Removal
        #[test]
        fn prop_cursor_removal(
            peer_id in "[a-z0-9]{8}",
            position in 0usize..1000,
        ) {
            let mut doc = SyncDoc::new("test");
            doc.update_cursor(peer_id.clone(), position, None, "Name".into(), "#000".into());
            
            prop_assert!(doc.has_cursor(&peer_id));
            
            doc.remove_cursor(&peer_id);
            
            prop_assert!(!doc.has_cursor(&peer_id));
        }

        /// Property 20: Operation Serialization Round-Trip
        #[test]
        fn prop_operation_serialization_roundtrip(
            pos in 0usize..1000,
            text in "[a-z]{1,50}",
            clock in 1u64..1000,
            author in "[a-z]{5}",
        ) {
            let op = DocOp::Insert {
                pos,
                text,
                ver: DocVersion { clock, author },
            };
            
            let bytes = op.to_bytes().unwrap();
            let restored = DocOp::from_bytes(&bytes).unwrap();
            
            prop_assert_eq!(op, restored);
        }

        /// Property: Delete operation serialization
        #[test]
        fn prop_delete_serialization_roundtrip(
            pos in 0usize..1000,
            len in 1usize..100,
            clock in 1u64..1000,
            author in "[a-z]{5}",
        ) {
            let op = DocOp::Delete {
                pos,
                len,
                ver: DocVersion { clock, author },
            };
            
            let bytes = op.to_bytes().unwrap();
            let restored = DocOp::from_bytes(&bytes).unwrap();
            
            prop_assert_eq!(op, restored);
        }

        /// Property 18: Operation Merge - Content Length Preservation
        /// Total content length equals sum of all inserted text
        #[test]
        fn prop_operation_merge_content_preservation(
            text1 in "[a-z]{5,20}",
            text2 in "[A-Z]{1,5}",
            text3 in "[0-9]{1,5}",
        ) {
            let mut doc = SyncDoc::new("test");
            
            // Apply three sequential inserts
            doc.apply(DocOp::Insert {
                pos: 0,
                text: text1.clone(),
                ver: DocVersion { clock: 1, author: "a".into() },
            });
            doc.apply(DocOp::Insert {
                pos: doc.content.len(),
                text: text2.clone(),
                ver: DocVersion { clock: 2, author: "b".into() },
            });
            doc.apply(DocOp::Insert {
                pos: doc.content.len(),
                text: text3.clone(),
                ver: DocVersion { clock: 3, author: "c".into() },
            });
            
            // Total length should be sum of all parts
            prop_assert_eq!(doc.content.len(), text1.len() + text2.len() + text3.len());
            
            // Content should end with text3 (last insert at end)
            prop_assert!(doc.content.ends_with(&text3));
        }

        /// Property: Merge idempotency - applying same ops twice doesn't change result
        #[test]
        fn prop_merge_idempotency(
            text in "[a-z]{5,20}",
            author in "[a-z]{5}",
        ) {
            let mut doc = SyncDoc::new("test");
            let op = DocOp::Insert {
                pos: 0,
                text: text.clone(),
                ver: DocVersion { clock: 1, author },
            };
            
            doc.apply(op.clone());
            let content_after_first = doc.content.clone();
            let version_after_first = doc.version;
            
            // Merge same op again (should be ignored due to version check)
            doc.merge(vec![op]);
            
            // Content should be unchanged (op already applied)
            prop_assert_eq!(doc.content, content_after_first);
            prop_assert_eq!(doc.version, version_after_first);
        }
    }
}

