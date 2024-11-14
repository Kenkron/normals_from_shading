pub mod albedo_utils;
pub mod normal_utils;
pub mod radiance_map;

use image::{DynamicImage, RgbImage};
use na::{Vector2, Vector3};
extern crate nalgebra as na;

use normal_utils::*;
use radiance_map::*;

pub fn generate_normal_map(images: &[DynamicImage]) -> Result<DynamicImage, String> {
    if images.is_empty() {
        return Err("No images provided".to_string());
    }
    let size = Vector2::new(images[0].width() as usize, images[0].height() as usize);

    // Initialize maps
    let mut radiance_maps = Vec::<RadianceMap>::new();
    for image in images {
        radiance_maps.push(RadianceMap::from(image.to_owned()));
    }

    let mut initial_normal_map = Vec::<f32>::new();
    for y in 0..size[1] {
        for x in 0..size[0] {
            initial_normal_map.extend_from_slice(
                Vector3::new(
                    x as f32 - size[0] as f32 / 2.0,
                    y as f32 - size[1] as f32 / 2.0,
                    size[0].max(size[1]) as f32,
                )
                .normalize()
                .as_slice(),
            );
        }
    }

    let mut normal_matrix = NormalMatrix::from_row_slice(&initial_normal_map);

    for _ in 0..4 {
        // Generate new radiance maps
        for radiance_map in &mut radiance_maps {
            let est_light_direction =
                generate_lighting_direction(&normal_matrix, &radiance_map.radiance);
            radiance_map.lighting_direction = est_light_direction;
        }
        // Generate new normal maps
        let est_normal_map = generate_normals(&radiance_maps);
        // Reorient the normal map to face towards the camera
        let new_normal_map = normal_utils::reorient_normals(&est_normal_map);
        normal_matrix = new_normal_map;
    }

    for radiance_map in &radiance_maps {
        println!("Est light direction: {}", radiance_map.lighting_direction);
    }

    // Flatten normal map
    let mut flattened_normals = normal_matrix;
    for _ in 0..10 {
        flattened_normals = normal_utils::corner_flatten(&flattened_normals, &size);
        // Reorient the normal map to face towards the camera
        flattened_normals = normal_utils::reorient_normals(&flattened_normals);
    }

    // Write flattened normal map
    let normal_bytes: Vec<u8> = flattened_normals
        .transpose()
        .iter()
        .map(|channel| (channel * 128.0 + 128.0) as u8)
        .collect();
    let normal_output = match RgbImage::from_vec(size[0] as u32, size[1] as u32, normal_bytes) {
        None => Err("Normal output wasn't the right size".to_string()),
        Some(x) => Ok(x),
    }?;
    Ok(normal_output.into())
}

/// Attempts to generate an albedo map by averaging and
/// flattening a slice of images.
pub fn generate_albedo(images: &[DynamicImage]) -> Option<DynamicImage> {
    let average_image = albedo_utils::average(images)?;
    let mut flattened_average = average_image;
    for _ in 0..10 {
        flattened_average = albedo_utils::corner_weight_flatten(&flattened_average);
    }
    Some(flattened_average)
}
