extern crate imageproc;
extern crate image;
extern crate yaml_rust;

use std::collections::LinkedList;
use std::io::Read;

fn main() {

    let i = load_parsing_info(String::from("data/kela_form.yaml"));
    println!("{:?}", i);
    
    let img = image::open(String::from("data/kela_form.png")).unwrap().to_luma();
    println!("Blurring image");
    let img = imageproc::filter::gaussian_blur_f32(&img, 0.5);
    img.save("blurred.png").unwrap();
    println!("Applying threshold");
    let threshed = imageproc::contrast::adaptive_threshold(&img, 14);
    threshed.save("threshed.png").unwrap();

    println!("Flooding");
    let flooded = flood_breadth_first(threshed, 100, 100, &image::Luma([121]));
    
    flooded.save("flooded.png").unwrap();

}

fn polygon_ramer_douglas_peucker(path: Vec<(i64, i64)>, tolerance: i64) -> Vec<(i64, i64)> {
    let mut widest = 0;
    let mut start_index = 0;
    for i in 0..path.len() {
        let point = path[0];
        for j in i + 1 .. path.len() {
            let distance = distance2(point, path[j]);
            if distance > widest {
                let start_index = i;
                let widest = distance;
            }
        }
    }
    let mut v = Vec::new();
    v.extend_from_slice(&path[start_index..path.len()]);
    v.extend_from_slice(&path[0..start_index]);
    ramer_douglas_peucker(v, tolerance)
}

fn ramer_douglas_peucker(path: Vec<(i64, i64)>, tolerance: i64) -> Vec<(i64, i64)> {

    let max_distance = 0;
    let index = 0;
    let end = path.len() - 1;

    for i in 1..(end - 1) {
        let d = perpendicular_distance(path[i], (path[0], path[end]));
        if d > max_distance {
            let index = i;
            let max_distance = d;
        }
    }

    if max_distance > tolerance {
        let mut recursive_results1 = ramer_douglas_peucker(path[0..index].to_vec(), tolerance);
        let mut recursive_results2 = ramer_douglas_peucker(path[index..end].to_vec(), tolerance);
        recursive_results1.pop().unwrap();
        recursive_results1.append(&mut recursive_results2);
        recursive_results1
    } else {
        vec![path[0], path[end]]
    }
}

fn perpendicular_distance(point: (i64, i64), line: ((i64, i64), (i64, i64))) -> i64 {
    let (x_0, y_0) = point;
    let ((x_1, y_1), (x_2, y_2)) = line;
    ((y_2 - y_1) * x_0 - (x_2 - x_1) * y_0 + x_2 * y_1 - y_2 * x_1)^2 / ((y_2 - y_1)^2 + (x_2 - x_1)^2)
}

fn distance2(point: (i64, i64), other: (i64, i64)) -> i64 {
    let (x, y) = point;
    let (u, v) = other;
    (u - x)^2 + (v - y)^2
}

#[derive(Debug)]
struct FormParsingInfo {
    questions: Vec<Question>,
    image_width: i64,
    image_height: i64,
}

#[derive(Debug)]
struct Question {
    wording: String,
    options: Vec<OptionBox>,
}

#[derive(Debug)]
struct OptionBox {
    topleft: (i64, i64),
    bottomright: (i64, i64),
}

fn load_parsing_info(path: String) -> FormParsingInfo {
    let mut s = String::new();
    std::fs::File::open(path).unwrap().read_to_string(&mut s).unwrap();
    let docs = yaml_rust::YamlLoader::load_from_str(&s).unwrap();
    let spec = &docs[0];
    let mut questions = Vec::new();
    for question in spec["questions"].clone() {
        let mut optionboxes = Vec::new();
        let wording = question["wording"].clone();
        for optionbox in question["options"].clone() {
            optionboxes.push(OptionBox { topleft: (optionbox["topleft"]["x"].as_i64().unwrap(), optionbox["topleft"]["y"].as_i64().unwrap()),
                                         bottomright: (optionbox["bottomright"]["x"].as_i64().unwrap(), optionbox["bottomright"]["y"].as_i64().unwrap())
                                       }
                            )
        };
        questions.push(Question { wording: wording.as_str().unwrap().to_string(), options: optionboxes });
    };
    FormParsingInfo { questions, image_width: spec["image"]["size"]["width"].as_i64().unwrap(), image_height: spec["image"]["size"]["height"].as_i64().unwrap() }
}


fn flood_breadth_first(img: image::GrayImage, x: i64, y: i64, replacement_color: &image::Luma<u8>) -> image::GrayImage {
    let target_color = img.get_pixel(x as u32, y as u32).clone();
    let mut queue = LinkedList::new();
    let (w, h) = img.dimensions();
    let mut flooded = img.clone();
    queue.push_back((x, y));
    while !queue.is_empty() {
        let (x, y) = queue.pop_front().unwrap();
        println!("{}, {}", x, y);
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
