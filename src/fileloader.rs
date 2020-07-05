use crossbeam_channel::{bounded, Receiver, TryRecvError};
use futures::Future;
use futures::{executor::ThreadPool, task::AtomicWaker};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{
    convert::TryFrom,
    io::{Error, ErrorKind},
    marker::PhantomData,
    task::Poll,
};

pub struct FileLoadFuture<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)>,
{
    path: PathBuf,
    pool: Arc<ThreadPool>,
    status: LoadStatus,
    waker: Arc<AtomicWaker>,
    _phantomdata: PhantomData<T>,
}

impl<T> FileLoadFuture<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)>,
{
    pub fn new<P: AsRef<Path>>(path: P, pool: Arc<ThreadPool>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            pool,
            status: LoadStatus::Path,
            waker: Arc::new(AtomicWaker::new()),
            _phantomdata: PhantomData::default(),
        }
    }
}

enum LoadStatus {
    Path,
    Loading(Receiver<Result<Vec<u8>, std::io::Error>>),
}

impl<T> Future for FileLoadFuture<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)> + Unpin,
{
    type Output = Result<Arc<T>, Arc<Error>>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let path = self.path.clone();
        match &self.status {
            LoadStatus::Path => {
                let (tx, rx) = bounded(1);
                self.waker.register(cx.waker());
                let waker = self.waker.clone();
                self.pool.spawn_ok(async move {
                    tx.send(std::fs::read(path))
                        .expect("Error forwarding loaded data!");
                    waker.wake();
                });
                self.get_mut().status = LoadStatus::Loading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::Loading(rx) => match rx.try_recv() {
                Ok(r) => match r {
                    Ok(v) => match T::try_from((path, v)) {
                        Ok(f) => Poll::Ready(Ok(Arc::new(f))),
                        Err(_e) => {
                            Poll::Ready(Err(Arc::new(Error::new(ErrorKind::InvalidData, ""))))
                        }
                    },
                    Err(e) => Poll::Ready(Err(Arc::new(e))),
                },
                Err(TryRecvError::Empty) => {
                    self.waker.register(cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(Arc::new(Error::new(ErrorKind::BrokenPipe, e)))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileLoadFuture;
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
    fn it_works() {
        let pool = Arc::new(ThreadPoolBuilder::new().create().unwrap());
        let path = PathBuf::new().join("benches/benchfiles/s01");
        let l = FileLoadFuture::<LoadedFile>::new(&path, pool);
        async_std::task::block_on(async {
            let f = l.await.unwrap();
            println!("{:?}", f);
        })
    }
}
