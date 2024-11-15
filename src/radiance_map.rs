use image::{self, GenericImageView, ImageReader, ImageResult, RgbImage};
use na::{Vector2, Vector3};

pub type RadianceMatrix = na::Matrix<f32, na::Dyn, na::U1, na::VecStorage<f32, na::Dyn, na::U1>>;

/// Container for image brightness data and lighting direction.
///
/// radiance is stored as an n x 1 matrix of brightness, where
/// n is the pixel count.
pub struct RadianceMap {
    pub lighting_direction: Vector3<f32>,
    pub size: Vector2<usize>,
    pub radiance: RadianceMatrix,
}

/// Creates a radiance map from a dynamic image,
/// with a lighting direction along the z axis.
impl From<image::DynamicImage> for RadianceMap {
    fn from(image_data: image::DynamicImage) -> Self {
        let size = Vector2::new(image_data.width() as usize, image_data.height() as usize);
        let greyscale: Vec<f32> = image_data
            .grayscale()
            .pixels()
            .map(|pixel| {
                // convert to greyscale float
                pixel.2 .0[0] as f32 / 255.0
            })
            .collect();
        Self {
            lighting_direction: Vector3::<f32>::z(),
            size,
            radiance: RadianceMatrix::from_row_slice(&greyscale),
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
        let light_direction = Vector3::new(
            f32::cos(seed as f32) * 0.01,
            f32::sin(seed as f32) * 0.01,
            1.0,
        )
        .normalize();
        let mut result = RadianceMap::from(image);
        result.lighting_direction = light_direction;
        Ok(result)
    }
    pub fn export(&self, path: &str) -> Result<(), String>{
        // Write flattened normal map
        let image_bytes: Vec<u8> = self.radiance
            .transpose()
            .iter()
            .flat_map(|channel| {
                vec![(channel * 256.0) as u8; 3]
            })
            .collect();
        let output_image = match RgbImage::from_vec(self.size[0] as u32, self.size[1] as u32, image_bytes) {
            None => Err("Output wasn't the right size".to_string()),
            Some(x) => Ok(x),
        }?;
        output_image
            .save(path)
            .expect("Error saving albedo");
        Ok(())
    }
}

pub fn balance_radiances(radiance_maps: &[RadianceMap]) -> Vec<RadianceMap> {
    let average_radiances = radiance_maps.iter().map(
        |radiances| radiances.radiance.mean()
    );
    let total_sum: f32 = average_radiances.clone().sum();
    let total_average = total_sum / radiance_maps.len() as f32;
    println!("total avg:{}", total_average);
    radiance_maps
        .iter()
        .zip(average_radiances)
        .map(|(radiance, local_average)| {
            println!("local avg: {}", local_average);
            RadianceMap {
                lighting_direction: radiance.lighting_direction,
                size: radiance.size,
                radiance: radiance.radiance.scale(total_average/local_average)
            }
        })
        .collect()
}
