use futures::Future;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
};
use crossbeam_channel::{bounded, Receiver};
use std::{io::ErrorKind, task::Poll};
use futures::executor::ThreadPool;

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
                let (tx, rx) = bounded(1);
                let w = cx.waker().clone();
                self.pool.spawn_ok(async move {
                    tx.send((path.clone(), std::fs::read(path)))
                        .expect("Error forwarding loaded data!");
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
    use super::AsyncFileLoader;
    use std::{path::PathBuf, sync::Arc};
    use futures::executor::ThreadPoolBuilder;

    #[test]
    fn it_works() {
        let pool = Arc::new(ThreadPoolBuilder::new().create().unwrap());
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
}
