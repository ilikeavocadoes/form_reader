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
