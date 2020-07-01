mod fileloader;
mod filemanager;
mod gpuloader;
mod gpumanager;
mod imagedata;

pub use fileloader::AsyncFileLoader;
pub use filemanager::AsyncFileManager;
///
#[derive(Debug, Eq, PartialEq)]
pub enum LoadStatus {
    NotLoading,
    Loading,
    Loaded,
}