use crate::{FileLoadFuture, LoadStatus};
use futures::executor::ThreadPool;
use futures::{future::Shared, FutureExt};
use std::{
    collections::HashMap,
    convert::TryFrom,
    path::{Path, PathBuf},
    sync::Arc,
    task::Poll,
};

#[allow(unused)]
pub struct AsyncFileManager<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)> + Unpin,
{
    pool: Arc<ThreadPool>,
    loading: HashMap<PathBuf, Shared<FileLoadFuture<T>>>,
    cache: HashMap<PathBuf, Arc<T>>,
}

impl<T> AsyncFileManager<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)> + Unpin,
{
    #[allow(unused)]
    pub fn new(pool: Arc<ThreadPool>) -> Self {
        Self {
            pool,
            loading: HashMap::new(),
            cache: HashMap::new(),
        }
    }
    #[allow(unused)]
    pub async fn load<P: AsRef<Path>>(&mut self, path: P) {
        if !self.cache.contains_key(path.as_ref()) && !self.loading.contains_key(path.as_ref()) {
            let mut f = FileLoadFuture::new(path.as_ref(), self.pool.clone()).shared();
            futures::poll!(&mut f);
            self.loading.insert(path.as_ref().to_owned(), f);
        }
    }
    #[allow(unused)]
    pub async fn get<P: AsRef<Path>>(&mut self, path: P) -> LoadStatus<T> {
        if let Some(f) = self.loading.get_mut(path.as_ref()) {
            if let Poll::Ready(result) = futures::poll!(f) {
                self.loading.remove(path.as_ref());
                match result {
                    Ok(t) => {
                        self.cache
                            .entry(path.as_ref().to_owned())
                            .or_insert(t.clone());
                        LoadStatus::Loaded(t)
                    }
                    Err(e) => LoadStatus::Error(e),
                }
            } else {
                LoadStatus::Loading(self.loading.get(path.as_ref()).unwrap().clone())
            }
        } else if let Some(f) = self.cache.get(path.as_ref()) {
            LoadStatus::Loaded(f.clone())
        } else {
            LoadStatus::NotLoading
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncFileManager;
    use crate::LoadStatus;
    use futures::executor::ThreadPoolBuilder;
    use std::{convert::TryFrom, path::PathBuf, sync::Arc};

    #[derive(Debug, Eq, PartialEq)]
    struct LoadedFile {
        string: String,
    }

    impl TryFrom<(PathBuf, Vec<u8>)> for LoadedFile {
        type Error = std::string::FromUtf8Error;
        fn try_from((_path, bytes): (PathBuf, Vec<u8>)) -> Result<Self, Self::Error> {
            Ok(LoadedFile {
                string: String::from_utf8(bytes)?,
            })
        }
    }
    #[test]
    fn manager() {
        let pool = Arc::new(ThreadPoolBuilder::new().create().unwrap());
        let path = PathBuf::new().join("benches/benchfiles/s01");

        let mut manager = AsyncFileManager::<LoadedFile>::new(pool);
        futures::executor::block_on(async {
            manager.load(&path).await;
            match manager.get(&path).await {
                LoadStatus::Loading(f) => println!("{:?}", f.await),
                LoadStatus::Loaded(f) => println!("{:?}", f),
                _ => panic!(),
            }
            if let LoadStatus::Loaded(file) = manager.get(&path).await {
                println!("{:?}", file)
            }
            manager.load(&path).await;

            match manager.get(&path).await {
                LoadStatus::Loaded(f) => println!("{:?}", f),
                _ => panic!(),
            }
        });
        //assert_eq!(t, Some(&Arc::new(LoadedFile::try_from((PathBuf::new(),b"\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\n\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest".to_vec())).unwrap())));
    }
}
