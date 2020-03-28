extern crate cssparser;
extern crate image;
extern crate line_drawing;
extern crate markup5ever;
extern crate once_cell;
extern crate osmpbf;
extern crate pest;
extern crate winapi;
#[macro_use]
extern crate log;
extern crate flexi_logger;
#[macro_use]
extern crate pest_derive;

mod data;
mod extractor;
mod mapcss;

use std::error::Error;
use std::time::Instant;
#[cfg(windows)]
use winapi::um::processthreadsapi::GetCurrentProcess;
#[cfg(windows)]
use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

pub(crate) fn print_peak_memory_usage() {
    #[cfg(windows)]
    unsafe {
        let mut pmc = std::mem::zeroed::<PROCESS_MEMORY_COUNTERS>();
        let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, cb) == 0 {
            eprintln!("getting memory info of process failed");
        }

        println!(
            "Peak memory usage: {} MB",
            (pmc.PeakWorkingSetSize as u64) / 1024 / 1024
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    flexi_logger::Logger::with_str("debug")
        .format(flexi_logger::colored_detailed_format)
        .start()
        .unwrap();

    print_peak_memory_usage();

    let instant = Instant::now();
    let result =
        mapcss::parser::MapCssParser::parse_mapcss(include_str!("../include/target.mapcss"));

    let (map_css_acknowledgement, rules) = result;

    dbg!(instant.elapsed());

    if let Some(map_css_acknowledgement) = map_css_acknowledgement {
        info!(
            "Using MapCSS stylesheet \"{}\" for rendering. Parsed successfully.",
            map_css_acknowledgement.title()
        );
    }
    print_peak_memory_usage();

    std::process::exit(0);

    println!("Extracting data!");

    let instant = Instant::now();
    let (nid_to_node_data, wid_to_way_data, rid_to_relation_data) =
        extractor::extract_data_from_filepath(String::from("bremen-latest.osm.pbf"))?;

    print_peak_memory_usage();
    println!(
        "Extracting the data from the given PBF file took {:.2?}. ({} nodes, {} ways)",
        instant.elapsed(),
        nid_to_node_data.len(),
        wid_to_way_data.len()
    );

    Ok(())
}
