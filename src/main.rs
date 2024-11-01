use std::str::FromStr;

use albedo_utils::generate_albedo;
use image::{DynamicImage, ImageReader, RgbImage};
use na::{DMatrix, Vector2, Vector3};
extern crate nalgebra as na;

mod radiance_map;
use normal_utils::NormalMatrix;
use radiance_map::*;
mod normal_utils;
mod albedo_utils;

/// Find linear least squares solution to Ax = b
/// This will return None for an underconstrained system.
pub fn least_squares(a: &NormalMatrix, b: &RadianceMatrix) -> Option<Vector3<f32>> {
    let a_transpose = a.transpose();
    let ata = &a_transpose * a;
    let atb = &a_transpose * b;

    let inv_ata = ata.try_inverse()?;
    Some(inv_ata * atb)
}

/// Estimating a lighting direction by finding the least squares solution
/// for (light_direction) of (normals)(light_directions) = (brightness_values)
/// This is based on phong diffuse shading.
///
/// The normal matrix must be an n x 3 matrix where n is the pixel count, and
/// each row holds the xyz values of the normal. The radiance vector is an
/// n x 1 matrix holding brightness data for each pixel.
pub fn generate_lighting_direction(
    normal_matrix: &NormalMatrix,
    radiance_vector: &RadianceMatrix)
-> Vector3<f32> {
    // least squares solution for normal * light_direction = radiance;
    let light_direction =
        least_squares(&normal_matrix, &radiance_vector)
            .expect("Could not find least squares for lighting direction")
            .normalize();

    // return as vec3
    Vector3::<f32>::from_column_slice(light_direction.as_slice())
}

/// Using a set of radiance maps, including brightness and
/// light direction, attempts to estimate the normal direction
/// of each pixel by finding the least squares solution
/// for (normals) of (light_directions)(normals) = (brightness_values).
/// This is based on phong diffuse shading.
pub fn generate_normals(radiance_maps: &[RadianceMap]) -> NormalMatrix {
    // perform a least squares for each pixel
    let normals: Vec<f32> = (0..radiance_maps[0].size.product()).map(|pixel| {
        let mut light_directions: Vec<f32> = Vec::new();
        let mut radiances: Vec<f32> = Vec::new();
        for radiance_map in radiance_maps {
            light_directions.extend_from_slice(radiance_map.lighting_direction.as_slice());
            radiances.push(radiance_map.radiance[pixel]);
        }
        let light_directions = NormalMatrix::from_row_slice(&light_directions);
        let radiances = RadianceMatrix::from_row_slice(&radiances);
        let least_squares_normal = least_squares(
            &light_directions,
            &radiances);
        Vec::from(least_squares_normal.expect("Could not find least squares for normal map").normalize().as_slice())
    }).flatten().collect();

    NormalMatrix::from_row_slice(&normals)
}

pub fn generate_normal_map(images: &[DynamicImage]) -> Result<DynamicImage, String> {
    if images.len() == 0 {
        return Err("No images provided".to_string());
    }
    let size =
        Vector2::new(images[0].width() as usize, images[0].height() as usize);

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
                    size[0].max(size[1]) as f32)
                    .normalize()
                    .as_slice()
            );
        }
    }

    let mut normal_matrix =
        NormalMatrix::from_row_slice(&initial_normal_map);

    for _ in 0..4 {
        // Generate new radiance maps
        for radiance_map in &mut radiance_maps {
            let est_light_direction = generate_lighting_direction(&normal_matrix, &radiance_map.radiance);
            radiance_map.lighting_direction = est_light_direction;
        }
        // Generate new normal maps
        let est_normal_map = generate_normals(&radiance_maps);
        // Reorient the normal map to face towards the camera
        let new_normal_map = normal_utils::reorient_normals(&est_normal_map);
        normal_matrix = new_normal_map;
    }

    for radiance_map in &radiance_maps {
        println!("Est light direction: {}" , radiance_map.lighting_direction);
    }

    // Flatten normal map
    let mut flattened_normals = normal_matrix;
    for _ in 0..10 {
        flattened_normals = normal_utils::corner_flatten(&flattened_normals, &size);
        // Reorient the normal map to face towards the camera
        flattened_normals = normal_utils::reorient_normals(&flattened_normals);
    }

    // Write flattened normal map
    let normal_bytes: Vec<u8> = flattened_normals.transpose().iter().map(|channel| {
        (channel * 128.0 + 128.0) as u8
    }).collect();
    let normal_output = match RgbImage::from_vec(size[0] as u32, size[1] as u32, normal_bytes) {
        None => Err("Normal output wasn't the right size".to_string()),
        Some(x) => Ok(x)
    }?;
    Ok(normal_output.into())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut images = Vec::<DynamicImage>::new();

    // Load images
    for path in &args[1..] {
        let image =
            ImageReader::open(path)
            .expect(&format!("Could not open image: {}", path))
            .decode()
            .expect(&format!("Could not decode image: {}", path));
        images.push(image);
    }

    // Validate images
    if images.len() == 0 {
        println!("No images provided");
        return;
    }

    let get_size =
        |image: &DynamicImage|
            Vector2::new(image.width() as usize, image.height() as usize);
    let size = get_size(&images[0]);
    for image in &images {
        if get_size(image) != size {
            println!("Images have different sizes");
            println!("{}, {}", size, get_size(image));
            return;
        }
    }

    // Generate albedo
    let albedo = generate_albedo(&images)
        .expect("Error generating albedo");
    albedo.save_with_format("albedo.png", image::ImageFormat::Png)
        .expect("Error saving albedo");

    // Generate normal map
    let normal_map = match generate_normal_map(&images) {
        Err(err) => return println!("{}", err),
        Ok(x) => x
    };
    normal_map.save_with_format("normal_map.png", image::ImageFormat::Png)
        .expect("Error writing normal map");
}
