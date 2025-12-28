//! Filesystem capability for file operations
use crux_core::capability::{CapabilityContext, Operation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsOp {
    ReadFile { path: String },
    WriteFile { path: String, data: Vec<u8> },
    ListDir { path: String },
    FileInfo { path: String },
    DeleteFile { path: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsResult {
    FileRead { data: Vec<u8>, size: u64 },
    FileWritten { path: String },
    DirListed { entries: Vec<FileEntry> },
    FileInfo(FileEntry),
    FileDeleted,
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

impl Operation for FsOp {
    type Output = FsResult;
}

pub struct Fs<Ev> {
    context: CapabilityContext<FsOp, Ev>,
}

impl<Ev> Fs<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<FsOp, Ev>) -> Self {
        Self { context }
    }

    pub fn read_file<F>(&self, path: String, callback: F)
    where
        F: FnOnce(FsResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(FsOp::ReadFile { path }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn write_file<F>(&self, path: String, data: Vec<u8>, callback: F)
    where
        F: FnOnce(FsResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(FsOp::WriteFile { path, data }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn list_dir<F>(&self, path: String, callback: F)
    where
        F: FnOnce(FsResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(FsOp::ListDir { path }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn file_info<F>(&self, path: String, callback: F)
    where
        F: FnOnce(FsResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(FsOp::FileInfo { path }).await;
                ctx.update_app(callback(result));
            }
        });
    }

    pub fn delete_file<F>(&self, path: String, callback: F)
    where
        F: FnOnce(FsResult) -> Ev + Send + 'static,
    {
        self.context.spawn({
            let ctx = self.context.clone();
            async move {
                let result = ctx.request_from_shell(FsOp::DeleteFile { path }).await;
                ctx.update_app(callback(result));
            }
        });
    }
}

impl<Ev> crux_core::Capability<Ev> for Fs<Ev> {
    type Operation = FsOp;
    type MappedSelf<MappedEv> = Fs<MappedEv>;

    fn map_event<F, NewEv>(&self, f: F) -> Self::MappedSelf<NewEv>
    where
        F: Fn(NewEv) -> Ev + Send + Sync + 'static,
        Ev: 'static,
        NewEv: 'static + Send,
    {
        Fs::new(self.context.map_event(f))
    }
}
