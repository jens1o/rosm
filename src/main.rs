extern crate cssparser;
extern crate image;
extern crate line_drawing;
extern crate markup5ever;
extern crate osmpbf;
extern crate winapi;

mod data;
mod extractor;
mod mapcss;
mod painter;

use crate::painter::Painter;
use std::error::Error;
use std::time::Instant;
#[cfg(windows)]
use winapi::um::processthreadsapi::GetCurrentProcess;
#[cfg(windows)]
use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

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

pub(crate) fn print_peak_memory_usage() {
    #[cfg(windows)]
    unsafe {
        let mut pmc = std::mem::zeroed::<PROCESS_MEMORY_COUNTERS>();
        let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, cb) == 0 {
            eprintln!("fail to get memory info of process");
        }

        println!(
            "Peak memory usage: {} MB",
            (pmc.PeakWorkingSetSize as u64) / 1024 / 1024
        );
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
    print_peak_memory_usage();

    dbg!(std::mem::size_of::<crate::data::NodeData>());
    dbg!(std::mem::size_of::<crate::data::WayData>());

    println!("Extracting data!");

    let instant = Instant::now();
    let (nid_to_node_data, wid_to_way_data) =
        extractor::extract_data_from_filepath(String::from("regbez-karlsruhe.osm.pbf"))?;

    print_peak_memory_usage();
    println!(
        "Extracting the data from the given PBF file took {:.2?}. ({} nodes, {} ways)",
        instant.elapsed(),
        nid_to_node_data.len(),
        wid_to_way_data.len()
    );

    println!("Parsing mapcss.");
    let mapcss_rules = mapcss::parse_mapcss(include_str!("../include/mapcss.css"));

    println!("Now painting the picture!");
    print_peak_memory_usage();

    let instant = Instant::now();
    let mut painter = painter::PngPainter::default();
    let file_name = painter.paint(
        IMAGE_RESOLUTION,
        nid_to_node_data,
        wid_to_way_data,
        mapcss_rules,
    );

    print_peak_memory_usage();

    println!(
        "Saved image successfully to {} (took {:.2?}).",
        file_name,
        instant.elapsed()
    );

    Ok(())
}
