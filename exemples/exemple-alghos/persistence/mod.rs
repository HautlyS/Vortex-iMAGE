//! Persistence module - SQLite for drive metadata, contacts, transfers, settings
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Drive file metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub hash: Option<[u8; 32]>,
    pub modified: u64,
    pub is_dir: bool,
    pub parent_id: Option<String>,
    pub ticket: Option<String>,
    pub sync_status: SyncState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SyncState {
    Synced,
    Pending,
    Uploading,
    Downloading,
    Conflict,
    Error(String),
}

impl Default for SyncState {
    fn default() -> Self { Self::Pending }
}

/// HIGH FIX: Explicit enum-to-string conversion for safe SQL storage
/// Prevents potential injection from Debug formatting of Error variant
fn sync_state_to_string(state: &SyncState) -> String {
    match state {
        SyncState::Synced => "Synced".to_string(),
        SyncState::Pending => "Pending".to_string(),
        SyncState::Uploading => "Uploading".to_string(),
        SyncState::Downloading => "Downloading".to_string(),
        SyncState::Conflict => "Conflict".to_string(),
        SyncState::Error(msg) => {
            // Sanitize error message to prevent SQL injection
            let sanitized = msg.chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .take(100)  // Limit length
                .collect::<String>();
            format!("Error:{}", sanitized)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FolderSync {
    pub id: String,
    pub local_path: String,
    pub remote_id: String,
    pub last_sync: u64,
    pub file_count: u32,
    pub total_size: u64,
}

/// Contact stored in database
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredContact {
    pub id: String,
    pub name: String,
    pub peer_id: String,
    pub public_key: Vec<u8>,
    pub trust_level: String,
    pub blocked: bool,
    pub last_seen: u64,
    pub created_at: u64,
}

/// Transfer record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredTransfer {
    pub id: String,
    pub name: String,
    pub total_bytes: u64,
    pub bytes_transferred: u64,
    pub status: String,
    pub direction: String,
    pub ticket: Option<String>,
    pub hash: Option<String>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

/// Message stored in database
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub content_type: String,
    pub timestamp: u64,
    pub reply_to: Option<String>,
}

/// Conversation stored in database
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredConversation {
    pub id: String,
    pub conv_type: String,
    pub name: String,
    pub created_at: u64,
    pub last_message_at: u64,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub invite_code: Option<String>,
}

pub const SCHEMA: &str = r#"
-- Files table
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    size INTEGER NOT NULL,
    hash TEXT,
    modified INTEGER NOT NULL,
    is_dir INTEGER NOT NULL,
    parent_id TEXT,
    ticket TEXT,
    sync_status TEXT NOT NULL DEFAULT 'Pending'
);
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent_id);

-- Folder syncs
CREATE TABLE IF NOT EXISTS folder_syncs (
    id TEXT PRIMARY KEY,
    local_path TEXT NOT NULL UNIQUE,
    remote_id TEXT NOT NULL,
    last_sync INTEGER NOT NULL,
    file_count INTEGER NOT NULL DEFAULT 0,
    total_size INTEGER NOT NULL DEFAULT 0
);

-- Contacts table
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    peer_id TEXT NOT NULL UNIQUE,
    public_key BLOB NOT NULL,
    trust_level TEXT NOT NULL DEFAULT 'unknown',
    blocked INTEGER NOT NULL DEFAULT 0,
    last_seen INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_contacts_peer ON contacts(peer_id);

-- Transfers table
CREATE TABLE IF NOT EXISTS transfers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    total_bytes INTEGER NOT NULL,
    bytes_transferred INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Pending',
    direction TEXT NOT NULL,
    ticket TEXT,
    hash TEXT,
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_transfers_status ON transfers(status);

-- Conversations table
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    conv_type TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_message_at INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_muted INTEGER NOT NULL DEFAULT 0,
    invite_code TEXT
);

-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text',
    timestamp INTEGER NOT NULL,
    reply_to TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_time ON messages(timestamp);

-- Settings table (key-value)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

/// SQLite database for drive persistence
pub struct DriveDb {
    conn: Connection,
}

impl DriveDb {
    pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    // ============ FILE METHODS ============
    
    pub fn insert_file(&self, file: &FileMetadata) -> rusqlite::Result<()> {
        let hash_str = file.hash.map(hex::encode);
        // HIGH FIX: Use explicit enum-to-string conversion instead of Debug formatting
        // This prevents potential injection if sync_status contained user input
        let sync_status_str = sync_state_to_string(&file.sync_status);
        self.conn.execute(
            "INSERT OR REPLACE INTO files (id, path, name, size, hash, modified, is_dir, parent_id, ticket, sync_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                file.id, file.path, file.name, file.size as i64,
                hash_str, file.modified as i64, file.is_dir,
                file.parent_id, file.ticket, sync_status_str
            ],
        )?;
        Ok(())
    }

    pub fn get_file(&self, id: &str) -> rusqlite::Result<Option<FileMetadata>> {
        let mut stmt = self.conn.prepare("SELECT * FROM files WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_file(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_path(&self, path: &str) -> rusqlite::Result<Option<FileMetadata>> {
        let mut stmt = self.conn.prepare("SELECT * FROM files WHERE path = ?1")?;
        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_file(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_dir(&self, parent_id: Option<&str>) -> rusqlite::Result<Vec<FileMetadata>> {
        let mut stmt = if parent_id.is_some() {
            self.conn.prepare("SELECT * FROM files WHERE parent_id = ?1 ORDER BY is_dir DESC, name")?
        } else {
            self.conn.prepare("SELECT * FROM files WHERE parent_id IS NULL ORDER BY is_dir DESC, name")?
        };
        let mut rows = stmt.query(params![parent_id])?;
        let mut files = Vec::new();
        while let Some(row) = rows.next()? {
            files.push(Self::row_to_file(row)?);
        }
        Ok(files)
    }

    pub fn pending_sync(&self) -> rusqlite::Result<Vec<FileMetadata>> {
        let mut stmt = self.conn.prepare("SELECT * FROM files WHERE sync_status = 'Pending' ORDER BY modified DESC LIMIT 100")?;
        let mut rows = stmt.query([])?;
        let mut files = Vec::new();
        while let Some(row) = rows.next()? {
            files.push(Self::row_to_file(row)?);
        }
        Ok(files)
    }

    pub fn update_sync_status(&self, id: &str, status: SyncState) -> rusqlite::Result<()> {
        // HIGH FIX: Use explicit enum-to-string conversion
        let status_str = sync_state_to_string(&status);
        self.conn.execute(
            "UPDATE files SET sync_status = ?1 WHERE id = ?2",
            params![status_str, id],
        )?;
        Ok(())
    }

    pub fn delete_file(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM files WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn stats(&self) -> rusqlite::Result<(u64, u64)> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*), COALESCE(SUM(size), 0) FROM files")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let count: i64 = row.get(0)?;
            let size: i64 = row.get(1)?;
            Ok((count as u64, size as u64))
        } else {
            Ok((0, 0))
        }
    }

    // ============ CONTACT METHODS ============
    
    pub fn insert_contact(&self, contact: &StoredContact) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO contacts (id, name, peer_id, public_key, trust_level, blocked, last_seen, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![contact.id, contact.name, contact.peer_id, contact.public_key, contact.trust_level, contact.blocked, contact.last_seen as i64, contact.created_at as i64],
        )?;
        Ok(())
    }

    pub fn get_contact(&self, id: &str) -> rusqlite::Result<Option<StoredContact>> {
        let mut stmt = self.conn.prepare("SELECT * FROM contacts WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(StoredContact {
                id: row.get(0)?,
                name: row.get(1)?,
                peer_id: row.get(2)?,
                public_key: row.get(3)?,
                trust_level: row.get(4)?,
                blocked: row.get(5)?,
                last_seen: row.get::<_, i64>(6)? as u64,
                created_at: row.get::<_, i64>(7)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_contacts(&self) -> rusqlite::Result<Vec<StoredContact>> {
        let mut stmt = self.conn.prepare("SELECT * FROM contacts WHERE blocked = 0 ORDER BY name")?;
        let mut rows = stmt.query([])?;
        let mut contacts = Vec::new();
        while let Some(row) = rows.next()? {
            contacts.push(StoredContact {
                id: row.get(0)?,
                name: row.get(1)?,
                peer_id: row.get(2)?,
                public_key: row.get(3)?,
                trust_level: row.get(4)?,
                blocked: row.get(5)?,
                last_seen: row.get::<_, i64>(6)? as u64,
                created_at: row.get::<_, i64>(7)? as u64,
            });
        }
        Ok(contacts)
    }

    pub fn update_contact_trust(&self, id: &str, trust_level: &str) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE contacts SET trust_level = ?1 WHERE id = ?2", params![trust_level, id])?;
        Ok(())
    }

    pub fn block_contact(&self, id: &str, blocked: bool) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE contacts SET blocked = ?1 WHERE id = ?2", params![blocked, id])?;
        Ok(())
    }

    pub fn delete_contact(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM contacts WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ============ TRANSFER METHODS ============
    
    pub fn insert_transfer(&self, transfer: &StoredTransfer) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO transfers (id, name, total_bytes, bytes_transferred, status, direction, ticket, hash, created_at, completed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![transfer.id, transfer.name, transfer.total_bytes as i64, transfer.bytes_transferred as i64, transfer.status, transfer.direction, transfer.ticket, transfer.hash, transfer.created_at as i64, transfer.completed_at.map(|t| t as i64)],
        )?;
        Ok(())
    }

    pub fn get_transfer(&self, id: &str) -> rusqlite::Result<Option<StoredTransfer>> {
        let mut stmt = self.conn.prepare("SELECT * FROM transfers WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_transfer(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_transfers(&self, limit: u32) -> rusqlite::Result<Vec<StoredTransfer>> {
        let mut stmt = self.conn.prepare("SELECT * FROM transfers ORDER BY created_at DESC LIMIT ?1")?;
        let mut rows = stmt.query(params![limit])?;
        let mut transfers = Vec::new();
        while let Some(row) = rows.next()? {
            transfers.push(Self::row_to_transfer(row)?);
        }
        Ok(transfers)
    }

    pub fn update_transfer_progress(&self, id: &str, bytes: u64, status: &str) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE transfers SET bytes_transferred = ?1, status = ?2 WHERE id = ?3", params![bytes as i64, status, id])?;
        Ok(())
    }

    // ============ MESSAGE METHODS ============
    
    pub fn insert_message(&self, msg: &StoredMessage) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO messages (id, conversation_id, sender_id, content, content_type, timestamp, reply_to) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![msg.id, msg.conversation_id, msg.sender_id, msg.content, msg.content_type, msg.timestamp as i64, msg.reply_to],
        )?;
        // Update conversation last_message_at
        self.conn.execute("UPDATE conversations SET last_message_at = ?1 WHERE id = ?2", params![msg.timestamp as i64, msg.conversation_id])?;
        Ok(())
    }

    pub fn get_messages(&self, conversation_id: &str, limit: u32, before: Option<u64>) -> rusqlite::Result<Vec<StoredMessage>> {
        let mut stmt = if before.is_some() {
            self.conn.prepare("SELECT * FROM messages WHERE conversation_id = ?1 AND timestamp < ?2 ORDER BY timestamp DESC LIMIT ?3")?
        } else {
            self.conn.prepare("SELECT * FROM messages WHERE conversation_id = ?1 ORDER BY timestamp DESC LIMIT ?2")?
        };
        let mut rows = if let Some(ts) = before {
            stmt.query(params![conversation_id, ts as i64, limit])?
        } else {
            stmt.query(params![conversation_id, limit])?
        };
        let mut messages = Vec::new();
        while let Some(row) = rows.next()? {
            messages.push(StoredMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                sender_id: row.get(2)?,
                content: row.get(3)?,
                content_type: row.get(4)?,
                timestamp: row.get::<_, i64>(5)? as u64,
                reply_to: row.get(6)?,
            });
        }
        messages.reverse(); // Return in chronological order
        Ok(messages)
    }

    // ============ CONVERSATION METHODS ============
    
    pub fn insert_conversation(&self, conv: &StoredConversation) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO conversations (id, conv_type, name, created_at, last_message_at, is_pinned, is_muted, invite_code) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![conv.id, conv.conv_type, conv.name, conv.created_at as i64, conv.last_message_at as i64, conv.is_pinned, conv.is_muted, conv.invite_code],
        )?;
        Ok(())
    }

    pub fn list_conversations(&self) -> rusqlite::Result<Vec<StoredConversation>> {
        let mut stmt = self.conn.prepare("SELECT * FROM conversations ORDER BY is_pinned DESC, last_message_at DESC")?;
        let mut rows = stmt.query([])?;
        let mut convs = Vec::new();
        while let Some(row) = rows.next()? {
            convs.push(StoredConversation {
                id: row.get(0)?,
                conv_type: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
                last_message_at: row.get::<_, i64>(4)? as u64,
                is_pinned: row.get(5)?,
                is_muted: row.get(6)?,
                invite_code: row.get(7)?,
            });
        }
        Ok(convs)
    }

    pub fn pin_conversation(&self, id: &str, pinned: bool) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE conversations SET is_pinned = ?1 WHERE id = ?2", params![pinned, id])?;
        Ok(())
    }

    pub fn mute_conversation(&self, id: &str, muted: bool) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE conversations SET is_muted = ?1 WHERE id = ?2", params![muted, id])?;
        Ok(())
    }

    pub fn delete_conversation(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM messages WHERE conversation_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ============ SETTINGS METHODS ============
    
    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_setting(&self, key: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    // ============ HELPER METHODS ============

    fn row_to_file(row: &rusqlite::Row) -> rusqlite::Result<FileMetadata> {
        let hash_str: Option<String> = row.get(4)?;
        let hash = hash_str.and_then(|s| {
            let bytes = hex::decode(&s).ok()?;
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            } else { None }
        });
        let status_str: String = row.get(9)?;
        let sync_status = match status_str.as_str() {
            "Synced" => SyncState::Synced,
            "Uploading" => SyncState::Uploading,
            "Downloading" => SyncState::Downloading,
            "Conflict" => SyncState::Conflict,
            s if s.starts_with("Error") => SyncState::Error(s.to_string()),
            _ => SyncState::Pending,
        };
        Ok(FileMetadata {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            size: row.get::<_, i64>(3)? as u64,
            hash,
            modified: row.get::<_, i64>(5)? as u64,
            is_dir: row.get(6)?,
            parent_id: row.get(7)?,
            ticket: row.get(8)?,
            sync_status,
        })
    }

    fn row_to_transfer(row: &rusqlite::Row) -> rusqlite::Result<StoredTransfer> {
        Ok(StoredTransfer {
            id: row.get(0)?,
            name: row.get(1)?,
            total_bytes: row.get::<_, i64>(2)? as u64,
            bytes_transferred: row.get::<_, i64>(3)? as u64,
            status: row.get(4)?,
            direction: row.get(5)?,
            ticket: row.get(6)?,
            hash: row.get(7)?,
            created_at: row.get::<_, i64>(8)? as u64,
            completed_at: row.get::<_, Option<i64>>(9)?.map(|t| t as u64),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_crud() {
        let db = DriveDb::memory().unwrap();
        let file = FileMetadata {
            id: "f_1".into(),
            path: "/test.txt".into(),
            name: "test.txt".into(),
            size: 1024,
            hash: None,
            modified: 12345,
            is_dir: false,
            parent_id: None,
            ticket: None,
            sync_status: SyncState::Pending,
        };
        db.insert_file(&file).unwrap();
        let loaded = db.get_file("f_1").unwrap().unwrap();
        assert_eq!(loaded.name, "test.txt");
        assert_eq!(loaded.size, 1024);
    }

    #[test]
    fn test_list_dir() {
        let db = DriveDb::memory().unwrap();
        db.insert_file(&FileMetadata {
            id: "d_1".into(), path: "/docs".into(), name: "docs".into(),
            size: 0, hash: None, modified: 0, is_dir: true,
            parent_id: None, ticket: None, sync_status: SyncState::Synced,
        }).unwrap();
        db.insert_file(&FileMetadata {
            id: "f_1".into(), path: "/docs/a.txt".into(), name: "a.txt".into(),
            size: 100, hash: None, modified: 0, is_dir: false,
            parent_id: Some("d_1".into()), ticket: None, sync_status: SyncState::Synced,
        }).unwrap();
        let files = db.list_dir(Some("d_1")).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "a.txt");
    }
}
