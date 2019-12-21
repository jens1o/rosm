extern crate cssparser;
extern crate image;
extern crate line_drawing;
extern crate markup5ever;
extern crate osmpbf;

mod data;
mod extractor;
mod mapcss;
mod painter;

// use crate::painter::Painter;
use crate::data::ToNodeRef;
use kuchiki::{Attribute, ExpandedName, Node, NodeDataRef};
use markup5ever::{LocalName, Namespace, QualName};
use std::collections::HashMap;
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

    let mut nid_to_noderef = HashMap::with_capacity(nid_to_node_data.len());
    let mut wid_to_noderef = HashMap::with_capacity(wid_to_way_data.len());

    let instant = Instant::now();
    for node in nid_to_node_data.values() {
        nid_to_noderef.insert(node.nid, node.node_ref());
    }
    for way in wid_to_way_data.values() {
        wid_to_noderef.insert(way.wid, way.node_ref());
    }
    println!("Generating NodeRefs took {:.2?}", instant.elapsed());

    println!("Parsing mapcss.");
    let mapcss_rules = mapcss::parse_mapcss(include_str!("../include/mapcss.css"));

    for parse_result in mapcss_rules {
        let node_ref = kuchiki::NodeRef::new_element(
            QualName::new(None, Namespace::from(""), LocalName::from("node")),
            vec![(
                ExpandedName::new(Namespace::from(""), "amenity"),
                Attribute {
                    prefix: None,
                    value: "drinking_water".into(),
                },
            )],
        );
        let rule = &parse_result.original_rule;

        if rule
            .selectors
            .matches(&NodeDataRef::new_opt(node_ref, Node::as_element).unwrap())
        {
            dbg!(&rule.declarations);
        }
        // TODO: Parse our elements into a "DOM"
    }

    // let mut painter = painter::PngPainter::default();
    // let file_name = painter.paint(IMAGE_RESOLUTION, nid_to_node_data, wid_to_way_data);

    // println!("Saved image successfully to {}.", file_name);

    Ok(())
}
