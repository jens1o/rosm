extern crate once_cell;
extern crate osmpbf;
extern crate pest;
extern crate winapi;
#[macro_use]
extern crate log;
extern crate flexi_logger;
#[macro_use]
extern crate pest_derive;
extern crate opengl_graphics;
extern crate piston_window;

mod data;
mod element;
mod extractor;
mod gui;
mod mapcss;

use crate::mapcss::declaration::{MapCssDeclaration, MapCssDeclarationList};
use crate::mapcss::selectors::{SelectorCondition, SelectorType};
use std::collections::HashMap;
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
            error!("getting memory info of process failed");
        }

        info!(
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

    let (map_css_acknowledgement, rules) = result.unwrap();
    dbg!(rules.get(&SelectorType::Any).unwrap());

    dbg!(instant.elapsed());

    if let Some(map_css_acknowledgement) = map_css_acknowledgement {
        info!(
            "Using MapCSS stylesheet \"{}\" for rendering. Parsed successfully.",
            map_css_acknowledgement.title()
        );
    }
    print_peak_memory_usage();

    println!("Extracting data!");

    let instant = Instant::now();
    let (nid_to_node_data, wid_to_way_data, rid_to_relation_data) =
        extractor::extract_data_from_filepath(String::from("bremen-latest.osm.pbf"))?;

    print_peak_memory_usage();
    info!(
        "Extracting the data from the given PBF file took {:.2?}. ({} nodes, {} ways, {} relations)",
        instant.elapsed(),
        nid_to_node_data.len(),
        wid_to_way_data.len(),
        rid_to_relation_data.len()
    );

    Ok(())
}

fn run_window(
    mapcss_ast: HashMap<SelectorType, HashMap<SelectorCondition, Vec<MapCssDeclaration>>>,
) {
    use opengl_graphics::{GlGraphics, OpenGL};
    use piston_window::*;

    let opengl = OpenGL::V4_5;

    println!("Preparing window!");

    let mut window: PistonWindow = WindowSettings::new("rosm", [640, 480])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    let mut gui = gui::Gui {
        gl: GlGraphics::new(opengl),
        canvas: element::canvas::CanvasElement {
            mapcss_declarations: MapCssDeclarationList::new(mapcss_ast),
        },
    };

    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            gui.render(&args);
        }
    }
}
