use super::imagedata::ImageData;
use crossbeam_channel::{bounded, Receiver, TryRecvError};
use futures::Future;
use futures::{executor::ThreadPool, task::AtomicWaker};
use std::{sync::Arc, task::Poll};

use wgpu::{Device, Queue, Texture};

pub struct GpuLoadFuture {
    imgdata: Arc<ImageData>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pool: Arc<ThreadPool>,
    waker: Arc<AtomicWaker>,
    status: LoadStatus,
}
#[allow(unused)]
impl GpuLoadFuture {
    pub fn new(
        imgdata: Arc<ImageData>,
        device: Arc<Device>,
        queue: Arc<Queue>,
        pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            imgdata,
            device,
            queue,
            pool,
            waker: Arc::new(AtomicWaker::new()),
            status: LoadStatus::ImageData,
        }
    }
}

enum LoadStatus {
    ImageData,
    Uploading(Receiver<Arc<Texture>>),
}

impl Future for GpuLoadFuture {
    type Output = Result<Arc<Texture>, Arc<std::io::Error>>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &self.status {
            LoadStatus::ImageData => {
                let (tx, rx) = bounded(1);
                self.waker.register(cx.waker());
                let waker = self.waker.clone();
                let imgdata = self.imgdata.clone();
                let device = self.device.clone();
                let queue = self.queue.clone();
                self.pool.spawn_ok(async move {
                    tx.send(Arc::new(imgdata.upload(device, queue)))
                        .expect("Error forwarding loaded data!");
                    waker.wake();
                });
                self.get_mut().status = LoadStatus::Uploading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::Uploading(rx) => match rx.try_recv() {
                Ok(texture) => Poll::Ready(Ok(texture)),
                Err(TryRecvError::Empty) => {
                    self.waker.register(cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(Arc::new(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    e,
                )))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GpuLoadFuture, ImageData};
    use crate::{AsyncFileManager, LoadStatus};
    use futures::executor::ThreadPoolBuilder;
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn single_image_load_and_gpu_upload() {
        async_std::task::block_on(async {
            let (needed_features, unsafe_features) =
                (wgpu::Features::empty(), wgpu::UnsafeFeatures::disallow());

            let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                    unsafe_features,
                )
                .await
                .unwrap();

            let adapter_features = adapter.features();
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: adapter_features & needed_features,
                        limits: wgpu::Limits::default(),
                        shader_validation: true,
                    },
                    None,
                )
                .await
                .unwrap();
            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);

            let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
            let mut mngr = AsyncFileManager::<ImageData>::new(pool.clone());

            let path = PathBuf::new().join("small_scream.png");
            mngr.load(&path).await;
            let img = match mngr.get(&path).await {
                LoadStatus::Loaded(t) => t,
                LoadStatus::Loading(f) => f.await.unwrap(),
                _ => panic!(),
            };

            let gpufut = GpuLoadFuture::new(img, arc_device, arc_queue, pool);
            let _tex = gpufut.await.unwrap();
            
        });
    }
}
