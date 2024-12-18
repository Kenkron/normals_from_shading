use image::{DynamicImage, ImageReader};
use nalgebra::Vector2;
use normals_from_shading::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut images = Vec::<DynamicImage>::new();

    // Load images
    for path in &args[1..] {
        let image = ImageReader::open(path)
            .unwrap_or_else(|_| panic!("Could not open image: {}", path))
            .decode()
            .unwrap_or_else(|_| panic!("Could not decode image: {}", path));
        images.push(image);
    }

    // Validate images
    if images.is_empty() {
        println!("No images provided");
        return;
    }

    let get_size =
        |image: &DynamicImage| Vector2::new(image.width() as usize, image.height() as usize);
    let size = get_size(&images[0]);
    for image in &images {
        if get_size(image) != size {
            println!("Images have different sizes");
            println!("{}, {}", size, get_size(image));
            return;
        }
    }

    // Generate albedo
    let albedo = generate_albedo(&images).expect("Error generating albedo");
    albedo
        .save_with_format("albedo.png", image::ImageFormat::Png)
        .expect("Error saving albedo");

    // Generate normal map
    let normal_map = match generate_normal_map(&images) {
        Err(err) => return println!("{}", err),
        Ok(x) => x,
    };
    normal_map
        .save_with_format("normal_map.png", image::ImageFormat::Png)
        .expect("Error writing normal map");
}
