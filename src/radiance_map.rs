use image::{self, GenericImageView, ImageReader, ImageResult};
use na::{DMatrix, Vector2, Vector3};

pub struct RadianceMap {
    pub lighting_direction: Vector3<f32>,
    pub size: Vector2<usize>,
    pub radiance: DMatrix<f32>
}

impl From<image::DynamicImage> for RadianceMap {
    fn from(image_data: image::DynamicImage) -> Self {
        let size = Vector2::new(image_data.width() as usize, image_data.height() as usize);
        let greyscale: Vec<f32> = image_data.grayscale().pixels().map(|pixel| {
            // convert to greyscale float
            pixel.2.0[0] as f32 / 255 as f32
        }).collect();
        Self {
            lighting_direction: Vector3::<f32>::z(),
            size,
            radiance: DMatrix::from_row_slice(greyscale.len(), 1, &greyscale)}
    }
}

impl RadianceMap {
    pub fn load_rgb(path: &str) -> ImageResult<Self> {
        let image = ImageReader::open(path)?.decode()?;
        Ok(RadianceMap::from(image))
    }
    pub fn load_rgb_seed(path: &str, seed: i32) -> ImageResult<Self> {
        let image = ImageReader::open(path)?.decode()?;
        let light_direction = Vector3::new(f32::cos(seed as f32) * 0.01, f32::sin(seed as f32) * 0.01, 1.0).normalize();
        let mut result = RadianceMap::from(image);
        result.lighting_direction = light_direction;
        Ok(result)
    }
}