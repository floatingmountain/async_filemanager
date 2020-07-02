use crate::{AsyncFileLoader, LoadStatus};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    path::{Path, PathBuf},
    sync::Arc,
    task::Poll,
};
use futures::executor::ThreadPool;

#[allow(unused)]
pub struct AsyncFileManager<F>
where
    F: TryFrom<(PathBuf, Vec<u8>)>,
{
    pool: Arc<ThreadPool>,
    loading: FuturesUnordered<AsyncFileLoader>,
    paths_loading: HashSet<PathBuf>,
    load_errors: HashMap<PathBuf, std::io::Error>,
    cache: HashMap<PathBuf, Arc<F>>,
}

impl<F> AsyncFileManager<F>
where
    F: TryFrom<(PathBuf, Vec<u8>)>,
{
    #[allow(unused)]
    pub fn new(pool: Arc<ThreadPool>) -> Self {
        Self {
            pool,
            loading: futures::stream::FuturesUnordered::new(),
            paths_loading: HashSet::new(),
            load_errors: HashMap::new(),
            cache: HashMap::new(),
        }
    }
    #[allow(unused)]
    pub async fn load<P: AsRef<Path>>(&mut self, path: P) {
        if !self.cache.contains_key(path.as_ref()) {
            self.loading.push(AsyncFileLoader::new(
                path.as_ref().to_owned(),
                self.pool.clone(),
            ));
            self.paths_loading.insert(path.as_ref().to_owned());
        }
        self.update().await;
    }
    #[allow(unused)]
    pub async fn get<P: AsRef<Path>>(&mut self, path: P) -> Option<&Arc<F>> {
        self.update().await;
        self.cache.get(path.as_ref())
    }
    #[allow(unused)]
    pub async fn remove<P: AsRef<Path>>(&mut self, path: P) -> Option<Arc<F>> {
        self.update().await;
        self.cache.remove(path.as_ref())
    }
    #[allow(unused)]
    pub async fn status<P: AsRef<Path>>(&mut self, path: P) -> LoadStatus {
        self.update().await;
        if self.paths_loading.contains(path.as_ref()) {
            LoadStatus::Loading
        } else if let Some(error) = self.load_errors.remove(path.as_ref()){
            LoadStatus::Error(error)
        } else {
            if let Some(_) = self.cache.get(path.as_ref()) {
                LoadStatus::Loaded
            } else {
                LoadStatus::NotLoading
            }
        }
    }
    async fn update(&mut self) {
        while let Poll::Ready(Some((p, r))) = futures::poll!(self.loading.next()) {
            self.paths_loading.remove(&p);
            match r {
                Ok(b) => {
                    match F::try_from((p.clone(), b)) {
                        Ok(f) => {
                            self.cache.entry(p.clone()).or_insert(Arc::new(f));
                        }
                        Err(_e) => {  // TODO: log this
                            self.load_errors.entry(p).or_insert(std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not convert raw Data!"));
                        }
                    }
                }
                Err(e) => {
                    self.load_errors.entry(p).or_insert(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncFileManager;
    use std::{path::PathBuf, sync::Arc, convert::TryFrom};
    use futures::executor::ThreadPoolBuilder;

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
    fn it_works_2() {
        let pool = Arc::new(ThreadPoolBuilder::new().create().unwrap());
        let path = PathBuf::new().join("benches/benchfiles/s01");

        let mut manager = AsyncFileManager::<LoadedFile>::new(pool);
        futures::executor::block_on(manager.load(&path));
        std::thread::sleep(std::time::Duration::from_millis(500));
        let t = futures::executor::block_on(manager.get(&path));
        assert_eq!(t, Some(&Arc::new(LoadedFile::try_from((PathBuf::new(),b"\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\n\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest".to_vec())).unwrap())));
    }
}
