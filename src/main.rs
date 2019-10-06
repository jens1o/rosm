extern crate image;
extern crate line_drawing;
extern crate osmpbf;
extern crate rand;

use std::collections::HashMap;
use std::error::Error;
use std::time;
use std::time::Instant;

/// Holds an extract of the data from the Protobuf-file,
/// containing application-important data over nodes.
#[derive(Debug)]
struct NodeData {
    pub nid: i64,
    pub uid: i32,
    pub lat: f64,
    pub lon: f64,
    pub tags: Vec<(String, String)>,
    pub way: Option<i64>,
}

#[derive(Debug)]
struct WayData {
    pub wid: i64,
    pub tags: Vec<(String, String)>,
    pub refs: Vec<i64>,
}

impl WayData {
    pub fn is_waterway(&self) -> bool {
        self.tags
            .iter()
            .find(|(k, v)| match (&k[..], &v[..]) {
                ("type", "waterway") | ("waterway", "river") | ("waterway", "canal") => true,
                _ => false,
            })
            .is_some()
    }

    pub fn is_highway(&self) -> bool {
        self.tags
            .iter()
            .find(|(k, v)| match (&k[..], &v[..]) {
                ("highway", "motorway") | ("highway", "trunk") | ("highway", "secondary") => true,
                _ => false,
            })
            .is_some()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let reader = osmpbf::ElementReader::from_path("regbez-karlsruhe.osm.pbf")?;
    let start_instant = Instant::now();

    let mut nid_to_node_data: HashMap<i64, NodeData> = HashMap::new();
    let mut uid_to_name: HashMap<i32, String> = HashMap::new();
    let mut wid_to_way_data: HashMap<i64, WayData> = HashMap::new();
    let mut processed_entities: u32 = 0;

    reader
        .for_each(|element| {
            if let osmpbf::Element::Node(_) = element {
                panic!("OSM-Nodes not supported, use extractions with DenseNodes instead!");
            } else if let osmpbf::Element::DenseNode(node) = element {
                uid_to_name
                    .entry(node.uid)
                    .or_insert_with(|| node.user().unwrap().to_string());
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

                processed_entities += 1;

                if processed_entities % 100_000 == 0 {
                    println!("Processed {} elements", processed_entities);
                }
            } else if let osmpbf::Element::Way(way) = element {
                let way_info = way.info();
                let wid = way.id();
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

                wid_to_way_data.insert(
                    wid,
                    WayData {
                        wid,
                        tags: way
                            .tags()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect::<Vec<_>>(),
                        refs: way.refs().collect::<Vec<_>>(),
                    },
                );
                processed_entities += 1;

                if processed_entities % 100_000 == 0 {
                    println!("Processed {} elements", processed_entities);
                }
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

    println!("Pre-processing took {:.2?}.", start_instant.elapsed());

    let mut pixels = Vec::new();

    for way in wid_to_way_data.values() {
        if way
            .tags
            .iter()
            .find(|(_k, v)| v.to_lowercase().contains("straße"))
            .is_some()
        {
            continue;
        }

        let nodes = way
            .refs
            .iter()
            .map(|nid| nid_to_node_data.get(nid).unwrap())
            .collect::<Vec<_>>();

        for node_list in nodes[..].windows(2) {
            if let [node_a, node_b] = node_list {
                for (x, y) in line_drawing::Midpoint::<f64, i32>::new(
                    (node_a.lat * 100.0, node_a.lon * 100.0),
                    (node_b.lat * 100.0, node_b.lon * 100.0),
                ) {
                    pixels.push((node_a.nid, x, y));
                }
            } else {
                panic!("Windows iterator does not deliver expected size!");
            }
        }
    }

    pixels.sort();
    pixels.dedup();

    let min_x = pixels.iter().map(|(_, x, _y)| x).min().unwrap();
    let max_x = pixels.iter().map(|(_, x, _y)| x).max().unwrap();
    let min_y = pixels.iter().map(|(_, _x, y)| y).min().unwrap();
    let max_y = pixels.iter().map(|(_, _x, y)| y).max().unwrap();

    dbg!(min_x, max_x);
    dbg!(min_y, max_y);

    let mut image = image::ImageBuffer::new((max_x - min_x) as u32 + 1, (max_y - min_y) as u32 + 1);

    const WATER_COLOR: image::Rgb<u8> = image::Rgb([170u8, 211u8, 223u8]);
    const HIGHWAY_COLOR: image::Rgb<u8> = image::Rgb([249u8, 178u8, 156u8]);
    const NORMAL_COLOR: image::Rgb<u8> = image::Rgb([255u8, 255u8, 255u8]);
    const BG_COLOR: image::Rgb<u8> = image::Rgb([0u8, 0u8, 0u8]);

    // TODO: Mark cycleways
    //

    for (nid, pixel_x, pixel_y) in pixels
        .iter()
        .map(|(nid, pixel_x, pixel_y)| (nid, pixel_x - min_x, pixel_y - min_y))
    {
        let pixel = image.get_pixel(pixel_x as u32, pixel_y as u32);

        if pixel != &BG_COLOR && pixel != &NORMAL_COLOR {
            continue;
        }

        let way_data = nid_to_node_data
            .get(nid)
            .and_then(|node_data| node_data.way)
            .and_then(|wid| wid_to_way_data.get(&wid));

        image.put_pixel(
            pixel_x as u32,
            pixel_y as u32,
            if way_data.map(|way| way.is_waterway()).unwrap_or(false) {
                WATER_COLOR
            } else if way_data.map(|way| way.is_highway()).unwrap_or(false) {
                HIGHWAY_COLOR
            } else {
                dbg!(way_data.map(|way| dbg!(&way.tags)));

                NORMAL_COLOR
            },
        );
    }

    image
        .save(format!(
            "test-{}.png",
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    Ok(())
}
