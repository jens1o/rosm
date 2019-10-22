extern crate image;
extern crate line_drawing;
extern crate osmpbf;

mod data;
mod extractor;
mod painter;

use crate::painter::Painter;
use std::error::Error;

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
    let (nid_to_node_data, wid_to_way_data, _) =
        extractor::extract_data_from_filepath(String::from("regbez-karlsruhe.osm.pbf"), false)?;

    let mut painter = painter::PngPainter::default();
    let file_name = painter.paint(IMAGE_RESOLUTION, nid_to_node_data, wid_to_way_data);

    println!("Saved image successfully to {}.", file_name);

    Ok(())
}
