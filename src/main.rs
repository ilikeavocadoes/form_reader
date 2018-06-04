extern crate imageproc;
extern crate image;

use std::collections::LinkedList;

fn main() {
    
    let img = image::open("data/kela_form.png").unwrap().to_luma();
    let img = imageproc::filter::gaussian_blur_f32(&img, 1.0);
    img.save("blurred.png").unwrap();
    let threshed = imageproc::contrast::adaptive_threshold(&img, 14);
    threshed.save("threshed.png").unwrap();
    let threshed = flood_breadth_first(threshed, 121, 121, &image::Luma([0]));
    
    threshed.save("flooded.png").unwrap();
}

fn flood_breadth_first(img: image::GrayImage, x: i64, y: i64, replacement_color: &image::Luma<u8>) -> image::GrayImage {
    let target_color = img.get_pixel(x as u32, y as u32).clone();
    let mut queue = LinkedList::new();
    let (w, h) = img.dimensions();
    let mut flooded = img.clone();
    queue.push_back((x, y));
    while !queue.is_empty() {
        let (x, y) = queue.pop_front().unwrap();
        if (x as u32) < w && (y as u32) < h && x >= 0 && y >= 0 {
            if flooded.get_pixel(x as u32, y as u32) == &target_color {
                flooded.put_pixel(x as u32, y as u32, replacement_color.clone());
                queue.push_back((x + 1, y));
                queue.push_back((x - 1, y));
                queue.push_back((x, y + 1));
                queue.push_back((x, y - 1));
            }
        }
    }
    flooded
}
