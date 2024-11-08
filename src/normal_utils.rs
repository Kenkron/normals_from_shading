use na::{DMatrix, Matrix3, Rotation3, Vector2, Vector3};

use crate::radiance_map::*;

pub type NormalMatrix = na::Matrix<f32, na::Dyn, na::U3, na::VecStorage<f32, na::Dyn, na::U3>>;

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
        least_squares(normal_matrix, radiance_vector)
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
    let normals: Vec<f32> = (0..radiance_maps[0].size.product()).flat_map(|pixel| {
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
        Vec::from(
            least_squares_normal
                .expect("Could not find least squares for normal map")
                .normalize()
                .as_slice())
    }).collect();

    NormalMatrix::from_row_slice(&normals)
}

// Rotates normals so their average points upwards
pub fn reorient_normals(normals: &NormalMatrix) -> NormalMatrix {
    let average_normal_raw = normals.row_mean().normalize();
    let average_normal = Vector3::from_row_slice(average_normal_raw.as_slice());
    let rotation = Rotation3::rotation_between(&average_normal, &Vector3::z());

    // If the normals are already averaged, return a copy of the originals
    // TODO (edge case): this will also happen if the average normal is exactly opposite of z
    if rotation.is_none() {
        return normals.clone();
    }
    let rotation = rotation.unwrap();

    let rotation_matrix: Matrix3<f32> =rotation.into();
    let new_normals = rotation_matrix * normals.transpose();
    NormalMatrix::from_column_slice(new_normals.transpose().as_slice())
}

pub fn normal_tilt(
    normals: &NormalMatrix,
    size: &Vector2<usize>,
    upper_left: &Vector3<f32>,
    upper_right: &Vector3<f32>,
    lower_left: &Vector3<f32>,
    lower_right: &Vector3<f32>)
-> NormalMatrix {
    let i_to_xy = |i: usize| (i % size[0], i / size[0]);
    let mut result = normals.clone();
    for i in 0..result.nrows() {
        // get coordinates as a fraction of the image size
        let (i_x, i_y) = i_to_xy(i);
        let f_x = i_x as f32 / size[0] as f32;
        let f_y = i_y as f32 / size[1] as f32;
        // Estimate "flat" at this coordinate
        let alignment_vector =
            (upper_left.scale(1.0 - f_x) + upper_right.scale(f_x)).scale(1.0 - f_y) +
            (lower_left.scale(1.0 - f_x) + lower_right.scale(f_x)).scale(f_y);
        let alignment_vector =
            Vector3::from_column_slice(alignment_vector.as_slice())
            .normalize();
        // Rotate to flatten
        let rotation =
            Rotation3::rotation_between(&alignment_vector, &Vector3::z())
            .unwrap_or(Rotation3::identity());
        let rotation_matrix_3: Matrix3<f32> = rotation.into();
        let rotation_matrix =
            DMatrix::from_column_slice(3, 3, rotation_matrix_3.as_slice());
        let aligned_normal =
            (rotation_matrix * result.row(i).transpose()).transpose().normalize();
        result.set_row(i, &aligned_normal.row(0));
    }
    result
}

// Finds the average normal corners of the edges of the image.
// Normals are then rotated with linear interpolation between the corners.
// Note that this assumes edge normals face forwards.
// If the edges match their opposites, but are not necessarily flat,
// reorient normals may be used to attempt to compensate
pub fn corner_flatten(normals: &NormalMatrix, size: &Vector2<usize>) -> NormalMatrix {
    let top = normals.rows_range(0..size[0]);
    let bottom = normals.rows_range(normals.nrows() - size[0]..normals.nrows());
    let left = normals.rows_with_step(0, size[1], size[0]);
    let right = normals.rows_with_step(size[0] - 1, size[1]-1, size[0]);
    let upper_left =
        top.rows_range(0..top.nrows()/2).row_sum() +
        left.rows_range(0..left.nrows()/2).row_sum();
    let upper_left = upper_left.normalize().transpose();
    let lower_left =
        bottom.rows_range(0..bottom.nrows()/2).row_sum() +
        left.rows_range(left.nrows()/2..left.nrows()).row_sum();
    let lower_left = lower_left.normalize().transpose();
    let upper_right =
        top.rows_range(top.nrows()/2..top.nrows()).row_sum() +
        right.rows_range(0..left.nrows()/2).row_sum();
    let upper_right = upper_right.normalize().transpose();
    let lower_right =
        bottom.rows_range(bottom.nrows()/2..bottom.nrows()).row_sum() +
        right.rows_range(right.nrows()/2..right.nrows()).row_sum();
    let lower_right = lower_right.normalize().transpose();
    normal_tilt(normals, size, &upper_left, &upper_right, &lower_left, &lower_right)
}

// Finds the average normal of the top, bottom, left, and right of the image.
// Normals are then rotated with linear interpolation between the edges.
// Note that this assumes edge normals face forwards.
// If the edges match their opposites, but are not necessarily flat,
// reorient normals may be used to attempt to compensate
pub fn edge_flatten(normals: &NormalMatrix, size: &Vector2<usize>) -> NormalMatrix {
    let i_to_xy = |i: usize| (i % size[0], i / size[0]);
    let top = normals.view_range(0..size[0], 0..3)
        .row_mean()
        .normalize();
    let bottom = normals.view_range(normals.nrows() - size[0]..normals.nrows(), 0..3)
        .row_mean()
        .normalize();
    let left = normals.rows_with_step(0, size[1], size[0])
        .row_mean()
        .normalize();
    let right = normals.rows_with_step(size[0] - 1, size[1]-1, size[0])
        .row_mean()
        .normalize();
    let mut result = normals.clone();
    for i in 0..result.nrows() {
        // get coordinates as a fraction of the image size
        let (i_x, i_y) = i_to_xy(i);
        let f_x = i_x as f32 / size[0] as f32;
        let f_y = i_y as f32 / size[1] as f32;
        // Estimate "flat" at this coordinate
        let alignment_vector =
            left.scale(1.0 - f_x) + right.scale(f_x) +
            top.scale(1.0 - f_y) + bottom.scale(f_x);
        let alignment_vector =
            Vector3::from_column_slice(alignment_vector.as_slice())
            .normalize();
        // Rotate to flatten
        let rotation =
            Rotation3::rotation_between(&alignment_vector, &Vector3::z())
            .unwrap_or(Rotation3::identity());
        let rotation_matrix_3: Matrix3<f32> = rotation.into();
        let rotation_matrix =
            DMatrix::from_column_slice(3, 3, rotation_matrix_3.as_slice());
        let aligned_normal =
            (rotation_matrix * result.row(i).transpose()).transpose();
        result.set_row(i, &aligned_normal.row(0));
    }
    result
}