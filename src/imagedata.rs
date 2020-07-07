use image::ImageFormat;
use std::{convert::TryFrom, path::PathBuf, sync::Arc};
use wgpu::{Device, Queue, Texture};

fn convert_format(i: ImageFormat) -> wgpu::TextureFormat {
    match i {
        ImageFormat::Hdr => wgpu::TextureFormat::Rgba32Float,
        ImageFormat::Png => wgpu::TextureFormat::Rgba8Unorm,
        _ => panic!(),
    }
}

#[derive(Debug, PartialEq)]
pub struct ImageData {
    name: Option<String>,
    extent: wgpu::Extent3d,
    raw: Vec<u8>,
    format: ImageFormat,
}

impl ImageData {
    pub fn upload(&self, device: Arc<Device>, queue: Arc<Queue>) -> Texture {
        let format = convert_format(self.format);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: self.extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &self.raw[..],
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: (self.raw.len() as f64 / self.extent.height as f64) as u32,
                rows_per_image: self.extent.height,
            },
            self.extent,
        );
        texture
    }
}

impl TryFrom<(PathBuf, Vec<u8>)> for ImageData {
    fn try_from((p, raw): (PathBuf, Vec<u8>)) -> Result<Self, std::io::Error> {
        if let Some(format) = get_format_from_extension(&p) {
            let image = image::load_from_memory_with_format(&raw, format)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
            let image = image.to_rgba();
            let (width, height) = image.dimensions();
            Ok(ImageData {
                name: p
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .and_then(|name| Some(String::from(name))),
                extent: wgpu::Extent3d {
                    width,
                    height,
                    depth: 1,
                },
                raw: image.into_raw(),
                format,
            })
        } else {
            todo!() // TODO: Maybe guess format from raw?
        }
    }
    type Error = std::io::Error;
}

fn get_format_from_extension(p: &PathBuf) -> Option<ImageFormat> {
    match p.extension()?.to_str()? {
        "png" => Some(ImageFormat::Png),
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

            let _ = match manager.get(&path).await {
                LoadStatus::Loaded(f) => f,
                LoadStatus::Loading(f) => f.await.unwrap(),
                _ => panic!(),
            };
        });
    }
}
