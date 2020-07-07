mod fileloader;
mod filemanager;
mod gpuloader;
mod gpumanager;
mod imagedata;

mod ronmanager;

pub use fileloader::FileLoadFuture;
pub use filemanager::AsyncFileManager;
use futures::{future::Shared, Future};
use std::{io::Error, path::PathBuf, sync::Arc};

///
pub enum LoadStatus<T, F>
where
    T: Unpin,
    F: Future<Output = Result<Arc<T>, Arc<Error>>>,
{
    NotLoading,
    Loading(Shared<F>),
    Loaded(Arc<T>),
    Error(Arc<std::io::Error>),
}

impl<T, F> PartialEq for LoadStatus<T, F>
where
    T: Unpin,
    F: Future<Output = Result<Arc<T>, Arc<Error>>>,
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

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
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
