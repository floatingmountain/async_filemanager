mod fileloader;
mod filemanager;
//mod gpuloader;
//mod gpumanager;
//mod imagedata;

pub use fileloader::FileLoadFuture;
//pub use filemanager::AsyncFileManager;
use futures::future::Shared;
use std::{convert::TryFrom, path::PathBuf, sync::Arc};
///
pub enum LoadStatus<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)> + Unpin,
{
    NotLoading,
    Loading(Shared<FileLoadFuture<T>>),
    Loaded(Arc<T>),
    Error(Arc<std::io::Error>),
}
impl<T> PartialEq for LoadStatus<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)> + Unpin,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoadStatus::NotLoading, LoadStatus::NotLoading) => true,
            (LoadStatus::Loading(_l1), LoadStatus::Loading(_l2)) => true,
            (LoadStatus::Loaded(_t1), LoadStatus::Loaded(_t2)) => true,
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
