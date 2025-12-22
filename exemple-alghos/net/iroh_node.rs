//! Iroh P2P node with encrypted blob transfer + Tor integration
use anyhow::Result;
use bytes::Bytes;
use iroh::{protocol::Router, Endpoint, NodeId, SecretKey};
use iroh_blobs::{
    downloader::DownloadRequest,
    net_protocol::Blobs,
    store::{mem::Store as MemStore, Map, MapEntry, Store},
    ticket::BlobTicket,
    BlobFormat, HashAndFormat,
};
use iroh_io::AsyncSliceReader;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::compress::{self, decompress_segmented_sync};
use crate::crypto::{self, EncryptedPayload, HybridKeypair, PublicBundle};

#[cfg(feature = "tor")]
use super::tor_transport::TorTransport;

pub struct IrohNode {
    endpoint: Endpoint,
    blobs: Blobs<MemStore>,
    router: Option<Router>,
    keypair: HybridKeypair,
    #[cfg(feature = "tor")]
    tor: Option<Arc<TorTransport>>,
}

impl IrohNode {
    pub async fn new() -> Result<Self> {
        let endpoint = Endpoint::builder()
            .discovery_n0()
            .bind()
            .await?;
        
        let blobs = Blobs::memory().build(&endpoint);
        let keypair = HybridKeypair::generate();
        
        Ok(Self {
            endpoint,
            blobs,
            router: None,
            keypair,
            #[cfg(feature = "tor")]
            tor: None,
        })
    }

    pub async fn with_secret(secret: SecretKey) -> Result<Self> {
        let endpoint = Endpoint::builder()
            .secret_key(secret)
            .discovery_n0()
            .bind()
            .await?;
        
        let blobs = Blobs::memory().build(&endpoint);
        let keypair = HybridKeypair::generate();
        
        Ok(Self {
            endpoint,
            blobs,
            router: None,
            keypair,
            #[cfg(feature = "tor")]
            tor: None,
        })
    }

    #[cfg(feature = "tor")]
    pub async fn enable_tor(&mut self) -> Result<()> {
        let tor = TorTransport::bootstrap().await?;
        self.tor = Some(Arc::new(tor));
        Ok(())
    }

    #[cfg(feature = "tor")]
    pub fn is_tor_enabled(&self) -> bool {
        self.tor.is_some()
    }

    #[cfg(not(feature = "tor"))]
    pub fn is_tor_enabled(&self) -> bool {
        false
    }

    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
    
    pub fn public_bundle(&self) -> PublicBundle {
        self.keypair.public_bundle()
    }

    pub fn keypair(&self) -> &HybridKeypair {
        &self.keypair
    }

    pub async fn start_server(&mut self) -> Result<()> {
        let router = Router::builder(self.endpoint.clone())
            .accept(iroh_blobs::ALPN, self.blobs.clone())
            .spawn();
        self.router = Some(router);
        Ok(())
    }

    /// Add encrypted + compressed blob (PQ-secure)
    pub async fn add_blob_encrypted(&self, data: &[u8], recipient: &PublicBundle) -> Result<BlobTicket> {
        let compressed = compress::compress_segmented(data, 10).await?;
        let encrypted = crypto::encrypt(&compressed, recipient)?;
        let payload = bincode::serialize(&encrypted)?;
        
        let store = self.blobs.store();
        let tag = store.import_bytes(Bytes::from(payload), BlobFormat::Raw).await?;
        let node_addr = self.endpoint.node_addr().await?;
        
        BlobTicket::new(node_addr, *tag.hash(), tag.format())
    }

    /// Add encrypted blob with progress callback
    pub async fn add_blob_encrypted_with_progress<F>(
        &self,
        data: &[u8],
        recipient: &PublicBundle,
        mut on_progress: F,
    ) -> Result<BlobTicket>
    where
        F: FnMut(u64, u64) + Send,
    {
        let total = data.len() as u64;
        on_progress(0, total);
        
        // Compress with progress (simulated at 50%)
        let compressed = compress::compress_segmented(data, 10).await?;
        on_progress(total / 2, total);
        
        // Encrypt
        let encrypted = crypto::encrypt(&compressed, recipient)?;
        let payload = bincode::serialize(&encrypted)?;
        on_progress(total * 3 / 4, total);
        
        // Store
        let store = self.blobs.store();
        let tag = store.import_bytes(Bytes::from(payload), BlobFormat::Raw).await?;
        let node_addr = self.endpoint.node_addr().await?;
        
        on_progress(total, total);
        BlobTicket::new(node_addr, *tag.hash(), tag.format())
    }

    /// Add unencrypted compressed blob
    pub async fn add_blob(&self, data: &[u8]) -> Result<BlobTicket> {
        let compressed = compress::compress_async(data, 10).await?;
        let store = self.blobs.store();
        let tag = store.import_bytes(compressed, BlobFormat::Raw).await?;
        let node_addr = self.endpoint.node_addr().await?;
        BlobTicket::new(node_addr, *tag.hash(), tag.format())
    }

    /// Add raw blob (no compression)
    pub async fn add_blob_raw(&self, data: Bytes) -> Result<BlobTicket> {
        let store = self.blobs.store();
        let tag = store.import_bytes(data, BlobFormat::Raw).await?;
        let node_addr = self.endpoint.node_addr().await?;
        BlobTicket::new(node_addr, *tag.hash(), tag.format())
    }

    /// Download and decrypt blob
    pub async fn download_encrypted(&self, ticket: &BlobTicket) -> Result<Bytes> {
        let raw = self.download_raw(ticket).await?;
        let encrypted: EncryptedPayload = bincode::deserialize(&raw)?;
        let compressed = crypto::decrypt(&encrypted, &self.keypair)?;
        
        // Use sync decompression to avoid Send bound issues with async BufReader
        // The data is already in memory, so sync is fine here
        decompress_segmented_sync(&compressed).map_err(Into::into)
    }

    /// Download unencrypted blob
    pub async fn download(&self, ticket: &BlobTicket) -> Result<Bytes> {
        let raw = self.download_raw(ticket).await?;
        compress::decompress_async(&raw).await.map_err(Into::into)
    }

    /// Download raw blob (no decompression)
    pub async fn download_raw(&self, ticket: &BlobTicket) -> Result<Bytes> {
        let peer_addr = ticket.node_addr().clone();
        self.endpoint.add_node_addr(peer_addr.clone())?;
        
        let request = DownloadRequest::new(HashAndFormat::raw(ticket.hash()), vec![peer_addr]);
        self.blobs.downloader().queue(request).await.await?;
        
        let store = self.blobs.store();
        let entry = store.get(&ticket.hash()).await?.ok_or_else(|| anyhow::anyhow!("blob not found"))?;
        let size = entry.size().value() as usize;
        let mut reader = entry.data_reader().await?;
        
        // The reader's read_at is actually sync internally, use block_in_place
        let data = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                reader.read_at(0, size).await
            })
        })?;
        
        Ok(data)
    }

    /// Get blob size without downloading
    pub async fn blob_size(&self, ticket: &BlobTicket) -> Result<u64> {
        let store = self.blobs.store();
        if let Some(entry) = store.get(&ticket.hash()).await? {
            Ok(entry.size().value())
        } else {
            Err(anyhow::anyhow!("blob not found"))
        }
    }

    /// Check if blob exists locally
    pub async fn has_blob(&self, ticket: &BlobTicket) -> Result<bool> {
        let store = self.blobs.store();
        Ok(store.get(&ticket.hash()).await?.is_some())
    }

    // ============ VORTEX CODEC METHODS ============
    
    /// Add blob with VortexCodec (nonary) compression + encryption
    /// Best for structured/repetitive data
    pub async fn add_blob_vortex(&self, data: &[u8], recipient: &PublicBundle) -> Result<BlobTicket> {
        use crate::compress::vortex_compress_segmented;
        
        // Vortex compression (nonary encoding)
        let compressed = vortex_compress_segmented(data, 64 * 1024).await;
        
        // PQ encryption
        let encrypted = crypto::encrypt(&compressed, recipient)?;
        let payload = bincode::serialize(&encrypted)?;
        
        let store = self.blobs.store();
        let tag = store.import_bytes(Bytes::from(payload), BlobFormat::Raw).await?;
        let node_addr = self.endpoint.node_addr().await?;
        
        BlobTicket::new(node_addr, *tag.hash(), tag.format())
    }

    /// Download and decompress VortexCodec blob
    pub async fn download_vortex(&self, ticket: &BlobTicket) -> Result<Bytes> {
        use crate::compress::vortex_decompress_segmented;
        
        let raw = self.download_raw(ticket).await?;
        let encrypted: EncryptedPayload = bincode::deserialize(&raw)?;
        let compressed = crypto::decrypt(&encrypted, &self.keypair)?;
        
        vortex_decompress_segmented(&compressed).await
            .ok_or_else(|| anyhow::anyhow!("vortex decompression failed"))
    }

    /// Add blob with adaptive compression (auto-selects best algorithm)
    pub async fn add_blob_adaptive(&self, data: &[u8], recipient: &PublicBundle) -> Result<BlobTicket> {
        use crate::compress::{adaptive_compress, calculate_entropy};
        
        let entropy = calculate_entropy(data);
        
        // Choose compression based on entropy
        let compressed = if entropy < 4.0 {
            // Low entropy: use Vortex
            crate::compress::vortex_compress(data)
        } else if entropy < 7.0 {
            // Medium entropy: use adaptive
            adaptive_compress(data)
        } else {
            // High entropy: use Zstd
            compress::compress_sync(data, 3)?
        };
        
        // PQ encryption
        let encrypted = crypto::encrypt(&compressed, recipient)?;
        let payload = bincode::serialize(&encrypted)?;
        
        let store = self.blobs.store();
        let tag = store.import_bytes(Bytes::from(payload), BlobFormat::Raw).await?;
        let node_addr = self.endpoint.node_addr().await?;
        
        BlobTicket::new(node_addr, *tag.hash(), tag.format())
    }

    /// Download with adaptive decompression
    pub async fn download_adaptive(&self, ticket: &BlobTicket) -> Result<Bytes> {
        use crate::compress::adaptive_decompress;
        
        let raw = self.download_raw(ticket).await?;
        let encrypted: EncryptedPayload = bincode::deserialize(&raw)?;
        let compressed = crypto::decrypt(&encrypted, &self.keypair)?;
        
        // Try adaptive first, fall back to others
        if let Some(data) = adaptive_decompress(&compressed) {
            return Ok(data);
        }
        
        // Try vortex
        if let Some(data) = crate::compress::vortex_decompress(&compressed) {
            return Ok(data);
        }
        
        // Try standard decompress
        compress::decompress_sync(&compressed).map_err(Into::into)
    }

    pub async fn shutdown(self) -> Result<()> {
        if let Some(router) = self.router {
            router.shutdown().await?;
        }
        self.endpoint.close().await;
        Ok(())
    }
}

pub type SharedNode = Arc<RwLock<IrohNode>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let node = IrohNode::new().await.unwrap();
        assert!(!node.node_id().to_string().is_empty());
    }

    #[tokio::test]
    async fn test_add_blob() {
        let node = IrohNode::new().await.unwrap();
        let data = b"test data";
        let ticket = node.add_blob(data).await.unwrap();
        assert!(node.has_blob(&ticket).await.unwrap());
    }

    #[tokio::test]
    async fn test_encrypted_blob() {
        let node = IrohNode::new().await.unwrap();
        let data = b"secret message";
        let bundle = node.public_bundle();
        let ticket = node.add_blob_encrypted(data, &bundle).await.unwrap();
        assert!(node.has_blob(&ticket).await.unwrap());
    }

    #[tokio::test]
    async fn test_large_blob() {
        let node = IrohNode::new().await.unwrap();
        let data = vec![42u8; 100_000];
        let ticket = node.add_blob(&data).await.unwrap();
        let size = node.blob_size(&ticket).await.unwrap();
        assert!(size > 0);
    }

    #[tokio::test]
    async fn test_keypair_persistence() {
        let node = IrohNode::new().await.unwrap();
        let bundle1 = node.public_bundle();
        let bundle2 = node.public_bundle();
        assert_eq!(bundle1.pq, bundle2.pq);
        assert_eq!(bundle1.x25519, bundle2.x25519);
    }
}

