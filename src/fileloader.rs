use futures::StreamExt;
use futures::{stream::FuturesUnordered, Future};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{
    mpsc::{sync_channel, Receiver},
    Arc,
};
use std::{collections::HashMap, io::ErrorKind, task::Poll};
use threadpool::ThreadPool;
#[allow(unused)]
pub struct FileManager<C>
where
    C: Codec,
{
    pool: Arc<ThreadPool>,
    loading: FuturesUnordered<AsyncFileLoader>,
    cache: HashMap<PathBuf, C>,
}

impl<C> FileManager<C>
where
    C: Codec,
{
    #[allow(unused)]
    pub fn new(pool: Arc<ThreadPool>) -> Self {
        Self {
            pool,
            loading: futures::stream::FuturesUnordered::new(),
            cache: HashMap::new(),
        }
    }
    #[allow(unused)]
    pub async fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref().to_owned();
        if !self.cache.contains_key(&path) {
            self.loading
                .push(AsyncFileLoader::new(path, self.pool.clone()));
        }
        self.update().await;
    }
    #[allow(unused)]
    pub async fn get<P: AsRef<Path>>(&mut self, path: P) -> Option<&C> {
        self.update().await;
        self.cache.get(path.as_ref())
    }
    async fn update(&mut self) {
        while let Poll::Ready(Some((p, r))) = futures::poll!(self.loading.next()) {
            match r {
                Ok(b) => {
                    self.cache.entry(p).or_insert(C::decode(&b));
                }
                Err(_e) => (),
            }
        }
    }
}

pub trait Codec
where
    Self: Serialize + DeserializeOwned,
{
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &Vec<u8>) -> Self;
}

pub struct AsyncFileLoader {
    path: PathBuf,
    pool: Arc<ThreadPool>,
    status: LoadStatus,
}

impl AsyncFileLoader {
    pub fn new<P: AsRef<Path>>(path: P, pool: Arc<ThreadPool>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            pool,
            status: LoadStatus::Path,
        }
    }
}

enum LoadStatus {
    Path,
    Loading(Receiver<(PathBuf, Result<Vec<u8>, std::io::Error>)>),
}

impl Future for AsyncFileLoader {
    type Output = (PathBuf, Result<Vec<u8>, std::io::Error>);
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let path = self.path.clone();
        match &self.status {
            LoadStatus::Path => {
                let (tx, rx) = sync_channel(1);
                let w = cx.waker().clone();
                self.pool.execute(move || {
                    tx.send((path.clone(), std::fs::read(path))).unwrap();
                    w.wake();
                });
                self.get_mut().status = LoadStatus::Loading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::Loading(rx) => match rx.recv() {
                Ok(r) => Poll::Ready(r),
                Err(e) => Poll::Ready((path, Err(std::io::Error::new(ErrorKind::BrokenPipe, e)))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncFileLoader, Codec, FileManager};
    use std::{path::PathBuf, sync::Arc};
    use threadpool::Builder;

    impl Codec for String {
        fn encode(&self) -> Vec<u8> {
            todo!()
        }
        fn decode(bytes: &Vec<u8>) -> Self {
            String::from_utf8(bytes.clone()).unwrap()
        }
    }
    #[test]
    fn it_works() {
        let pool = Arc::new(Builder::new().build());
        let path = PathBuf::new().join("benches/benchfiles/s01");
        let l = AsyncFileLoader::new(&path, pool);
        async_std::task::block_on(async {
            let (p, r) = l.await;
            assert_eq!(
                r.unwrap(),
                vec![
                    13, 10, 116, 101, 115, 116, 13, 10, 13, 10, 116, 101, 115, 116, 13, 10, 116,
                    101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116,
                    101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116,
                    101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 13, 10, 13, 10, 116,
                    101, 115, 116, 13, 10, 13, 10, 116, 101, 115, 116, 13, 10, 116, 101, 115, 116,
                    116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116,
                    116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116,
                    116, 101, 115, 116, 116, 101, 115, 116, 13, 10, 116, 101, 115, 116, 13, 10, 13,
                    10, 116, 101, 115, 116, 13, 10, 116, 101, 115, 116, 116, 101, 115, 116, 116,
                    101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116,
                    101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116,
                    101, 115, 116
                ]
            );
            assert_eq!(p, path);
        })
    }
    #[test]
    fn it_works_2() {
        let pool = Arc::new(Builder::new().build());
        let path = PathBuf::new().join("benches/benchfiles/s01");

        let mut manager = FileManager::<String>::new(pool);
        futures::executor::block_on(manager.load(&path));
        std::thread::sleep(std::time::Duration::from_millis(500));
        let t = futures::executor::block_on(manager.get(&path));
        assert_eq!(t, Some(&String::from("\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\n\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest")));
    }
}
