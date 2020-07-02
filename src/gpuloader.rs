use super::imagedata::ImageData;
use futures::Future;
use std::{
    path::PathBuf,
    sync::{
        mpsc::{sync_channel, Receiver},
        Arc,
    },
    task::Poll,
};
use futures::executor::ThreadPool;

pub struct Texture {}

#[allow(unused)]
pub struct Device;

#[allow(unused)]
pub struct Queue;

pub struct AsyncGpuLoader {
    path: PathBuf,
    imgdata: Arc<ImageData>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pool: Arc<ThreadPool>,
    status: LoadStatus,
}
#[allow(unused)]
impl AsyncGpuLoader {
    pub fn new(
        path: PathBuf,
        imgdata: Arc<ImageData>,
        device: Arc<Device>,
        queue: Arc<Queue>,
        pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            path,
            imgdata,
            device,
            queue,
            pool,
            status: LoadStatus::ImageData,
        }
    }
}

enum LoadStatus {
    ImageData,
    Uploading(Receiver<(PathBuf, Texture)>),
}

impl Future for AsyncGpuLoader {
    type Output = (PathBuf, Texture);
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &self.status {
            LoadStatus::ImageData => {
                let (tx, rx) = sync_channel(1);
                let w = cx.waker().clone();
                let path = self.path.clone();
                let imgdata = self.imgdata.clone();
                let device = self.device.clone();
                let queue = self.queue.clone();
                self.pool.spawn_ok(async move {
                    tx.send((path.clone(), imgdata.upload(device, queue)))
                        .expect("Error forwarding loaded data!");
                    w.wake();
                });
                self.get_mut().status = LoadStatus::Uploading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::Uploading(rx) => {
                Poll::Ready(rx.recv().expect("Could not recieve uploaded Texture!"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncGpuLoader, Device, ImageData, Queue};
    use crate::{AsyncFileManager, LoadStatus};
    use std::{path::PathBuf, sync::Arc};
    use futures::executor::ThreadPoolBuilder;

    #[test]
    fn single_image_load_and_gpu_upload() {
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let mut mngr = AsyncFileManager::<ImageData>::new(pool.clone());
        futures::executor::block_on(async {
            let path = PathBuf::new().join("small_scream.png");
            mngr.load(&path).await;
            while mngr.status(&path).await.eq(&LoadStatus::Loading) {}
            let img = mngr.get(&path).await.expect("Image not loaded!").clone();

            let device = Arc::new(Device {});
            let queue = Arc::new(Queue {});
            let gpufut = AsyncGpuLoader::new(path, img, device, queue, pool);
            let (_path, _texture) = gpufut.await;
        });
    }
}
