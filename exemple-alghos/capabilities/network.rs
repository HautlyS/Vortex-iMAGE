//! Network capability for P2P operations
use crux_core::capability::{CapabilityContext, Operation};
use serde::{Deserialize, Serialize};

/// Network operations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkOp {
    StartNode,
    ConnectTor,
    ShareBlob {
        data: Vec<u8>,
        recipient_key: Vec<u8>,
        #[serde(default)]
        compress: bool,
    },
    DownloadBlob { ticket: String },
    GetProgress { transfer_id: String },
    CancelTransfer { transfer_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkResult {
    NodeStarted { id: String, public_key: String },
    TorConnected,
    BlobShared { ticket: String, hash: String, transfer_id: String },
    BlobDownloaded { data: Vec<u8>, transfer_id: String },
    Progress { transfer_id: String, bytes: u64, total: u64 },
    Cancelled { transfer_id: String },
    Error(String),
}

impl Operation for NetworkOp {
    type Output = NetworkResult;
}

pub struct Network<Ev> {
    context: CapabilityContext<NetworkOp, Ev>,
}

impl<Ev> Network<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<NetworkOp, Ev>) -> Self {
        Self { context }
    }

    pub fn start_node<F>(&self, callback: F)
    where
        F: FnOnce(NetworkResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(NetworkOp::StartNode).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn connect_tor<F>(&self, callback: F)
    where
        F: FnOnce(NetworkResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(NetworkOp::ConnectTor).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn share_blob<F>(&self, data: Vec<u8>, recipient_key: Vec<u8>, compress: bool, callback: F)
    where
        F: FnOnce(NetworkResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(NetworkOp::ShareBlob { data, recipient_key, compress }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn download_blob<F>(&self, ticket: String, callback: F)
    where
        F: FnOnce(NetworkResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(NetworkOp::DownloadBlob { ticket }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn get_progress<F>(&self, transfer_id: String, callback: F)
    where
        F: FnOnce(NetworkResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(NetworkOp::GetProgress { transfer_id }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn cancel_transfer<F>(&self, transfer_id: String, callback: F)
    where
        F: FnOnce(NetworkResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(NetworkOp::CancelTransfer { transfer_id }).await;
                ctx.update_app(callback(result));
            }
        });
    }
}

impl<Ev> crux_core::Capability<Ev> for Network<Ev> {
    type Operation = NetworkOp;
    type MappedSelf<MappedEv> = Network<MappedEv>;

    fn map_event<F, NewEv>(&self, f: F) -> Self::MappedSelf<NewEv>
    where
        F: Fn(NewEv) -> Ev + Send + Sync + 'static,
        Ev: 'static,
        NewEv: 'static + Send,
    {
        Network::new(self.context.map_event(f))
    }
}
