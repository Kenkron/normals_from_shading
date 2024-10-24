use na::{DMatrix, Matrix3, Matrix4, Rotation3, RowDVector, Vector2, Vector3};

// Rotates normals so their average points upwards
pub fn reorient_normals(normals: &DMatrix<f32>) -> DMatrix<f32> {
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
    let normals_dmat = DMatrix::from_column_slice(
        new_normals.nrows(), new_normals.ncols(), new_normals.as_slice());
    normals_dmat.transpose()
}

// Finds the average normal corners of the edges of the image.
// Normals are then rotated with linear interpolation between the corners.
// Note that this assumes edge normals face forwards.
// If the edges match their opposites, but are not necessarily flat,
// reorient normals may be used to attempt to compensate
pub fn corner_flatten(normals: &DMatrix<f32>, size: &Vector2<usize>) -> DMatrix<f32> {
    let i_to_xy = |i: usize| (i % size[0], i / size[0]);
    let top = normals.view_range(0..size[0], 0..3);
    let bottom = normals.view_range(normals.nrows() - size[0]..normals.nrows(), 0..3);
    let left = normals.rows_with_step(0, size[1], size[0]);
    let right = normals.rows_with_step(size[0] - 1, size[1]-1, size[0]);
    let upper_left =
        top.view_range(0..top.nrows()/2, 0..3).row_sum() +
        left.view_range(0..left.nrows()/2, 0..3).row_sum();
    let upper_left = upper_left.normalize();
    let lower_left =
        bottom.view_range(0..bottom.nrows()/2, 0..3).row_sum() +
        left.view_range(left.nrows()/2..left.nrows(), 0..3).row_sum();
    let lower_left = lower_left.normalize();
    let upper_right =
        top.view_range(top.nrows()/2..top.nrows(), 0..3).row_sum() +
        right.view_range(0..left.nrows()/2, 0..3).row_sum();
    let upper_right = upper_right.normalize();
    let lower_right =
        bottom.view_range(bottom.nrows()/2..bottom.nrows(), 0..3).row_sum() +
        right.view_range(right.nrows()/2..right.nrows(), 0..3).row_sum();
    let lower_right = lower_right.normalize();
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
            (rotation_matrix * &result.row(i).transpose()).transpose().normalize();
        result.set_row(i, &aligned_normal.row(0));
    }
    result
}

// Finds the average normal of the top, bottom, left, and right of the image.
// Normals are then rotated with linear interpolation between the edges.
// Note that this assumes edge normals face forwards.
// If the edges match their opposites, but are not necessarily flat,
// reorient normals may be used to attempt to compensate
pub fn edge_flatten(normals: &DMatrix<f32>, size: &Vector2<usize>) -> DMatrix<f32> {
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
            (rotation_matrix * &result.row(i).transpose()).transpose();
        result.set_row(i, &aligned_normal.row(0));
    }
    result
}