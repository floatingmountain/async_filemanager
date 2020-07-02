use crate::{
    gpuloader::{AsyncGpuLoader, Device, Queue, Texture},
    imagedata::ImageData,
    LoadStatus,
};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    task::Poll,
};
use futures::executor::ThreadPool;

#[allow(unused)]
pub struct AsyncGpuManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pool: Arc<ThreadPool>,
    loading: FuturesUnordered<AsyncGpuLoader>,
    paths_loading: HashSet<PathBuf>,
    cache: HashMap<PathBuf, Arc<Texture>>,
}

impl AsyncGpuManager {
    #[allow(unused)]
    pub fn new(pool: Arc<ThreadPool>, device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            device,
            queue,
            pool,
            loading: futures::stream::FuturesUnordered::new(),
            paths_loading: HashSet::new(),
            cache: HashMap::new(),
        }
    }
    #[allow(unused)]
    pub async fn load<P: AsRef<Path>>(&mut self, path: P, image_data: Arc<ImageData>) {
        if !self.cache.contains_key(path.as_ref()) {
            self.loading.push(AsyncGpuLoader::new(
                path.as_ref().to_owned(),
                image_data,
                self.device.clone(),
                self.queue.clone(),
                self.pool.clone(),
            ));
            self.paths_loading.insert(path.as_ref().to_owned());
        }
        self.update().await;
    }
    #[allow(unused)]
    pub async fn get<P: AsRef<Path>>(&mut self, path: P) -> Option<&Arc<Texture>> {
        self.update().await;
        self.cache.get(path.as_ref())
    }
    #[allow(unused)]
    pub async fn remove<P: AsRef<Path>>(&mut self, path: P) -> Option<Arc<Texture>> {
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
        while let Poll::Ready(Some((p, t))) = futures::poll!(self.loading.next()) {
            self.paths_loading.remove(&p);
            self.cache.entry(p.clone()).or_insert(Arc::new(t));
        }
    }
}

#[cfg(test)]
mod tests {

    use super::AsyncGpuManager;
    use crate::{
        gpuloader::{Device, Queue},
        imagedata::ImageData,
        AsyncFileManager, LoadStatus,
    };
    use std::{path::PathBuf, sync::Arc};
    use futures::executor::ThreadPoolBuilder;

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
    fn manage_gpu_upload() {
        let pool = Arc::new(ThreadPoolBuilder::new().create().unwrap());
        let mut imgmngr = AsyncFileManager::<ImageData>::new(pool.clone());

        let device = Arc::new(Device {});
        let queue = Arc::new(Queue {});
        let mut gpumngr = AsyncGpuManager::new(pool, device, queue);

        futures::executor::block_on(async {
            let path = PathBuf::new().join("small_scream.png");
            imgmngr.load(&path).await;
            while imgmngr.status(&path).await.eq(&LoadStatus::Loading) {}
            let img = imgmngr.get(&path).await.expect("Image not loaded!").clone();
            gpumngr.load(&path, img).await;
            while gpumngr.status(&path).await.eq(&LoadStatus::Loading) {}
            let _txt = gpumngr
                .get(&path)
                .await
                .expect("Texture not loaded!")
                .clone();
        });
    }
}
