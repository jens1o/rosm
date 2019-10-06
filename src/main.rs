extern crate image;
extern crate line_drawing;
extern crate osmpbf;
extern crate rand;

mod data;
mod extractor;

use std::cmp;
use std::error::Error;
use std::fs;
use std::time;
use std::time::Instant;

const IMAGE_RESOLUTION: f64 = 10000.0;
const IMAGE_PART_SIZE: u32 = 1024;

const WATER_COLOR: image::Rgb<u8> = image::Rgb([170u8, 211u8, 223u8]);
const HIGHWAY_COLOR: image::Rgb<u8> = image::Rgb([249u8, 178u8, 156u8]);
const NORMAL_COLOR: image::Rgb<u8> = image::Rgb([255u8, 255u8, 255u8]);
const RAILWAY_COLOR: image::Rgb<u8> = image::Rgb([146u8, 205u8, 0u8]);
const BG_COLOR: image::Rgb<u8> = image::Rgb([0u8, 0u8, 0u8]);

trait Zero {
    fn zero() -> Self;
}

impl Zero for u32 {
    fn zero() -> u32 {
        0
    }
}

fn round_up_to<T>(int: T, target: T) -> T
where
    T: std::cmp::PartialEq
        + std::ops::Sub<Output = T>
        + std::ops::Rem<Output = T>
        + std::ops::Add<Output = T>
        + Zero
        + Copy,
{
    if int % target == T::zero() {
        return int;
    }

    return (target - int % target) + int;
}

fn main() -> Result<(), Box<dyn Error>> {
    let start_instant = Instant::now();
    let (nid_to_node_data, wid_to_way_data, _) =
        extractor::extract_data_from_filepath(String::from("regbez-karlsruhe.osm.pbf"), false)?;
    println!("Pre-processing took {:.2?}.", start_instant.elapsed());

    let mut pixels = Vec::new();

    for way in wid_to_way_data.values() {
        let nodes = way
            .refs
            .iter()
            .map(|nid| nid_to_node_data.get(nid).unwrap())
            .collect::<Vec<_>>();

        for node_list in nodes[..].windows(2) {
            if let [node_a, node_b] = node_list {
                for (x, y) in line_drawing::Midpoint::<f64, i64>::new(
                    (node_a.lat * IMAGE_RESOLUTION, node_a.lon * IMAGE_RESOLUTION),
                    (node_b.lat * IMAGE_RESOLUTION, node_b.lon * IMAGE_RESOLUTION),
                )
                .map(|(x, y)| (x as u32, y as u32))
                {
                    pixels.push((node_a.nid, x, y));

                    // make the line a bit thicker
                    pixels.push((node_a.nid, x + 1, y + 1));
                    pixels.push((node_a.nid, x + 1, y - 1));
                    pixels.push((node_a.nid, x - 1, y + 1));
                    pixels.push((node_a.nid, x - 1, y - 1));
                }
            } else {
                panic!("Windows iterator does not deliver expected size!");
            }
        }
    }

    println!("Finished pixelating, drawing on canvas.");

    let pixel_sample = pixels
        .iter()
        .next()
        .expect("At least one pixel need to be drawn!");

    let mut min_x = pixel_sample.1;
    let mut max_x = pixel_sample.1;
    let mut min_y = pixel_sample.2;
    let mut max_y = pixel_sample.2;

    for (_, x, y) in pixels.iter() {
        min_x = cmp::min(*x, min_x);
        max_x = cmp::max(*x, max_x);

        min_y = cmp::min(*y, min_y);
        max_y = cmp::max(*y, max_y);
    }

    dbg!(min_x, max_x);
    dbg!(min_y, max_y);

    let image_width = round_up_to((max_x - min_x) as u32 + 1, IMAGE_PART_SIZE);
    let image_height = round_up_to((max_y - min_y) as u32 + 1, IMAGE_PART_SIZE);
    let image_pixels = image_width * image_height;

    dbg!(image_width, image_height, image_pixels);

    // order is changed to account for rotating by 270 degrees
    let mut image = image::ImageBuffer::new(image_height, image_width);

    for (nid, pixel_x, pixel_y) in pixels
        .iter()
        .map(|(nid, pixel_x, pixel_y)| (nid, pixel_x - min_x, pixel_y - min_y))
        // rotate by 270 degress
        .map(|(nid, pixel_x, pixel_y)| (nid, pixel_y, image_width - 1 - pixel_x))
    {
        let pixel = image.get_pixel(pixel_x as u32, pixel_y as u32);

        if pixel != &BG_COLOR && pixel != &NORMAL_COLOR {
            continue;
        }

        let way_data = nid_to_node_data
            .get(nid)
            .and_then(|node_data| node_data.way)
            .and_then(|wid| wid_to_way_data.get(&wid));

        image.put_pixel(
            pixel_x as u32,
            pixel_y as u32,
            // TODO: Mark cycleways
            if way_data.map(|way| way.is_waterway()).unwrap_or(false) {
                WATER_COLOR
            } else if way_data.map(|way| way.is_highway()).unwrap_or(false) {
                HIGHWAY_COLOR
            } else if way_data.map(|way| way.is_railway()).unwrap_or(false) {
                RAILWAY_COLOR
            } else {
                NORMAL_COLOR
            },
        );
    }

    println!("Finished drawing, saving now.");

    // attempt to save memory by dropping any meta-data (as it is useless now)
    drop(nid_to_node_data);
    drop(wid_to_way_data);

    image
        .save(format!(
            "test-{}.png",
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    println!("Saved image successfully, cropping tinier pictures.");

    // create folder if necessary
    fs::create_dir_all("tiles/")?;

    for i in 0..(image_height / IMAGE_PART_SIZE) {
        for j in 0..(image_width / IMAGE_PART_SIZE) {
            let x_pos = i * IMAGE_PART_SIZE;
            let y_pos = j * IMAGE_PART_SIZE;

            let sub_image =
                image::imageops::crop(&mut image, x_pos, y_pos, IMAGE_PART_SIZE, IMAGE_PART_SIZE);

            sub_image
                .to_image()
                .save(format!("tiles/part-{}-{}.png", i, j))
                .unwrap();

            drop(sub_image);
        }
    }

    Ok(())
}
