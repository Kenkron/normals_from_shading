use image::{self, GenericImageView, ImageReader, ImageResult};
use na::{Vector2, Vector3};

pub type RadianceMatrix = na::Matrix<f32, na::Dyn, na::U1, na::VecStorage<f32, na::Dyn, na::U1>>;

/// Container for image brightness data and lighting direction.
///
/// radiance is stored as an n x 1 matrix of brightness, where
/// n is the pixel count.
pub struct RadianceMap {
    pub lighting_direction: Vector3<f32>,
    pub size: Vector2<usize>,
    pub radiance: RadianceMatrix
}

/// Creates a radiance map from a dynamic image,
/// with a lighting direction along the z axis.
impl From<image::DynamicImage> for RadianceMap {
    fn from(image_data: image::DynamicImage) -> Self {
        let size = Vector2::new(image_data.width() as usize, image_data.height() as usize);
        let greyscale: Vec<f32> = image_data.grayscale().pixels().map(|pixel| {
            // convert to greyscale float
            pixel.2.0[0] as f32 / 255.0
        }).collect();
        Self {
            lighting_direction: Vector3::<f32>::z(),
            size,
            radiance: RadianceMatrix::from_row_slice(&greyscale)
        }
    }
}

impl RadianceMap {
    /// Load a radiance map from a file
    pub fn load(path: &str) -> ImageResult<Self> {
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