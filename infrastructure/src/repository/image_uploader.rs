use async_trait::async_trait;
use common::types::BoxError;
use config::CONFIG;
use domain::{
    model::image_uploader::{ImageResizeConfig, ImageResizeInfo},
    repository::image_uploader::ImageUploaderRepository,
};
use image::{
    ColorType, DynamicImage, GenericImageView, ImageReader, codecs::jpeg::JpegEncoder,
    imageops::FilterType,
};
use std::path::Path;
use uuid58;

#[allow(dead_code)]

pub struct ImageUploaderRepositoryImpl {
    output_dir: String,
    url_root: String,
    config: ImageResizeConfig,
}

impl ImageUploaderRepositoryImpl {
    pub fn new() -> Self {
        let config = ImageResizeConfig {
            default: ImageResizeInfo {
                size: CONFIG.image.default.size,
                quality: CONFIG.image.default.quality,
            },
            thumb: ImageResizeInfo {
                size: CONFIG.image.thumb.size,
                quality: CONFIG.image.thumb.quality,
            },
        };
        let output_dir = CONFIG.image.output_dir.clone();
        let url_root = CONFIG.image.url_root.clone();
        Self {
            output_dir,
            url_root,
            config,
        }
    }

    fn resize_with_aspect(img: &DynamicImage, target: &ImageResizeInfo) -> DynamicImage {
        let (width, height) = img.dimensions();
        let scale = (target.size as f32 / width as f32)
            .min(target.size as f32 / height as f32)
            .min(1.0);

        let new_width = (width as f32 * scale).round() as u32;
        let new_height = (height as f32 * scale).round() as u32;

        img.resize_exact(new_width, new_height, FilterType::Lanczos3)
    }
}

#[async_trait]
impl ImageUploaderRepository for ImageUploaderRepositoryImpl {
    async fn save(&self, data: &[u8]) -> Result<(String, String), BoxError> {
        tracing::debug!("start repository save");

        let config = self.config.clone();
        let data = data.to_vec();

        // CPU バウンド部分は spawn_blocking
        let (id, yyyymm, main_bytes, thumb_bytes) = tokio::task::spawn_blocking(
            move || -> Result<(String, String, Vec<u8>, Vec<u8>), BoxError> {
                let id = uuid58::new();
                let now = chrono::Utc::now();
                let yyyymm = now.format("%Y%m").to_string();

                let img = ImageReader::new(std::io::Cursor::new(data))
                    .with_guessed_format()?
                    .decode()?;

                // main
                let main_resized = Self::resize_with_aspect(&img, &config.default).to_rgb8();
                let mut main_bytes = Vec::new();
                JpegEncoder::new_with_quality(&mut main_bytes, config.default.quality).encode(
                    &main_resized,
                    main_resized.width(),
                    main_resized.height(),
                    ColorType::Rgb8.into(),
                )?;

                // thumb
                let thumb_resized = Self::resize_with_aspect(&img, &config.thumb).to_rgb8();
                let mut thumb_bytes = Vec::new();
                JpegEncoder::new_with_quality(&mut thumb_bytes, config.thumb.quality).encode(
                    &thumb_resized,
                    thumb_resized.width(),
                    thumb_resized.height(),
                    image::ColorType::Rgb8.into(),
                )?;

                Ok((id, yyyymm, main_bytes, thumb_bytes))
            },
        )
        .await??;

        // I/O バウンド部分は tokio::fs
        let dir = Path::new(&self.output_dir).join(&yyyymm);
        tokio::fs::create_dir_all(&dir).await?;

        let main_filename = format!("{}.jpg", id);
        let thumb_filename = format!("{}-thumb.jpg", id);

        let main_path = dir.join(&main_filename);
        let thumb_path = dir.join(&thumb_filename);

        tokio::fs::write(&main_path, &main_bytes).await?;
        tokio::fs::write(&thumb_path, &thumb_bytes).await?;

        let default_url = format!("{}/{}/{}", self.url_root, yyyymm, main_filename);
        let thumb_url = format!("{}/{}/{}", self.url_root, yyyymm, thumb_filename);

        Ok((default_url, thumb_url))
    }
}
