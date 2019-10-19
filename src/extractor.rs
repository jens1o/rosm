use crate::data::{NodeData, WayData};
use std::collections::HashMap;

pub fn extract_data_from_filepath(
    file_path: String,
    user_lookup: bool,
) -> Result<
    (
        HashMap<i64, NodeData>,
        HashMap<i64, WayData>,
        HashMap<i32, String>,
    ),
    osmpbf::Error,
> {
    let reader = osmpbf::ElementReader::from_path(file_path)?;

    let mut nid_to_node_data: HashMap<i64, NodeData> = HashMap::new();
    let mut uid_to_name: HashMap<i32, String> = HashMap::new();
    let mut wid_to_way_data: HashMap<i64, WayData> = HashMap::new();

    reader
        .for_each(|element| {
            if let osmpbf::Element::Node(_) = element {
                panic!(
                    "OSM-Nodes not supported (yet), use data extractions with DenseNodes instead!"
                );
            } else if let osmpbf::Element::DenseNode(node) = element {
                if user_lookup {
                    uid_to_name
                        .entry(node.uid)
                        .or_insert_with(|| node.user().unwrap().to_string());
                }
                nid_to_node_data.insert(
                    node.id,
                    NodeData {
                        nid: node.id,
                        uid: node.uid,
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
                let way_info = way.info();
                let wid = way.id();

                if user_lookup {
                    // add author to list of known authors if we have all metadata
                    if let Some(uid) = way_info.uid() {
                        // check whether we don't already know this user
                        if let std::collections::hash_map::Entry::Vacant(vacant) =
                            uid_to_name.entry(uid)
                        {
                            if let Some(Ok(user_name)) = way_info.user() {
                                vacant.insert(user_name.to_owned());
                            }
                        }
                    }
                }

                wid_to_way_data.insert(
                    wid,
                    WayData {
                        wid,
                        tags: way
                            .tags()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect::<Vec<_>>(),
                        refs: way.refs().collect::<Vec<_>>(),
                        draw_style: None,
                    },
                );
            }
        })
        .unwrap();

    wid_to_way_data.iter().for_each(|(wid, way_data)| {
        way_data.refs.iter().for_each(|nid| {
            // ways in the data extract may contain nodes outside of our area, which
            // is why we need to silently ignore them.
            if let Some(node_data) = nid_to_node_data.get_mut(nid) {
                node_data.way = Some(*wid);
            } else {
                panic!("No node found for #{} (belonging to way #{})", nid, wid);
            }
        });
    });

    Ok((nid_to_node_data, wid_to_way_data, uid_to_name))
}
