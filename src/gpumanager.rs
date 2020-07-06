use crate::{
    gpuloader::{GpuLoadFuture, },
    imagedata::ImageData,
    Identifier, LoadStatus,
};

use futures::executor::ThreadPool;
use futures::{future::Shared, FutureExt};
use std::{collections::HashMap, sync::Arc, task::Poll};
use wgpu::{Device, Queue ,Texture};

#[allow(unused)]
pub struct AsyncGpuManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pool: Arc<ThreadPool>,
    loading: HashMap<Identifier, Shared<GpuLoadFuture>>,
    cache: HashMap<Identifier, Arc<Texture>>,
}

impl AsyncGpuManager {
    #[allow(unused)]
    pub fn new(pool: Arc<ThreadPool>, device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            device,
            queue,
            pool,
            loading: HashMap::new(),
            cache: HashMap::new(),
        }
    }
    #[allow(unused)]
    pub async fn load(&mut self, id: &Identifier, img: Arc<ImageData>) {
        if !self.cache.contains_key(&id) && !self.loading.contains_key(&id) {
            let mut f = GpuLoadFuture::new(
                img,
                self.device.clone(),
                self.queue.clone(),
                self.pool.clone(),
            )
            .shared();
            futures::poll!(&mut f);
            self.loading.insert(id.clone(), f);
        }
    }
    #[allow(unused)]
    pub async fn get(&mut self, id: &Identifier) -> LoadStatus<Texture, GpuLoadFuture> {
        if let Some(f) = self.loading.get_mut(id) {
            if let Poll::Ready(result) = futures::poll!(f) {
                self.loading.remove(id);
                match result {
                    Ok(t) => {
                        self.cache.entry(id.clone()).or_insert(t.clone());
                        LoadStatus::Loaded(t)
                    }
                    Err(e) => LoadStatus::Error(e),
                }
            } else {
                LoadStatus::Loading(self.loading.get(id).unwrap().clone())
            }
        } else if let Some(f) = self.cache.get(id) {
            LoadStatus::Loaded(f.clone())
        } else {
            LoadStatus::NotLoading
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncGpuManager;
    use crate::{imagedata::ImageData, AsyncFileManager, LoadStatus};
    use futures::executor::ThreadPoolBuilder;
    use futures::FutureExt;
    use std::{path::PathBuf, sync::Arc};
    #[test]
    fn manager() {
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

            let pool = Arc::new(ThreadPoolBuilder::new().create().unwrap());
            let path = PathBuf::new().join("small_scream.png");
            let id = path.clone().into();
            let mut imgmngr = AsyncFileManager::<ImageData>::new(pool.clone());

            let mut gpumngr = AsyncGpuManager::new(pool, arc_device, arc_queue);

            imgmngr.load(&path).await;
            match imgmngr.get(&path).await {
                LoadStatus::Loading(img_future) => {
                    img_future
                        .then(|img| async { gpumngr.load(&id, img.unwrap()).await })
                        .await
                }
                LoadStatus::Loaded(img) => gpumngr.load(&id, img).await,
                _ => panic!(),
            };
            let _texture = match gpumngr.get(&id).await {
                LoadStatus::Loading(fut) => fut.await.unwrap(),
                LoadStatus::Loaded(tex) => tex,
                _ => panic!(),
            };
            
        });
    }
}
