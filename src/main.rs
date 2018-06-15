extern crate imageproc;
extern crate image;
extern crate yaml_rust;

use std::collections::LinkedList;
use std::io::Read;

fn main() {

    let filename = "data/one_cell.png";

    let i = load_parsing_info(String::from("data/kela_form.yaml"));
    println!("{:?}", i);
    
    let img = image::open(String::from(filename)).unwrap().to_luma();
    println!("Blurring image");
    let img = imageproc::filter::gaussian_blur_f32(&img, 0.5);
    img.save("blurred.png").unwrap();
    println!("Applying threshold");
    let threshed = imageproc::contrast::adaptive_threshold(&img, 14);
    threshed.save("threshed.png").unwrap();

    println!("Detecting edges");
    let edges = imageproc::edges::canny(&threshed, 0.01, 50.0);
    edges.save("edges.png").unwrap();
    let a = find_contour(&edges);
    println!("{:?}", a);
    let simplified = polygon_ramer_douglas_peucker(a.iter().map(|&(x, y)| (x as f64, y as f64)).collect(), 10.0);
    println!("{:?}", simplified);

    println!("Flooding");
    let flooded = flood_breadth_first(threshed, 1, 1, &image::Luma([121]));

    flooded.save("flooded.png").unwrap();

}

fn find_contour(image: &image::GrayImage) -> Vec<(i64, i64)> {
    let (w, h) = image.dimensions();
    let (x, y) = (|w, h| {
        for i in 0..w {
            for j in 0..h {
                if image.get_pixel(i, j) == &image::Luma([255]) {
                    return (i, j);
                }
            }
        }
        (0, 0)
    })(w, h);
    let (x_out, y_out) = (x - 1, y);

    follow_contour(image, x as i64, y as i64, Direction::Right, vec![])
}

enum Direction {
    Right,
    Down,
    Left,
    Up,
}

fn follow_contour(img: &image::GrayImage, x: i64, y: i64, direction: Direction, mut contour: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    let mut to_follow = Vec::new();
    let x_first = x;
    let y_first = y;
    to_follow.push((img, x, y, direction, contour.clone()));
    while to_follow.len() != 0 {
        let (img, x, y, direction, mut contour) = to_follow.pop().unwrap();

        if (contour.len() != 0) && (contour[0] == (x, y)) {
            contour.push((x, y));
            return contour
        }
        let ((x_n, y_n), (x_on, y_on)) = match direction {
            Direction::Right => ((x, y + 1), (x - 1, y + 1)),
            Direction::Down => ((x + 1, y), (x + 1, y + 1)),
            Direction::Left => ((x, y - 1), (x + 1, y - 1)),
            Direction::Up => ((x - 1, y), (x - 1, y - 1)),
        };
        let next_pixel_right_color = img.get_pixel(x_n as u32, y_n as u32) == &image::Luma([255]);
        let next_outer_pixel_right_color = img.get_pixel(x_on as u32, y_on as u32) == &image::Luma([0]);
        if next_pixel_right_color && next_outer_pixel_right_color {
            contour.push((x, y));
            let (next_x, next_y) = match direction {
                Direction::Right => (x, y + 1),
                Direction::Down => (x + 1, y),
                Direction::Left => (x, y - 1),
                Direction::Up => (x - 1, y),
            };

            to_follow.push((img, next_x, next_y, direction, contour.clone()));

        } else {
            if !next_outer_pixel_right_color {
                let (next_direction, next_x, next_y) = match direction {
                    Direction::Right => (Direction::Up, x - 1, y + 1),
                    Direction::Down => (Direction::Right, x + 1, y + 1),
                    Direction::Left => (Direction::Down, x + 1, y - 1),
                    Direction::Up => (Direction::Left, x - 1, y - 1),
                };
                contour.push((x, y));
                
                to_follow.push((img, next_x, next_y, next_direction, contour.clone()));
            } else {
                let next_direction = match direction {
                    Direction::Right => Direction::Down,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Up,
                    Direction::Up => Direction::Right
                };
                to_follow.push((img, x, y, next_direction, contour.clone()));
            }
        }
    }
    contour
}
    
fn polygon_ramer_douglas_peucker(path: Vec<(f64, f64)>, tolerance: f64) -> Vec<(f64, f64)> {
    let mut widest = 0.0;
    let mut start_index = 0;
    for i in 0..path.len() {
        let point = path[0];
        for j in i + 1 .. path.len() {
            let distance = distance2(point, path[j]);
            if distance > widest {
                start_index = i;
                widest = distance;
            }
        }
    }
    let mut v = Vec::new();
    v.extend_from_slice(&path[start_index..path.len()]);
    v.extend_from_slice(&path[0..(start_index + 1)]);
    println!("{:?}", v);
    ramer_douglas_peucker(v, tolerance)
}

fn ramer_douglas_peucker(path: Vec<(f64, f64)>, tolerance: f64) -> Vec<(f64, f64)> {

    let mut max_distance = 0.0;
    let mut index = 0;
    let end = path.len() - 1;

    for i in 1..(end - 1) {
        let d = perpendicular_distance(path[i], (path[0], path[end]));
        if d > max_distance {
            index = i;
            max_distance = d;
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

fn perpendicular_distance(point: (f64, f64), line: ((f64, f64), (f64, f64))) -> f64 {
    let (x_0, y_0) = point;
    let ((x_1, y_1), (x_2, y_2)) = line;
    if y_2 - y_1 == 0.0 && x_2 - x_1 == 0.0 {
        distance2(point, (x_1, y_1))
    } else {
        ((y_2 - y_1) * x_0 - (x_2 - x_1) * y_0 + x_2 * y_1 - y_2 * x_1).powi(2) / ((y_2 - y_1).powi(2) + (x_2 - x_1).powi(2))
    }
}

fn distance2(point: (f64, f64), other: (f64, f64)) -> f64 {
    let (x, y) = point;
    let (u, v) = other;
    (u - x).powi(2) + (v - y).powi(2)
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
