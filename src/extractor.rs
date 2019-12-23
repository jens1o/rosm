use crate::data::{NodeData, WayData};
use std::collections::HashMap;
use std::num::NonZeroI64;

pub fn extract_data_from_filepath(
    file_path: String,
) -> Result<(HashMap<NonZeroI64, NodeData>, HashMap<NonZeroI64, WayData>), osmpbf::Error> {
    let reader = osmpbf::ElementReader::from_path(file_path)?;

    let mut nid_to_node_data: HashMap<NonZeroI64, NodeData> = HashMap::new();
    let mut wid_to_way_data: HashMap<NonZeroI64, WayData> = HashMap::new();

    reader.for_each(|element| {
        // TODO: Parse relations
        if let osmpbf::Element::Node(_) = element {
            panic!("OSM-Nodes not supported (yet), use data extractions with DenseNodes instead!");
        } else if let osmpbf::Element::DenseNode(node) = element {
            nid_to_node_data.insert(
                NonZeroI64::new(node.id).expect("Node id must not zero!"),
                NodeData {
                    // unwrap is safe because we would panic in inserting the key into this hashmap already
                    nid: NonZeroI64::new(node.id).unwrap(),
                    tags: node
                        .tags()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<Vec<_>>(),
                    lat: node.lat(),
                    lon: node.lon(),
                    way: None,
                },
            );
        } else if let osmpbf::Element::Way(way) = element {
            let wid = way.id();

            wid_to_way_data.insert(
                NonZeroI64::new(wid).expect("Way id must not be zero!"),
                WayData {
                    wid: NonZeroI64::new(wid).unwrap(),
                    tags: way
                        .tags()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<Vec<_>>(),
                    refs: way
                        .refs()
                        .map(|x| NonZeroI64::new(x).expect("Node id must not be zero"))
                        .collect::<Vec<_>>(),
                },
            );
        }
    })?;

    wid_to_way_data.iter().for_each(|(wid, way_data)| {
        way_data.refs.iter().for_each(|nid| {
            // ways in the data extract may contain nodes outside of our area, which
            // is why we need to silently ignore them.
            if let Some(node_data) = nid_to_node_data.get_mut(nid) {
                node_data.way = Some(*wid);
            } else {
                panic!("No node found for #{} (belonging to way #{})!", nid, wid);
            }
        });
    });

    nid_to_node_data.shrink_to_fit();
    wid_to_way_data.shrink_to_fit();

    Ok((nid_to_node_data, wid_to_way_data))
}
