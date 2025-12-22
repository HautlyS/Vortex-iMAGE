//! Crux application core
use crux_core::{render::Render, Command};
use serde::{Deserialize, Serialize};
use crate::capabilities::{Network, NetworkResult, Fs, FsResult};
use crate::net::{Contact, PeerId, SharedItem};
use crate::sync::SyncDoc;
use crate::transfer::{Transfer, TransferStatus};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Model {
    pub node_id: Option<String>,
    pub tor_connected: bool,
    pub contacts: Vec<Contact>,
    pub shared_items: Vec<SharedItem>,
    pub active_doc: Option<SyncDoc>,
    pub status: AppStatus,
    pub public_key: Option<String>,
    pub transfers: Vec<Transfer>,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppStatus {
    #[default]
    Idle,
    Connecting,
    Connected,
    Syncing,
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    // Lifecycle
    Init,
    NodeReady { id: String, public_key: String },
    TorConnected,
    Error(String),
    
    // Contacts
    AddContact { name: String, peer_id: String, public_key: Vec<u8> },
    RemoveContact(PeerId),
    BlockContact { peer_id: String },
    UnblockContact { peer_id: String },
    TrustContact { peer_id: String },
    VerifyContact { peer_id: String },
    
    // File sharing
    ShareFile { data: Vec<u8>, recipient_key: Vec<u8> },
    FileShared(SharedItem),
    ReceiveFile { ticket: String },
    FileReceived { name: String, size: u64, data: Vec<u8> },
    
    // Transfers
    TransferStarted { id: String, name: String, size: u64 },
    TransferProgress { id: String, bytes: u64, total: u64 },
    TransferComplete { id: String },
    TransferError { id: String, error: String },
    CancelTransfer { id: String },
    
    // Documents
    OpenDoc(String),
    EditDoc { pos: usize, text: String },
    DeleteText { pos: usize, len: usize },
    SyncDoc,
    
    // Chat
    SendMessage { room_id: String, content: String },
    CreateRoom { name: String, is_group: bool },
    JoinRoom { room_id: String },
    
    // Capability responses
    NetworkResponse(NetworkResult),
    FsResponse(FsResult),
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ViewModel {
    pub node_id: String,
    pub status: String,
    pub tor_status: String,
    pub public_key: String,
    pub contacts: Vec<ContactView>,
    pub shared_items: Vec<SharedItemView>,
    pub transfers: Vec<TransferView>,
    pub doc_content: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContactView {
    pub id: String,
    pub name: String,
    pub trusted: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SharedItemView {
    pub name: String,
    pub size: String,
    pub shared_with: usize,
    pub ticket: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransferView {
    pub id: String,
    pub name: String,
    pub size: String,
    pub progress: u8,
    pub speed: String,
    pub status: String,
    pub direction: String,
}

#[derive(crux_core::macros::Effect)]
pub struct Capabilities {
    pub render: Render<Event>,
    pub network: Network<Event>,
    pub fs: Fs<Event>,
    #[effect(skip)]
    pub compose: crux_core::compose::Compose<Event>,
}

#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model, caps: &Capabilities) -> Command<Effect, Event> {
        match event {
            Event::Init => {
                model.status = AppStatus::Connecting;
                caps.network.start_node(Event::NetworkResponse);
                caps.render.render();
                Command::done()
            }
            
            Event::NetworkResponse(result) => {
                match result {
                    NetworkResult::NodeStarted { id, public_key } => {
                        model.node_id = Some(id);
                        model.public_key = Some(public_key);
                        model.status = AppStatus::Connected;
                    }
                    NetworkResult::TorConnected => {
                        model.tor_connected = true;
                    }
                    NetworkResult::BlobShared { ticket, hash, transfer_id } => {
                        if let Some(t) = model.transfers.iter_mut().find(|t| t.id == transfer_id) {
                            t.status = TransferStatus::Complete;
                            t.ticket = Some(ticket.clone());
                        }
                        model.shared_items.push(SharedItem {
                            name: format!("file_{}", &hash[..8]),
                            size: 0,
                            hash,
                            contacts: vec![],
                        });
                    }
                    NetworkResult::BlobDownloaded { data, transfer_id } => {
                        if let Some(t) = model.transfers.iter_mut().find(|t| t.id == transfer_id) {
                            t.status = TransferStatus::Complete;
                            t.bytes_transferred = data.len() as u64;
                        }
                    }
                    NetworkResult::Progress { transfer_id, bytes, total } => {
                        if let Some(t) = model.transfers.iter_mut().find(|t| t.id == transfer_id) {
                            t.bytes_transferred = bytes;
                            t.total_bytes = total;
                        }
                    }
                    NetworkResult::Cancelled { transfer_id } => {
                        model.transfers.retain(|t| t.id != transfer_id);
                    }
                    NetworkResult::Error(e) => {
                        model.status = AppStatus::Error(e);
                    }
                }
                caps.render.render();
                Command::done()
            }
            
            Event::FsResponse(result) => {
                match result {
                    FsResult::FileRead { data, size: _ } => {
                        // Handle file read - could trigger share
                        tracing::info!("File read: {} bytes", data.len());
                    }
                    FsResult::FileWritten { path } => {
                        tracing::info!("File written: {}", path);
                    }
                    _ => {}
                }
                caps.render.render();
                Command::done()
            }
            
            Event::NodeReady { id, public_key } => {
                model.node_id = Some(id);
                model.public_key = Some(public_key);
                model.status = AppStatus::Connected;
                caps.render.render();
                Command::done()
            }
            
            Event::TorConnected => {
                model.tor_connected = true;
                caps.render.render();
                Command::done()
            }
            
            Event::Error(e) => {
                model.status = AppStatus::Error(e);
                caps.render.render();
                Command::done()
            }
            
            Event::AddContact { name, peer_id, public_key } => {
                model.contacts.push(Contact {
                    id: PeerId(peer_id),
                    name,
                    public_key: public_key.try_into().unwrap_or([0u8; 32]),
                    trusted: false,
                });
                caps.render.render();
                Command::done()
            }
            
            Event::RemoveContact(id) => {
                model.contacts.retain(|c| c.id.0 != id.0);
                caps.render.render();
                Command::done()
            }
            
            Event::ShareFile { data, recipient_key } => {
                let transfer_id = format!("tx_{}", rand::random::<u32>());
                let size = data.len() as u64;
                let num_chunks = ((size as usize + crate::transfer::CHUNK_SIZE - 1) / crate::transfer::CHUNK_SIZE).max(1);
                model.transfers.push(Transfer {
                    id: transfer_id.clone(),
                    name: "Uploading...".into(),
                    total_bytes: size,
                    bytes_transferred: 0,
                    status: TransferStatus::Active,
                    direction: crate::transfer::TransferDirection::Upload,
                    ticket: None,
                    hash: None,
                    chunks_completed: vec![false; num_chunks],
                    chunk_tickets: Vec::new(),
                });
                model.status = AppStatus::Syncing;
                caps.network.share_blob(data, recipient_key, true, Event::NetworkResponse);
                caps.render.render();
                Command::done()
            }
            
            Event::FileShared(item) => {
                model.shared_items.push(item);
                model.status = AppStatus::Idle;
                caps.render.render();
                Command::done()
            }
            
            Event::ReceiveFile { ticket } => {
                let transfer_id = format!("rx_{}", rand::random::<u32>());
                model.transfers.push(Transfer {
                    id: transfer_id.clone(),
                    name: "Downloading...".into(),
                    total_bytes: 0,
                    bytes_transferred: 0,
                    status: TransferStatus::Active,
                    direction: crate::transfer::TransferDirection::Download,
                    ticket: Some(ticket.clone()),
                    hash: None,
                    chunks_completed: Vec::new(),
                    chunk_tickets: Vec::new(),
                });
                model.status = AppStatus::Syncing;
                caps.network.download_blob(ticket, Event::NetworkResponse);
                caps.render.render();
                Command::done()
            }
            
            Event::FileReceived { name, size, data: _ } => {
                model.status = AppStatus::Idle;
                tracing::info!("Received file: {} ({} bytes)", name, size);
                caps.render.render();
                Command::done()
            }
            
            Event::TransferStarted { id, name, size } => {
                let num_chunks = ((size as usize + crate::transfer::CHUNK_SIZE - 1) / crate::transfer::CHUNK_SIZE).max(1);
                model.transfers.push(Transfer {
                    id,
                    name,
                    total_bytes: size,
                    bytes_transferred: 0,
                    status: TransferStatus::Active,
                    direction: crate::transfer::TransferDirection::Upload,
                    ticket: None,
                    hash: None,
                    chunks_completed: vec![false; num_chunks],
                    chunk_tickets: Vec::new(),
                });
                caps.render.render();
                Command::done()
            }
            
            Event::TransferProgress { id, bytes, total } => {
                if let Some(t) = model.transfers.iter_mut().find(|t| t.id == id) {
                    t.bytes_transferred = bytes;
                    t.total_bytes = total;
                }
                caps.render.render();
                Command::done()
            }
            
            Event::TransferComplete { id } => {
                if let Some(t) = model.transfers.iter_mut().find(|t| t.id == id) {
                    t.status = TransferStatus::Complete;
                }
                model.status = AppStatus::Idle;
                caps.render.render();
                Command::done()
            }
            
            Event::TransferError { id, error } => {
                if let Some(t) = model.transfers.iter_mut().find(|t| t.id == id) {
                    t.status = TransferStatus::Error(error);
                }
                caps.render.render();
                Command::done()
            }
            
            Event::CancelTransfer { id } => {
                caps.network.cancel_transfer(id, Event::NetworkResponse);
                caps.render.render();
                Command::done()
            }
            
            Event::OpenDoc(id) => {
                model.active_doc = Some(SyncDoc::new(&id));
                caps.render.render();
                Command::done()
            }
            
            Event::EditDoc { pos, text } => {
                if let Some(doc) = &mut model.active_doc {
                    doc.apply(crate::sync::DocOp::Insert {
                        pos, text,
                        ver: crate::sync::DocVersion {
                            clock: doc.version + 1,
                            author: model.node_id.clone().unwrap_or_default(),
                        },
                    });
                }
                caps.render.render();
                Command::done()
            }
            
            Event::DeleteText { pos, len } => {
                if let Some(doc) = &mut model.active_doc {
                    doc.apply(crate::sync::DocOp::Delete {
                        pos, len,
                        ver: crate::sync::DocVersion {
                            clock: doc.version + 1,
                            author: model.node_id.clone().unwrap_or_default(),
                        },
                    });
                }
                caps.render.render();
                Command::done()
            }
            
            Event::SyncDoc => {
                model.status = AppStatus::Idle;
                caps.render.render();
                Command::done()
            }
            
            Event::SendMessage { room_id, content } => {
                tracing::info!("Sending message to {}: {}", room_id, content);
                // TODO: Broadcast via gossip
                caps.render.render();
                Command::done()
            }
            
            Event::CreateRoom { name, is_group } => {
                tracing::info!("Creating room: {} (group: {})", name, is_group);
                // TODO: Create room in chat module
                caps.render.render();
                Command::done()
            }
            
            Event::JoinRoom { room_id } => {
                tracing::info!("Joining room: {}", room_id);
                caps.render.render();
                Command::done()
            }
            
            Event::BlockContact { peer_id } => {
                tracing::info!("Blocking contact: {}", peer_id);
                caps.render.render();
                Command::done()
            }
            
            Event::UnblockContact { peer_id } => {
                tracing::info!("Unblocking contact: {}", peer_id);
                caps.render.render();
                Command::done()
            }
            
            Event::TrustContact { peer_id } => {
                tracing::info!("Trusting contact: {}", peer_id);
                caps.render.render();
                Command::done()
            }
            
            Event::VerifyContact { peer_id } => {
                tracing::info!("Verifying contact: {}", peer_id);
                caps.render.render();
                Command::done()
            }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        ViewModel {
            node_id: model.node_id.clone().unwrap_or_default(),
            status: match &model.status {
                AppStatus::Idle => "Ready",
                AppStatus::Connecting => "Connecting...",
                AppStatus::Connected => "Online",
                AppStatus::Syncing => "Syncing...",
                AppStatus::Error(e) => return ViewModel { status: format!("Error: {e}"), ..Default::default() },
            }.into(),
            tor_status: if model.tor_connected { "🧅 Tor" } else { "Direct" }.into(),
            public_key: model.public_key.clone().unwrap_or_default(),
            contacts: model.contacts.iter().map(|c| ContactView {
                id: c.id.0.clone(), name: c.name.clone(), trusted: c.trusted,
            }).collect(),
            shared_items: model.shared_items.iter().map(|i| SharedItemView {
                name: i.name.clone(),
                size: format_size(i.size),
                shared_with: i.contacts.len(),
                ticket: None,
            }).collect(),
            transfers: model.transfers.iter().map(|t| TransferView {
                id: t.id.clone(),
                name: t.name.clone(),
                size: format_size(t.total_bytes),
                progress: if t.total_bytes > 0 { ((t.bytes_transferred * 100) / t.total_bytes) as u8 } else { 0 },
                speed: format_speed(t.bytes_transferred, 1), // TODO: actual time
                status: match &t.status {
                    TransferStatus::Pending => "pending",
                    TransferStatus::Active => "active",
                    TransferStatus::Paused => "paused",
                    TransferStatus::Complete => "complete",
                    TransferStatus::Error(_) => "error",
                }.into(),
                direction: match t.direction {
                    crate::transfer::TransferDirection::Upload => "upload",
                    crate::transfer::TransferDirection::Download => "download",
                }.into(),
            }).collect(),
            doc_content: model.active_doc.as_ref().map(|d| d.content.clone()).unwrap_or_default(),
        }
    }
}

fn format_size(b: u64) -> String {
    match b {
        b if b >= 1073741824 => format!("{:.1} GB", b as f64 / 1073741824.0),
        b if b >= 1048576 => format!("{:.1} MB", b as f64 / 1048576.0),
        b if b >= 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{} B", b),
    }
}

fn format_speed(bytes: u64, secs: u64) -> String {
    if secs == 0 { return "-- B/s".into(); }
    let bps = bytes / secs;
    match bps {
        b if b >= 1048576 => format!("{:.1} MB/s", b as f64 / 1048576.0),
        b if b >= 1024 => format!("{:.1} KB/s", b as f64 / 1024.0),
        b => format!("{} B/s", b),
    }
}
