use super::gpuloader::{Device, Queue, Texture};
use std::{convert::TryFrom, path::PathBuf, sync::Arc};

#[derive(Debug, PartialEq)]
enum ImageFormat {
    PNG,
}
#[derive(Debug, PartialEq)]
pub struct ImageData {
    raw: Vec<u8>,
    format: ImageFormat,
}
impl ImageData {
    pub fn upload(&self, _device: Arc<Device>, _queue: Arc<Queue>) -> Texture {
        Texture {}
    }
}

impl TryFrom<(PathBuf, Vec<u8>)> for ImageData {
    fn try_from((p, raw): (PathBuf, Vec<u8>)) -> Result<Self, std::io::Error> {
        if let Some(format) = get_format_from_extension(&p) {
            Ok(ImageData { raw, format })
        } else {
            todo!() // TODO: Maybe guess format from raw?
        }
    }
    type Error = std::io::Error;
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
    use futures::executor::ThreadPoolBuilder;
    use std::{path::PathBuf, sync::Arc};
    //use std::convert::TryFrom;
    #[test]
    fn load_single_image() {
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let mut manager = AsyncFileManager::<ImageData>::new(pool);
        futures::executor::block_on(async {
            let path = PathBuf::new().join("small_scream.png");
            manager.load(&path).await;

            let file = match manager.get(&path).await {
                LoadStatus::Loaded(f) => f,
                LoadStatus::Loading(f) => f.await.unwrap(),
                _ => panic!(),
            };
            println!("{:?}", file)
            //assert_eq!(file, Arc::new(ImageData::try_from((PathBuf::new(),b"\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\n\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest\r\ntest\r\n\r\ntest\r\ntesttesttesttesttesttesttesttesttesttesttest".to_vec())).unwrap()));
        });
    }
}
