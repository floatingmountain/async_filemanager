mod fileloader;
mod filemanager;
mod gpuloader;
mod gpumanager;
mod imagedata;

pub use fileloader::AsyncFileLoader;
pub use filemanager::AsyncFileManager;
use std::path::PathBuf;
///
#[derive(Debug)]
pub enum LoadStatus {
    NotLoading,
    Loading,
    Loaded,
    Error(std::io::Error)
}
impl PartialEq for LoadStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoadStatus::NotLoading, LoadStatus::NotLoading) => true,
            (LoadStatus::Loading, LoadStatus::Loading) => true,
            (LoadStatus::Loaded, LoadStatus::Loaded) => true,
            (LoadStatus::Error(e1), LoadStatus::Error(e2)) => e1.kind().eq(&e2.kind()),
            _ => false,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Identifier {
    Path(PathBuf),
    Index(usize),
}
impl From<PathBuf> for Identifier {
    fn from(p: PathBuf) -> Self {
        Identifier::Path(p)
    }
}
impl From<usize> for Identifier {
    fn from(u: usize) -> Self {
        Identifier::Index(u)
    }
}
