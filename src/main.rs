extern crate once_cell;
extern crate osmpbf;
extern crate pest;
#[cfg(windows)]
extern crate winapi;
#[macro_use]
extern crate log;
extern crate flexi_logger;
#[macro_use]
extern crate pest_derive;
extern crate image;

mod data;
mod element;
mod extractor;
mod mapcss;
mod painter;

use mapcss::declaration::MapCssDeclarationList;
use painter::Painter;
use std::error::Error;
use std::time::Instant;
#[cfg(windows)]
use winapi::{
    shared::minwindef::FALSE,
    um::processthreadsapi::GetCurrentProcess,
    um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
};
const IMAGE_PART_SIZE: u32 = 512;

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

    (target - int % target) + int
}

pub(crate) fn print_peak_memory_usage() {
    #[cfg(windows)]
    unsafe {
        let mut pmc = std::mem::zeroed::<PROCESS_MEMORY_COUNTERS>();
        let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, cb) == FALSE {
            error!("Getting memory info of current process failed!");
        } else {
            info!(
                "Peak memory usage: {} MB",
                (pmc.PeakWorkingSetSize as u64) / 1024 / 1024
            );
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    flexi_logger::Logger::try_with_str("debug")?
        .format(flexi_logger::colored_detailed_format)
        .start()?;

    print_peak_memory_usage();

    let instant = Instant::now();
    let result = mapcss::parser::MapCssParser::parse_mapcss(include_str!("../include/main.mapcss"));

    let (map_css_acknowledgement, rules) = result.unwrap();

    dbg!(instant.elapsed());

    if let Some(map_css_acknowledgement) = map_css_acknowledgement {
        info!(
            "Using MapCSS stylesheet \"{}\" for rendering. Parsed successfully.",
            map_css_acknowledgement.title()
        );
    } else {
        info!("Using MapCSS stylesheet for rendering. Parsed successfully.");
    }
    print_peak_memory_usage();

    info!("Extracting data!");

    let instant = Instant::now();
    let (nid_to_node_data, wid_to_way_data, rid_to_relation_data) =
        extractor::extract_data_from_filepath(String::from("liechtenstein-latest.osm.pbf"))?;

    print_peak_memory_usage();
    info!(
        "Extracting the data from the given PBF file took {:.2?}. ({} nodes, {} ways, {} relations)",
        instant.elapsed(),
        nid_to_node_data.len(),
        wid_to_way_data.len(),
        rid_to_relation_data.len()
    );

    let mut painter = painter::PngPainter::default();

    for res in [2_500.0f64] {
        let mapcss_ast = MapCssDeclarationList::new(rules.clone());

        painter.paint(
            res,
            mapcss_ast,
            nid_to_node_data.clone(),
            wid_to_way_data.clone(),
            rid_to_relation_data.clone(),
        );
    }

    Ok(())
}
