use image::{DynamicImage, GenericImage, GenericImageView, Rgba, RgbaImage};
use na::{DMatrix, Vector2};

/// Averages the pixels in a slice of images
pub fn average(images: &[DynamicImage]) -> Option<DynamicImage> {
    let size = (images.first()?.width(), images.first()?.height());
    // Sum the pixel bytes for all the images
    let mut images_sum = Vec::<f32>::new();
    for image in images {
        let image_data: Vec<f32> = image.pixels().flat_map(|pixel| {
            // convert to greyscale float
            pixel.2.0.map(|x| x as f32)
        }).collect();
        if images_sum.is_empty() {
            images_sum = image_data;
        } else {
            images_sum =
                images_sum.iter().zip(image_data)
                .map(|(a, b)| a + b)
                .collect();
        }
    }
    // Divide by the total number of images to get the average
    let images_average: Vec<u8> =
        images_sum.iter()
        .map(|x| (x / images.len() as f32) as u8)
        .collect();
    let result = RgbaImage::from_vec(size.0, size.1, images_average)?;
    Some(result.into())
}

/// Scales the brightness of an image non-uniformly
/// given the scale desired on the four corners of the
/// image, and linearly interpolating between them.
pub fn brightness_tilt(
    image_data: &DynamicImage,
    upper_left: f32,
    upper_right: f32,
    lower_left: f32,
    lower_right: f32
) -> DynamicImage {
    let mut result = image_data.clone();
    for y in 0..result.height() {
        for x in 0..result.width() {
            // Estimate relative brightness for this coordinate
            let f_x = x as f32 / result.width() as f32;
            let f_y = y as f32 / result.height() as f32;
            let relative_intensity =
                (upper_left * (1. - f_x) + upper_right * f_x) * (1. - f_y) +
                (lower_left * (1. - f_x) + lower_right * f_x) * (f_y);
            let mut pixel_data = result.get_pixel(x, y).0;
            // Scale the pixel channels with relative brightness
            // (except alpha)
            for i in 0..pixel_data.len() - 1 {
                pixel_data[i] =
                    (pixel_data[i] as f32 / relative_intensity)
                    .min(255.0) as u8;
            }
            result.put_pixel(x, y, Rgba::from(pixel_data));
        }
    }
    result
}

// Attempts to adjust for non-uniform brightness by balancing the pixels
// along the edge of the image corners, and adjusting the brightness so
// their averages match.
// Currently, the flattening only approaches average,
// it doesn't make it in a single step, so it may need repeating.
pub fn corner_flatten(image_data: &DynamicImage) -> DynamicImage {
    let size =
        Vector2::new(
            image_data.width() as usize,
            image_data.height() as usize);
    let greyscale: Vec<f32> = image_data.grayscale().pixels().map(|pixel| {
        // convert to greyscale float
        pixel.2.0[0] as f32 / 255.0
    }).collect();
    let radiance = DMatrix::from_row_slice(greyscale.len(), 1, &greyscale);

    // Get the average brightness for each corner based on the pixels along
    // the edge
    let top = radiance.rows_range(0..size[0]);
    let bottom =
        radiance.rows_range(radiance.nrows() - size[0]..radiance.nrows());
    let left = radiance.rows_with_step(0, size[1], size[0]);
    let right = radiance.rows_with_step(size[0] - 1, size[1]-1, size[0]);
    let upper_left =
        top.rows_range(0..top.nrows()/2).row_sum() +
        left.rows_range(0..left.nrows()/2).row_sum();
    let upper_left = 2.0 * upper_left[0] / (size[0] + size[1]) as f32;
    let lower_left =
        bottom.rows_range(0..bottom.nrows()/2).row_sum() +
        left.rows_range(left.nrows()/2..left.nrows()).row_sum();
    let lower_left = 2.0 * lower_left[0] / (size[0] + size[1]) as f32;
    let upper_right =
        top.rows_range(top.nrows()/2..top.nrows()).row_sum() +
        right.rows_range(0..left.nrows()/2).row_sum();
    let upper_right = 2.0 * upper_right[0] / (size[0] + size[1]) as f32;
    let lower_right =
        bottom.rows_range(bottom.nrows()/2..bottom.nrows()).row_sum() +
        right.rows_range(right.nrows()/2..right.nrows()).row_sum();
    let lower_right = 2.0 * lower_right[0] / (size[0] + size[1]) as f32;

    let average_intensity =
        (upper_left + upper_right + lower_left + lower_right) / 4.;

    let (relative_ul, relative_ur, relative_ll, right_lr) = (
        (upper_left / average_intensity).powi(2),
        (upper_right / average_intensity).powi(2),
        (lower_left / average_intensity).powi(2),
        (lower_right / average_intensity).powi(2));

    brightness_tilt(image_data, relative_ul, relative_ur, relative_ll, right_lr)
}

// Attempts to adjust for non-uniform brightness by balancing the pixels
// around the corners of the image, with higher weigths for pixels close
// to the corner, and adjusting the brightness so their averages match.
// Currently, the flattening only approaches average,
// it doesn't make it in a single step, so it may need repeating.
pub fn corner_weight_flatten(image_data: &DynamicImage) -> DynamicImage {
    let (width, height) = (image_data.width(), image_data.height());
    let mut grayscale = image_data.grayscale();
    // upper left, upper right, lower left, lower right

    let weight_sums: Vec<_> = (0..4).map(|i| {
        let sub_image = match i {
            0 => grayscale.sub_image(0, 0, width/2, height/2),
            1 => grayscale.sub_image(width/2, 0, width/2, height/2),
            2 => grayscale.sub_image(0, height/2, width/2, height/2),
            _ => grayscale.sub_image(width/2, height/2, width/2, height/2)
        };
        let mut weight = 0.0;
        for x in 0..sub_image.width() {
            for y in 0..sub_image.height() {
                let dx =
                    if i % 2 == 0 {x}
                    else {sub_image.width() - 1 - x};
                let dy =
                    if i < 2 {y}
                    else {sub_image.height() - 1 - y};
                weight +=
                    (dx + dy) as f32 * sub_image.get_pixel(x, y).0[0] as f32;
            }
        }
        weight
    }).collect();
    let weight_total: f32 = weight_sums.iter().sum();
    let average_weight = weight_total / weight_sums.len() as f32;
    let weights: Vec<_> =
        weight_sums.iter().map(|w| w/average_weight).collect();
    brightness_tilt(image_data, weights[0], weights[1], weights[2], weights[3])
}