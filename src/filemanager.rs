use crate::{LoadStatus, AsyncFileLoader};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    task::Poll,
};
use threadpool::ThreadPool;

#[allow(unused)]
pub struct AsyncFileManager<F>
where
    F: From<(PathBuf, Vec<u8>)>,
{
    pool: Arc<ThreadPool>,
    loading: FuturesUnordered<AsyncFileLoader>,
    paths_loading: HashSet<PathBuf>,
    cache: HashMap<PathBuf, Arc<F>>,
}

impl<F> AsyncFileManager<F>
where
    F: From<(PathBuf, Vec<u8>)>,
{
    #[allow(unused)]
    pub fn new(pool: Arc<ThreadPool>) -> Self {
        Self {
            pool,
            loading: futures::stream::FuturesUnordered::new(),
            paths_loading: HashSet::new(),
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
                    self.cache
                        .entry(p.clone())
                        .or_insert(Arc::new(F::from((p, b))));
                }
                Err(_e) => { // TODO: log this
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncFileManager;
    use std::{path::PathBuf, sync::Arc};
    #[derive(Debug, Eq, PartialEq)]
    struct LoadedFile {
        string: String,
    }

    impl From<(PathBuf, Vec<u8>)> for LoadedFile {
        fn from((_p, v): (PathBuf, Vec<u8>)) -> Self {
            LoadedFile {
                string: String::from_utf8(v).unwrap(),
            }
        }
    }
    #[test]
    fn it_works_2() {
        let pool = Arc::new(threadpool::Builder::new().build());
        let path = PathBuf::new().join("benches/benchfiles/s01");

        let mut manager = AsyncFileManager::<LoadedFile>::new(pool);
        futures::executor::block_on(manager.load(&path));
        std::thread::sleep(std::time::Duration::from_millis(500));
        let t = futures::executor::block_on(manager.get(&path));
        assert_eq!(t, Some(&Arc::new(LoadedFile::from((PathBuf::new(),b"\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\n\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest".to_vec())))));
    }
}
