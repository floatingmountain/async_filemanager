mod fileloader;
mod filemanager;
mod gpuloader;
mod gpumanager;
mod imagedata;

pub use fileloader::AsyncFileLoader;
pub use filemanager::AsyncFileManager;
use std::path::PathBuf;
///
#[derive(Debug, Eq, PartialEq)]
pub enum LoadStatus {
    NotLoading,
    Loading,
    Loaded,
}
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Identifier{
    Path(PathBuf),
    Index(usize),
}
impl From<PathBuf> for Identifier{
    fn from(p: PathBuf) -> Self {
        Identifier::Path(p)
    }
}
impl From<usize> for Identifier{
    fn from(u: usize) -> Self {
        Identifier::Index(u)
    }
}