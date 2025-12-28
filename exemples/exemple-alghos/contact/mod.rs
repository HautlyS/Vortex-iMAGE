//! Contact Management Module
//! Trust levels, presence, and blocking

use serde::{Deserialize, Serialize};
use crate::crypto::PublicBundle;
use crate::net::PeerId;

/// Trust level for contacts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustLevel {
    Unknown,   // New contact, not verified
    Known,     // Communicated before
    Trusted,   // Auto-accept files
    Verified,  // Key verified out-of-band
}

impl Default for TrustLevel {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Presence status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Presence {
    Online,
    Away,
    Busy,
    Offline,
}

impl Default for Presence {
    fn default() -> Self {
        Self::Offline
    }
}

/// A contact in the address book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: PeerId,
    pub name: String,
    pub public_bundle: PublicBundle,
    pub trust_level: TrustLevel,
    pub presence: Presence,
    pub last_seen: u64,
    pub blocked: bool,
    pub verified: bool,
    pub created_at: u64,
    pub notes: Option<String>,
}

impl Contact {
    /// Create a new contact
    pub fn new(id: PeerId, name: String, public_bundle: PublicBundle) -> Self {
        Self {
            id,
            name,
            public_bundle,
            trust_level: TrustLevel::Unknown,
            presence: Presence::Offline,
            last_seen: 0,
            blocked: false,
            verified: false,
            created_at: now(),
            notes: None,
        }
    }

    /// Set trust level
    pub fn set_trust_level(&mut self, level: TrustLevel) {
        self.trust_level = level;
    }

    /// Mark as trusted (auto-accept files)
    pub fn set_trusted(&mut self) {
        self.trust_level = TrustLevel::Trusted;
    }

    /// Mark as verified (out-of-band verification)
    pub fn set_verified(&mut self) {
        self.verified = true;
        self.trust_level = TrustLevel::Verified;
    }

    /// Block this contact
    pub fn block(&mut self) {
        self.blocked = true;
    }

    /// Unblock this contact
    pub fn unblock(&mut self) {
        self.blocked = false;
    }

    /// Update presence
    pub fn update_presence(&mut self, presence: Presence) {
        let is_online = presence != Presence::Offline;
        self.presence = presence;
        if is_online {
            self.last_seen = now();
        }
    }

    /// Check if public bundle has changed (potential key change attack)
    pub fn verify_bundle_unchanged(&self, new_bundle: &PublicBundle) -> bool {
        self.public_bundle.pq == new_bundle.pq && self.public_bundle.x25519 == new_bundle.x25519
    }

    /// Update public bundle (with warning)
    pub fn update_bundle(&mut self, new_bundle: PublicBundle) {
        self.public_bundle = new_bundle;
        // Reset verification status on key change
        self.verified = false;
        if self.trust_level == TrustLevel::Verified {
            self.trust_level = TrustLevel::Known;
        }
    }
}

/// Contact storage manager
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ContactBook {
    contacts: Vec<Contact>,
}

impl ContactBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a contact
    pub fn add(&mut self, contact: Contact) {
        // Don't add duplicates
        if !self.contacts.iter().any(|c| c.id == contact.id) {
            self.contacts.push(contact);
        }
    }

    /// Get contact by ID
    pub fn get(&self, id: &PeerId) -> Option<&Contact> {
        self.contacts.iter().find(|c| &c.id == id)
    }

    /// Get mutable contact by ID
    pub fn get_mut(&mut self, id: &PeerId) -> Option<&mut Contact> {
        self.contacts.iter_mut().find(|c| &c.id == id)
    }

    /// Remove a contact
    pub fn remove(&mut self, id: &PeerId) -> Option<Contact> {
        if let Some(idx) = self.contacts.iter().position(|c| &c.id == id) {
            Some(self.contacts.remove(idx))
        } else {
            None
        }
    }

    /// Get all contacts
    pub fn all(&self) -> &[Contact] {
        &self.contacts
    }

    /// Get online contacts
    pub fn online(&self) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|c| c.presence == Presence::Online && !c.blocked)
            .collect()
    }

    /// Get trusted contacts
    pub fn trusted(&self) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|c| matches!(c.trust_level, TrustLevel::Trusted | TrustLevel::Verified))
            .collect()
    }

    /// Get blocked contacts
    pub fn blocked(&self) -> Vec<&Contact> {
        self.contacts.iter().filter(|c| c.blocked).collect()
    }

    /// Check if a peer is blocked
    pub fn is_blocked(&self, id: &PeerId) -> bool {
        self.get(id).map(|c| c.blocked).unwrap_or(false)
    }

    /// Filter messages from blocked contacts
    pub fn filter_blocked<T, F>(&self, items: Vec<T>, get_sender: F) -> Vec<T>
    where
        F: Fn(&T) -> &PeerId,
    {
        items
            .into_iter()
            .filter(|item| !self.is_blocked(get_sender(item)))
            .collect()
    }

    /// Search contacts by name
    pub fn search(&self, query: &str) -> Vec<&Contact> {
        let query_lower = query.to_lowercase();
        self.contacts
            .iter()
            .filter(|c| c.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Update presence for a contact
    pub fn update_presence(&mut self, id: &PeerId, presence: Presence) {
        if let Some(contact) = self.get_mut(id) {
            contact.update_presence(presence);
        }
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
    use proptest::prelude::*;

    fn mock_bundle() -> PublicBundle {
        PublicBundle {
            pq: vec![0u8; 1568],
            x25519: [0u8; 32],
            verifying_key: [0u8; 32],
        }
    }

    #[test]
    fn test_contact_creation() {
        let contact = Contact::new("peer1".into(), "Alice".into(), mock_bundle());
        assert_eq!(contact.trust_level, TrustLevel::Unknown);
        assert_eq!(contact.presence, Presence::Offline);
        assert!(!contact.blocked);
        assert!(!contact.verified);
    }

    #[test]
    fn test_trust_level_setting() {
        let mut contact = Contact::new("peer1".into(), "Alice".into(), mock_bundle());
        
        contact.set_trusted();
        assert_eq!(contact.trust_level, TrustLevel::Trusted);
        
        contact.set_verified();
        assert_eq!(contact.trust_level, TrustLevel::Verified);
        assert!(contact.verified);
    }

    #[test]
    fn test_blocking() {
        let mut contact = Contact::new("peer1".into(), "Alice".into(), mock_bundle());
        
        assert!(!contact.blocked);
        contact.block();
        assert!(contact.blocked);
        contact.unblock();
        assert!(!contact.blocked);
    }

    #[test]
    fn test_bundle_change_detection() {
        let bundle1 = mock_bundle();
        let mut bundle2 = mock_bundle();
        bundle2.x25519[0] = 1;  // Change one byte
        
        let contact = Contact::new("peer1".into(), "Alice".into(), bundle1.clone());
        
        assert!(contact.verify_bundle_unchanged(&bundle1));
        assert!(!contact.verify_bundle_unchanged(&bundle2));
    }

    #[test]
    fn test_contact_book_operations() {
        let mut book = ContactBook::new();
        
        let contact1 = Contact::new("peer1".into(), "Alice".into(), mock_bundle());
        let contact2 = Contact::new("peer2".into(), "Bob".into(), mock_bundle());
        
        book.add(contact1);
        book.add(contact2);
        
        assert_eq!(book.all().len(), 2);
        assert!(book.get(&"peer1".into()).is_some());
        
        book.remove(&"peer1".into());
        assert_eq!(book.all().len(), 1);
        assert!(book.get(&"peer1".into()).is_none());
    }

    #[test]
    fn test_block_filtering() {
        use crate::net::PeerId;
        
        let mut book = ContactBook::new();
        
        let mut contact1 = Contact::new("peer1".into(), "Alice".into(), mock_bundle());
        contact1.block();
        let contact2 = Contact::new("peer2".into(), "Bob".into(), mock_bundle());
        
        book.add(contact1);
        book.add(contact2);
        
        // Simulate messages with PeerId
        let messages = vec![
            (PeerId("peer1".into()), "Hello from Alice"),
            (PeerId("peer2".into()), "Hello from Bob"),
            (PeerId("peer1".into()), "Another from Alice"),
        ];
        
        let filtered = book.filter_blocked(messages, |m| &m.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, PeerId("peer2".into()));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 24: Contact Storage Round-Trip
        #[test]
        fn prop_contact_storage_roundtrip(
            id in "[a-z0-9]{8}",
            name in "[A-Za-z ]{1,20}",
        ) {
            use crate::net::PeerId;
            let mut book = ContactBook::new();
            let bundle = mock_bundle();
            let peer_id = PeerId(id.clone());
            let contact = Contact::new(peer_id.clone(), name.clone(), bundle.clone());
            
            book.add(contact);
            
            let retrieved = book.get(&peer_id).unwrap();
            prop_assert_eq!(&retrieved.name, &name);
            prop_assert_eq!(&retrieved.public_bundle.pq, &bundle.pq);
            prop_assert_eq!(&retrieved.public_bundle.x25519, &bundle.x25519);
        }

        /// Property 25: Trust Flag Behavior
        #[test]
        fn prop_trust_flag_behavior(
            id in "[a-z0-9]{8}",
            name in "[A-Za-z ]{1,20}",
        ) {
            use crate::net::PeerId;
            let mut contact = Contact::new(PeerId(id), name, mock_bundle());
            
            contact.set_trusted();
            prop_assert_eq!(contact.trust_level, TrustLevel::Trusted);
        }

        /// Property 26: Block Filtering
        #[test]
        fn prop_block_filtering(
            blocked_ids in prop::collection::vec("[a-z0-9]{8}", 1..5),
            allowed_ids in prop::collection::vec("[a-z0-9]{8}", 1..5),
        ) {
            use crate::net::PeerId;
            let mut book = ContactBook::new();
            
            // Add blocked contacts
            for id in &blocked_ids {
                let mut contact = Contact::new(PeerId(id.clone()), "Blocked".into(), mock_bundle());
                contact.block();
                book.add(contact);
            }
            
            // Add allowed contacts
            for id in &allowed_ids {
                let contact = Contact::new(PeerId(id.clone()), "Allowed".into(), mock_bundle());
                book.add(contact);
            }
            
            // Create messages from all contacts with PeerId
            let mut messages: Vec<(PeerId, &str)> = Vec::new();
            for id in &blocked_ids {
                messages.push((PeerId(id.clone()), "blocked msg"));
            }
            for id in &allowed_ids {
                messages.push((PeerId(id.clone()), "allowed msg"));
            }
            
            let filtered = book.filter_blocked(messages, |m| &m.0);
            
            // No blocked messages should remain
            for msg in &filtered {
                prop_assert!(!book.is_blocked(&msg.0));
            }
        }

        /// Property 27: Bundle Change Detection
        #[test]
        fn prop_bundle_change_detection(
            id in "[a-z0-9]{8}",
            name in "[A-Za-z ]{1,20}",
            change_byte in 0usize..32,
        ) {
            use crate::net::PeerId;
            let bundle1 = mock_bundle();
            let mut bundle2 = mock_bundle();
            bundle2.x25519[change_byte] ^= 0xFF;  // Flip bits at position
            
            let contact = Contact::new(PeerId(id), name, bundle1.clone());
            
            prop_assert!(contact.verify_bundle_unchanged(&bundle1));
            prop_assert!(!contact.verify_bundle_unchanged(&bundle2));
        }
    }
}
