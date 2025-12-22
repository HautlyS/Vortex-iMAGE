//! Vortex shared library - P2P Social Platform
//! Chat, Drive, Docs, Watch Party, Social Feed, Communities, Tasks, Meetings - All encrypted
//! 
//! ## Causal Layer (Quantum-Inspired)
//! - `clock`: Vector clocks for causal ordering
//! - `causal`: Causal message delivery
//! - `cache`: LRU cache with TTL

// Core modules
pub mod app;
pub mod cache;
pub mod capabilities;
pub mod causal;
pub mod channel;
pub mod chat;
pub mod clock;
pub mod community;
pub mod compress;
pub mod contact;
pub mod crypto;
pub mod crdt;
pub mod drive;
pub mod meetings;
pub mod messaging;
pub mod net;
pub mod party;
pub mod persistence;
pub mod queue;
pub mod room;
pub mod social;
pub mod storage;
pub mod stream;
pub mod sync;
pub mod tasks;
pub mod transfer;

// Re-exports
pub use app::{App, Event, Model, ViewModel, AppStatus};
pub use cache::VortexCache;
pub use capabilities::{Network, NetworkOp, NetworkResult, Fs, FsOp, FsResult};
pub use causal::{CausalMessage, CausalBuffer};
pub use chat::{Message, MessageContent, ChatRoom, RoomMember, EncryptedMessage, MessageQueue, QueuedMessage, HistoryReconciler};
pub use clock::VectorClock;
pub use community::{Community, Channel, ChannelType, Member, MemberPresence, Role, Permissions};
pub use contact::{Contact as ContactInfo, ContactBook, TrustLevel, Presence};
pub use drive::{DriveEntry, SharedFolder, SyncStatus, DriveEvent, Permission, ConflictInfo};
pub use meetings::{Meeting, MeetingStatus, MeetingManager, CallSignal, SignalType, IceCandidate};
pub use net::{Contact, PeerId, SharedItem, ConnectionStatus, CircuitInfo, NetworkStats, GossipNetwork, GossipPayload, PresenceStatus, P2PMessenger, P2PEvent};
pub use persistence::{FileMetadata, SyncState, FolderSync, SCHEMA as DB_SCHEMA};
pub use room::{WatchParty, MediaInfo, PlaybackState, PartyEvent, Participant};
pub use social::{Post, PostContent, Story, Feed, Reaction, Comment, GossipMessage, TopicId, dm_topic, random_topic, SocialState, InviteCode, new_message};
pub use sync::{SyncDoc, DocOp, DocVersion, Cursor, TextStyle};
pub use tasks::{TaskList, Task, SubTask, Priority, TaskOp, TaskUpdate, TaskManager};
pub use transfer::{Transfer, TransferStatus, TransferDirection, TransferManager, CHUNK_SIZE};
pub use party::{PartyManager, WatchParty as Party, PartySync, PartyInvite, MediaSource, MediaInfo as PartyMedia, ChatMessage as PartyChatMessage, extract_youtube_id};
