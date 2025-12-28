//! P2P Drive - Encrypted folder sync with permissions
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File entry in drive with hash for integrity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DriveEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: u64,
    pub ticket: Option<String>,
    pub hash: Option<[u8; 32]>,  // BLAKE3 hash for integrity
    pub children: Vec<DriveEntry>,
}

impl DriveEntry {
    pub fn file(name: String, path: String, size: u64, ticket: Option<String>) -> Self {
        Self {
            id: format!("f_{}", uuid::Uuid::new_v4()),
            name,
            path,
            size,
            is_dir: false,
            modified: now(),
            ticket,
            hash: None,
            children: Vec::new(),
        }
    }

    pub fn file_with_hash(name: String, path: String, size: u64, hash: [u8; 32]) -> Self {
        Self {
            id: format!("f_{}", uuid::Uuid::new_v4()),
            name,
            path,
            size,
            is_dir: false,
            modified: now(),
            ticket: None,
            hash: Some(hash),
            children: Vec::new(),
        }
    }

    pub fn folder(name: String, path: String) -> Self {
        Self {
            id: format!("d_{}", uuid::Uuid::new_v4()),
            name,
            path,
            size: 0,
            is_dir: true,
            modified: now(),
            ticket: None,
            hash: None,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, entry: DriveEntry) {
        if self.is_dir {
            self.size += entry.size;
            self.children.push(entry);
        }
    }

    pub fn total_files(&self) -> usize {
        if !self.is_dir { return 1; }
        self.children.iter().map(|c| c.total_files()).sum()
    }

    /// Set hash for file
    pub fn set_hash(&mut self, hash: [u8; 32]) {
        self.hash = Some(hash);
    }

    /// Find entry by path
    pub fn find_by_path(&self, path: &str) -> Option<&DriveEntry> {
        if self.path == path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_path(path) {
                return Some(found);
            }
        }
        None
    }
}

/// Permission levels for shared folders
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    Read,   // Can view and download
    Write,  // Can add and modify files
    Admin,  // Can manage permissions
}

impl Default for Permission {
    fn default() -> Self {
        Self::Read
    }
}

/// Conflict information when sync conflicts occur
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ConflictInfo {
    pub path: String,
    pub local_modified: u64,
    pub remote_modified: u64,
    pub local_hash: [u8; 32],
    pub remote_hash: [u8; 32],
}

impl ConflictInfo {
    pub fn new(path: String, local_modified: u64, remote_modified: u64, local_hash: [u8; 32], remote_hash: [u8; 32]) -> Self {
        Self { path, local_modified, remote_modified, local_hash, remote_hash }
    }
}

/// Shared folder with permissions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedFolder {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub root: DriveEntry,
    pub shared_with: Vec<(String, Permission)>,  // (peer_id, permission)
    pub sync_status: SyncStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Synced,
    Syncing { progress: u8, current_file: String },
    Pending,
    Conflict { files: Vec<ConflictInfo> },
    Error(String),
}

impl SharedFolder {
    pub fn new(name: String, owner_id: String, root_path: String) -> Self {
        Self {
            id: format!("sf_{}", uuid::Uuid::new_v4()),
            name: name.clone(),
            owner_id,
            root: DriveEntry::folder(name, root_path),
            shared_with: Vec::new(),
            sync_status: SyncStatus::Pending,
        }
    }

    /// Share folder with a peer with specific permission
    pub fn share_with(&mut self, peer_id: String, permission: Permission) {
        // Remove existing permission for this peer
        self.shared_with.retain(|(id, _)| id != &peer_id);
        self.shared_with.push((peer_id, permission));
    }

    /// Get permission for a peer
    pub fn get_permission(&self, peer_id: &str) -> Option<&Permission> {
        self.shared_with.iter()
            .find(|(id, _)| id == peer_id)
            .map(|(_, perm)| perm)
    }

    /// Check if peer has at least the given permission level
    pub fn has_permission(&self, peer_id: &str, required: Permission) -> bool {
        if peer_id == self.owner_id {
            return true;  // Owner has all permissions
        }
        
        self.get_permission(peer_id).map(|perm| {
            match required {
                Permission::Read => true,  // All permissions include read
                Permission::Write => matches!(perm, Permission::Write | Permission::Admin),
                Permission::Admin => matches!(perm, Permission::Admin),
            }
        }).unwrap_or(false)
    }

    /// Remove sharing with a peer
    pub fn unshare(&mut self, peer_id: &str) {
        self.shared_with.retain(|(id, _)| id != peer_id);
    }

    /// Get list of peers with access
    pub fn shared_peers(&self) -> Vec<&str> {
        self.shared_with.iter().map(|(id, _)| id.as_str()).collect()
    }

    /// Detect conflict between local and remote versions
    pub fn detect_conflict(&self, path: &str, local_modified: u64, local_hash: [u8; 32], 
                          remote_modified: u64, remote_hash: [u8; 32]) -> Option<ConflictInfo> {
        if local_hash != remote_hash && local_modified != remote_modified {
            Some(ConflictInfo::new(path.to_string(), local_modified, remote_modified, local_hash, remote_hash))
        } else {
            None
        }
    }

    /// Add conflict to status
    pub fn add_conflict(&mut self, conflict: ConflictInfo) {
        match &mut self.sync_status {
            SyncStatus::Conflict { files } => {
                files.push(conflict);
            }
            _ => {
                self.sync_status = SyncStatus::Conflict { files: vec![conflict] };
            }
        }
    }

    /// Resolve conflict by keeping both versions
    pub fn resolve_conflict_keep_both(&mut self, path: &str) -> Option<(String, String)> {
        if let SyncStatus::Conflict { files } = &mut self.sync_status {
            if let Some(idx) = files.iter().position(|c| c.path == path) {
                let _conflict = files.remove(idx);
                let local_path = format!("{}.local", path);
                let remote_path = format!("{}.remote", path);
                
                if files.is_empty() {
                    self.sync_status = SyncStatus::Synced;
                }
                
                return Some((local_path, remote_path));
            }
        }
        None
    }
}

/// Drive sync event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DriveEvent {
    FileAdded { folder_id: String, entry: DriveEntry },
    FileRemoved { folder_id: String, path: String },
    FileModified { folder_id: String, entry: DriveEntry },
    SyncStarted { folder_id: String },
    SyncProgress { folder_id: String, progress: u8 },
    SyncComplete { folder_id: String },
    SyncError { folder_id: String, error: String },
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Scan directory recursively with hash computation
pub async fn scan_directory(path: &PathBuf) -> std::io::Result<DriveEntry> {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let meta = tokio::fs::metadata(path).await?;
    
    if meta.is_file() {
        // Compute BLAKE3 hash for file
        let data = tokio::fs::read(path).await?;
        let hash = *blake3::hash(&data).as_bytes();
        
        return Ok(DriveEntry::file_with_hash(
            name,
            path.to_string_lossy().to_string(),
            meta.len(),
            hash,
        ));
    }

    let mut entry = DriveEntry::folder(name, path.to_string_lossy().to_string());
    let mut dir = tokio::fs::read_dir(path).await?;
    
    while let Some(child) = dir.next_entry().await? {
        let child_entry = Box::pin(scan_directory(&child.path())).await?;
        entry.add_child(child_entry);
    }
    
    Ok(entry)
}

/// Scan directory without computing hashes (faster)
pub async fn scan_directory_fast(path: &PathBuf) -> std::io::Result<DriveEntry> {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let meta = tokio::fs::metadata(path).await?;
    
    if meta.is_file() {
        return Ok(DriveEntry::file(
            name,
            path.to_string_lossy().to_string(),
            meta.len(),
            None,
        ));
    }

    let mut entry = DriveEntry::folder(name, path.to_string_lossy().to_string());
    let mut dir = tokio::fs::read_dir(path).await?;
    
    while let Some(child) = dir.next_entry().await? {
        let child_entry = Box::pin(scan_directory_fast(&child.path())).await?;
        entry.add_child(child_entry);
    }
    
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_drive_entry_creation() {
        let file = DriveEntry::file("test.txt".into(), "/path/test.txt".into(), 1024, None);
        assert!(!file.is_dir);
        assert_eq!(file.size, 1024);
        
        let folder = DriveEntry::folder("docs".into(), "/path/docs".into());
        assert!(folder.is_dir);
        assert_eq!(folder.size, 0);
    }

    #[test]
    fn test_total_files() {
        let mut root = DriveEntry::folder("root".into(), "/root".into());
        root.add_child(DriveEntry::file("a.txt".into(), "/root/a.txt".into(), 100, None));
        root.add_child(DriveEntry::file("b.txt".into(), "/root/b.txt".into(), 200, None));
        
        let mut sub = DriveEntry::folder("sub".into(), "/root/sub".into());
        sub.add_child(DriveEntry::file("c.txt".into(), "/root/sub/c.txt".into(), 300, None));
        root.add_child(sub);
        
        assert_eq!(root.total_files(), 3);
    }

    #[test]
    fn test_permission_assignment() {
        let mut folder = SharedFolder::new("shared".into(), "owner".into(), "/shared".into());
        
        folder.share_with("peer1".into(), Permission::Read);
        folder.share_with("peer2".into(), Permission::Write);
        folder.share_with("peer3".into(), Permission::Admin);
        
        assert_eq!(folder.get_permission("peer1"), Some(&Permission::Read));
        assert_eq!(folder.get_permission("peer2"), Some(&Permission::Write));
        assert_eq!(folder.get_permission("peer3"), Some(&Permission::Admin));
    }

    #[test]
    fn test_permission_check() {
        let mut folder = SharedFolder::new("shared".into(), "owner".into(), "/shared".into());
        folder.share_with("reader".into(), Permission::Read);
        folder.share_with("writer".into(), Permission::Write);
        folder.share_with("admin".into(), Permission::Admin);
        
        // Owner has all permissions
        assert!(folder.has_permission("owner", Permission::Admin));
        
        // Reader can only read
        assert!(folder.has_permission("reader", Permission::Read));
        assert!(!folder.has_permission("reader", Permission::Write));
        
        // Writer can read and write
        assert!(folder.has_permission("writer", Permission::Read));
        assert!(folder.has_permission("writer", Permission::Write));
        assert!(!folder.has_permission("writer", Permission::Admin));
        
        // Admin can do everything
        assert!(folder.has_permission("admin", Permission::Admin));
    }

    #[test]
    fn test_conflict_detection() {
        let folder = SharedFolder::new("shared".into(), "owner".into(), "/shared".into());
        
        let local_hash = [1u8; 32];
        let remote_hash = [2u8; 32];
        
        // Different hashes and times = conflict
        let conflict = folder.detect_conflict("/file.txt", 1000, local_hash, 2000, remote_hash);
        assert!(conflict.is_some());
        
        // Same hash = no conflict
        let no_conflict = folder.detect_conflict("/file.txt", 1000, local_hash, 2000, local_hash);
        assert!(no_conflict.is_none());
    }

    #[test]
    fn test_conflict_resolution() {
        let mut folder = SharedFolder::new("shared".into(), "owner".into(), "/shared".into());
        
        let conflict = ConflictInfo::new(
            "/file.txt".into(),
            1000, 2000,
            [1u8; 32], [2u8; 32],
        );
        folder.add_conflict(conflict);
        
        let result = folder.resolve_conflict_keep_both("/file.txt");
        assert!(result.is_some());
        
        let (local, remote) = result.unwrap();
        assert_eq!(local, "/file.txt.local");
        assert_eq!(remote, "/file.txt.remote");
        
        // Status should be synced after resolving all conflicts
        assert_eq!(folder.sync_status, SyncStatus::Synced);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 13: Drive Tree Scanning
        #[test]
        fn prop_drive_tree_total_files(
            num_files in 1usize..20,
        ) {
            let mut root = DriveEntry::folder("root".into(), "/root".into());
            
            for i in 0..num_files {
                root.add_child(DriveEntry::file(
                    format!("file{}.txt", i),
                    format!("/root/file{}.txt", i),
                    100,
                    None,
                ));
            }
            
            prop_assert_eq!(root.total_files(), num_files);
        }

        /// Property 14: Folder Permission Assignment
        #[test]
        fn prop_folder_permission_assignment(
            peer_id in "[a-z0-9]{8}",
            perm_idx in 0usize..3,
        ) {
            let mut folder = SharedFolder::new("shared".into(), "owner".into(), "/shared".into());
            let permission = match perm_idx {
                0 => Permission::Read,
                1 => Permission::Write,
                _ => Permission::Admin,
            };
            
            folder.share_with(peer_id.clone(), permission.clone());
            
            // Should contain exactly one entry for this peer
            let count = folder.shared_with.iter().filter(|(id, _)| id == &peer_id).count();
            prop_assert_eq!(count, 1);
            
            // Permission should match
            prop_assert_eq!(folder.get_permission(&peer_id), Some(&permission));
        }

        /// Property 15: Conflict Resolution Preservation
        #[test]
        fn prop_conflict_resolution_preservation(
            path in "[a-z/]{5,20}",
            local_mod in 1000u64..2000,
            remote_mod in 2000u64..3000,
        ) {
            let mut folder = SharedFolder::new("shared".into(), "owner".into(), "/shared".into());
            
            let conflict = ConflictInfo::new(
                path.clone(),
                local_mod, remote_mod,
                [1u8; 32], [2u8; 32],
            );
            folder.add_conflict(conflict);
            
            // Resolve should return both paths
            let result = folder.resolve_conflict_keep_both(&path);
            prop_assert!(result.is_some());
            
            let (local_path, remote_path) = result.unwrap();
            prop_assert!(local_path.contains(&path));
            prop_assert!(remote_path.contains(&path));
        }
    }
}
