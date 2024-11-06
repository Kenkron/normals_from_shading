use image::{DynamicImage, ImageReader};
use normals_from_shading::*;

#[test]
fn normal_map_generation() {
    let mut images = Vec::<DynamicImage>::new();

    let args = vec![
        "sample_input/tile_512_a.jpg",
        "sample_input/tile_512_b.jpg",
        "sample_input/tile_512_c.jpg"
    ];

    // Load images
    for path in &args[1..] {
        let image =
            ImageReader::open(path)
            .expect(&format!("Could not open image: {}", path))
            .decode()
            .expect(&format!("Could not decode image: {}", path));
        images.push(image);
    }

    generate_normal_map(&images);
}

#[test]
fn albedo_generation() {
    let mut images = Vec::<DynamicImage>::new();

    let args = vec![
        "sample_input/tile_2048_a.jpg",
        "sample_input/tile_2048_b.jpg",
        "sample_input/tile_2048_c.jpg"
    ];

    // Load images
    for path in &args[1..] {
        let image =
            ImageReader::open(path)
            .expect(&format!("Could not open image: {}", path))
            .decode()
            .expect(&format!("Could not decode image: {}", path));
        images.push(image);
    }

    generate_albedo(&images);
}