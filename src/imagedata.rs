use super::gpuloader::{Device, Queue, Texture};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
enum ImageFormat {
    PNG,
}
#[derive(Debug)]
pub struct ImageData {
    raw: Vec<u8>,
    format: ImageFormat,
}
impl ImageData {
    pub fn upload(&self, _device: Arc<Device>, _queue: Arc<Queue>) -> Texture {
        Texture {}
    }
}

impl From<(PathBuf, Vec<u8>)> for ImageData {
    fn from((p, raw): (PathBuf, Vec<u8>)) -> Self {
        if let Some(format) = get_format_from_extension(&p) {
            ImageData { raw, format }
        } else {
            todo!() // TODO: Maybe guess format from raw?
        }
    }
}

fn get_format_from_extension(p: &PathBuf) -> Option<ImageFormat> {
    match p.extension()?.to_str()? {
        "png" => Some(ImageFormat::PNG),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ImageData;
    use crate::{AsyncFileManager, LoadStatus};
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn load_single_image() {
        let pool = Arc::new(threadpool::ThreadPool::new(4));
        let mut mngr = AsyncFileManager::<ImageData>::new(pool);
        futures::executor::block_on(async {
            let path = PathBuf::new().join("small_scream.png");
            mngr.load(&path).await;
            while mngr.status(&path).await.eq(&LoadStatus::Loading) {}
            println!("{:?}", mngr.get(&path).await);
        });
    }
}
