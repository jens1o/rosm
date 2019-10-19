extern crate osmpbf;
extern crate svg;

mod data;
mod extractor;

use std::cmp;
use std::error::Error;
use std::time;
use std::time::Instant;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

const IMAGE_RESOLUTION: f64 = 10000.0;
const IMAGE_PART_SIZE: u32 = 1024;

// const WATER_COLOR: image::Rgb<u8> = image::Rgb([170u8, 211u8, 223u8]);
// const HIGHWAY_COLOR: image::Rgb<u8> = image::Rgb([249u8, 178u8, 156u8]);
// const NORMAL_COLOR: image::Rgb<u8> = image::Rgb([255u8, 255u8, 255u8]);
// const RAILWAY_COLOR: image::Rgb<u8> = image::Rgb([146u8, 205u8, 0u8]);
// const LONELY_NODE_COLOR: image::Rgb<u8> = image::Rgb([236u8, 78u8, 32u8]);
// const WAY_END_NODE_COLOR: image::Rgb<u8> = image::Rgb([255u8, 111u8, 255u8]);
// const BG_COLOR: image::Rgb<u8> = image::Rgb([0u8, 0u8, 0u8]);

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
    println!("Finished pixelating, drawing on canvas.");

    let mut svg_document = Document::new();

    let mut min_lat: Option<i8> = None;
    let mut min_lon: Option<i8> = None;
    let mut max_lat: Option<i8> = None;
    let mut max_lon: Option<i8> = None;

    for way_data in wid_to_way_data
        .values()
        .filter(|way| way.is_waterway())
        .take(10)
    {
        let mut drawn_way = Data::new();
        let mut is_first = true;

        for node in way_data
            .refs
            .iter()
            .map(|nid| nid_to_node_data.get(nid).unwrap())
        {
            if let None = min_lat {
                min_lat = Some(node.lat as i8);
                min_lon = Some(node.lon as i8);
            } else {
                min_lat = Some(cmp::min(min_lat.unwrap(), node.lat as i8));
                min_lon = Some(cmp::min(min_lon.unwrap(), node.lon as i8));
            }

            max_lat = cmp::max(max_lat, Some(node.lat as i8));
            max_lon = cmp::max(max_lon, Some(node.lon as i8));

            if is_first {
                drawn_way = drawn_way.move_to((node.lat, node.lon));
                is_first = false;
                continue;
            }

            drawn_way = drawn_way.line_by((node.lat, node.lon));
        }

        drawn_way = drawn_way.close();

        svg_document = svg_document.add(
            Path::new()
                .set("fill", "blue")
                .set("stroke", "blue")
                .set("stroke-width", 1)
                .set("d", drawn_way),
        );
    }

    svg_document = svg_document.set("width", "100%").set("height", "100%").set(
        "viewBox",
        (
            min_lat.expect("No node processed!"),
            min_lon.unwrap(),
            max_lat.unwrap() + 1,
            max_lon.unwrap() + 1,
        ),
    );

    svg::save("test.svg", &svg_document).unwrap();

    Ok(())
}
