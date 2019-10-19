extern crate image;
extern crate line_drawing;
extern crate osmpbf;
extern crate rand;

mod data;
mod extractor;

use std::cmp;
use std::error::Error;
use std::time;
use std::time::Instant;

const IMAGE_RESOLUTION: f64 = 10000.0;
const IMAGE_PART_SIZE: u32 = 1024;

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
    let (nid_to_node_data, mut wid_to_way_data, _) =
        extractor::extract_data_from_filepath(String::from("regbez-karlsruhe.osm.pbf"), false)?;
    println!("Pre-processing took {:.2?}.", start_instant.elapsed());

    let pixelating_start = Instant::now();

    let mut pixels = Vec::new();

    for way in wid_to_way_data.values() {
        let line_width = way.line_width();

        // way is invisible
        if line_width == 0 {
            continue;
        }

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
                    // make the line a bit thicker

                    let mut distance_to_origin_pixel = line_width as u32;

                    pixels.push((node_a.nid, x, y));

                    while distance_to_origin_pixel > 1 {
                        distance_to_origin_pixel -= 1;
                        pixels.push((
                            node_a.nid,
                            x + distance_to_origin_pixel,
                            y + distance_to_origin_pixel,
                        ));
                        pixels.push((
                            node_a.nid,
                            x + distance_to_origin_pixel,
                            y - distance_to_origin_pixel,
                        ));
                        pixels.push((
                            node_a.nid,
                            x - distance_to_origin_pixel,
                            y + distance_to_origin_pixel,
                        ));
                        pixels.push((
                            node_a.nid,
                            x - distance_to_origin_pixel,
                            y - distance_to_origin_pixel,
                        ));
                    }
                }
            } else {
                panic!("Windows iterator does not deliver expected size!");
            }
        }
    }

    nid_to_node_data
        .values()
        // Filter out nodes we've already drawn through the ways
        .filter(|node_data| node_data.way.is_none())
        .for_each(|node_data| {
            for (x, y) in line_drawing::Midpoint::<f64, i64>::new(
                (
                    node_data.lat * IMAGE_RESOLUTION,
                    node_data.lon * IMAGE_RESOLUTION,
                ),
                (
                    node_data.lat * IMAGE_RESOLUTION,
                    node_data.lon * IMAGE_RESOLUTION,
                ),
            )
            .map(|(x, y)| (x as u32, y as u32))
            {
                pixels.push((node_data.nid, x, y));
            }
        });

    println!(
        "Finished pixelating in {:.2?}, drawing {} pixels on canvas.",
        pixelating_start.elapsed(),
        pixels.len()
    );

    let (_, x_sample, y_sample) = pixels
        .iter()
        .next()
        .expect("At least one pixel needs to be drawn!");

    let mut min_x = x_sample;
    let mut max_x = x_sample;
    let mut min_y = y_sample;
    let mut max_y = y_sample;

    for (_, x, y) in pixels.iter() {
        min_x = cmp::min(x, min_x);
        max_x = cmp::max(x, max_x);

        min_y = cmp::min(y, min_y);
        max_y = cmp::max(y, max_y);
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

        if pixel != &data::BG_COLOR && pixel != &data::NORMAL_COLOR {
            continue;
        }

        let node_data = nid_to_node_data.get(nid);

        let way_data = node_data
            .and_then(|node_data| node_data.way)
            .and_then(|wid| wid_to_way_data.get_mut(&wid));

        image.put_pixel(
            pixel_x as u32,
            pixel_y as u32,
            // TODO: Mark cycleways
            way_data
                .map(|way| way.draw_color())
                .unwrap_or(data::NORMAL_COLOR),
        );
    }

    println!("Finished drawing, resizing and saving now.");

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

    println!(
        "Saved image successfully{}.",
        if cfg!(create_tiles) {
            ", cropping tinier tiles"
        } else {
            ""
        }
    );

    #[cfg(create_tiles)]
    {
        use std::fs;

        // create folder if necessary
        fs::create_dir_all("tiles/")?;

        for i in 0..(image_height / IMAGE_PART_SIZE) {
            for j in 0..(image_width / IMAGE_PART_SIZE) {
                let x_pos = i * IMAGE_PART_SIZE;
                let y_pos = j * IMAGE_PART_SIZE;

                let sub_image = image::imageops::crop(
                    &mut image,
                    x_pos,
                    y_pos,
                    IMAGE_PART_SIZE,
                    IMAGE_PART_SIZE,
                );

                sub_image
                    .to_image()
                    .save(format!("tiles/part-{}-{}.png", i, j))
                    .unwrap();

                drop(sub_image);
            }
        }
    }

    Ok(())
}
