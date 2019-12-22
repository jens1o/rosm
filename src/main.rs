extern crate cssparser;
extern crate image;
extern crate line_drawing;
extern crate markup5ever;
extern crate osmpbf;

mod data;
mod extractor;
mod mapcss;
mod painter;

use crate::painter::Painter;
use std::error::Error;
use std::time::Instant;

const IMAGE_RESOLUTION: f64 = 10000.0;
const IMAGE_PART_SIZE: u32 = 1024;

pub(crate) trait Zero {
    fn zero() -> Self;
}

impl Zero for u32 {
    fn zero() -> u32 {
        0
    }
}

pub(crate) fn round_up_to<T>(int: T, target: T) -> T
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
    println!("Extracting data!");

    let instant = Instant::now();
    let (nid_to_node_data, wid_to_way_data, _) =
        extractor::extract_data_from_filepath(String::from("bremen-latest.osm.pbf"), false)?;
    println!(
        "Extracting the data from the given PBF file took {:.2?}. ({} nodes, {} ways)",
        instant.elapsed(),
        nid_to_node_data.len(),
        wid_to_way_data.len()
    );

    println!("Parsing mapcss.");
    let mapcss_rules = mapcss::parse_mapcss(include_str!("../include/mapcss.css"));

    let mut painter = painter::PngPainter::default();
    let file_name = painter.paint(
        IMAGE_RESOLUTION,
        nid_to_node_data,
        wid_to_way_data,
        mapcss_rules,
    );

    println!("Saved image successfully to {}.", file_name);

    Ok(())
}
