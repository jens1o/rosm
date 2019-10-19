/// Holds an extract of the data from the Protobuf-file,
/// containing application-important data over nodes.
#[derive(Debug)]
pub struct NodeData {
    pub nid: i64,
    pub uid: i32,
    pub lat: f64,
    pub lon: f64,
    pub tags: Vec<(String, String)>,
    pub way: Option<i64>,
}

#[derive(Debug)]
pub struct WayData {
    pub wid: i64,
    pub tags: Vec<(String, String)>,
    pub refs: Vec<i64>,
}

impl WayData {
    pub fn is_waterway(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("type", "waterway") | ("waterway", "river") | ("waterway", "canal") => true,
            _ => false,
        })
    }

    pub fn is_highway(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("highway", "motorway")
            | ("highway", "trunk")
            | ("highway", "secondary")
            | ("highway", "motorway_link") => true,
            _ => false,
        })
    }

    pub fn is_railway(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("railway", "rail") => true,
            _ => false,
        })
    }
}
