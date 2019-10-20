pub const WATER_COLOR: image::Rgb<u8> = image::Rgb([170u8, 211u8, 223u8]);
pub const HIGHWAY_COLOR: image::Rgb<u8> = image::Rgb([249u8, 178u8, 156u8]);
pub const PRIMARY_ROAD_COLOR: image::Rgb<u8> = image::Rgb([252u8, 214u8, 164u8]);
pub const SECONDARY_ROAD_COLOR: image::Rgb<u8> = image::Rgb([247u8, 250u8, 191u8]);
pub const NORMAL_COLOR: image::Rgb<u8> = image::Rgb([255u8, 255u8, 255u8]);
pub const RAILWAY_COLOR: image::Rgb<u8> = image::Rgb([146u8, 205u8, 0u8]);
pub const BG_COLOR: image::Rgb<u8> = image::Rgb([0u8, 0u8, 0u8]);

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
    pub draw_style: Option<image::Rgb<u8>>,
}

impl WayData {
    pub fn line_width(&self) -> u8 {
        if self.is_waterway() || self.is_highway() {
            5
        } else if self.is_primary_road() {
            2
        } else {
            1
        }
    }

    pub fn draw_color(&mut self) -> image::Rgb<u8> {
        if let Some(draw_style) = self.draw_style {
            draw_style
        } else {
            if self.is_waterway() {
                self.draw_style = Some(WATER_COLOR);
            } else if self.is_highway() {
                self.draw_style = Some(HIGHWAY_COLOR);
            } else if self.is_primary_road() {
                self.draw_style = Some(PRIMARY_ROAD_COLOR);
            } else if self.is_secondary_road() {
                self.draw_style = Some(SECONDARY_ROAD_COLOR);
            } else if self.is_railway() {
                self.draw_style = Some(RAILWAY_COLOR);
            } else {
                self.draw_style = Some(NORMAL_COLOR);
            }

            self.draw_style.unwrap()
        }
    }

    pub fn is_waterway(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("type", "waterway") | ("waterway", "river") | ("waterway", "canal") => true,
            _ => false,
        })
    }

    pub fn is_highway(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("highway", "motorway")
            | ("highway", "motorway_link")
            | ("highway", "trunk")
            | ("highway", "trunk_link") => true,
            _ => false,
        })
    }
    pub fn is_primary_road(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("highway", "primary") => true,
            _ => false,
        })
    }

    pub fn is_secondary_road(&self) -> bool {
        self.tags.iter().any(|(k, v)| match (&k[..], &v[..]) {
            ("highway", "secondary") => true,
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
