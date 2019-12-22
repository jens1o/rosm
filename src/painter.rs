use crate::data::{NodeData, ToNodeRef, WayData};
use crate::mapcss::style::Size;
use crate::mapcss::{MapCssPropertyDeclaration, MapCssRule};
use kuchiki::{Node, NodeDataRef};
use std::cmp;
use std::collections::HashMap;
use std::time::{self, Instant};

#[derive(Clone, Copy)]
pub struct RenderStyle {
    z_index: u16,
    color: cssparser::RGBA,
    width: Size,
}

impl Default for RenderStyle {
    fn default() -> Self {
        RenderStyle {
            color: cssparser::RGBA::new(255, 255, 255, 0),
            z_index: 0,
            width: Size(1.0),
        }
    }
}

pub trait Painter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<i64, NodeData>,
        wid_to_way_data: HashMap<i64, WayData>,
        mapcss_rules: Vec<MapCssRule>,
    ) -> String;
}

#[derive(Default)]
pub struct PngPainter {}

impl Painter for PngPainter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<i64, NodeData>,
        wid_to_way_data: HashMap<i64, WayData>,
        mapcss_rules: Vec<MapCssRule>,
    ) -> String {
        // make a good guess for all the pixels
        let mut pixels = Vec::with_capacity(wid_to_way_data.len() + nid_to_node_data.len());
        let mut id_to_render_style = HashMap::with_capacity(nid_to_node_data.len());

        for way in wid_to_way_data.values() {
            let way_ref_data = NodeDataRef::new_opt(way.node_ref(), Node::as_element).unwrap();
            let mut render_style = RenderStyle::default();

            for mapcss_rule in mapcss_rules.iter() {
                if mapcss_rule.original_rule.selectors.matches(&way_ref_data) {
                    for rule_declaration in &mapcss_rule.original_rule.declarations {
                        use MapCssPropertyDeclaration::*;

                        match rule_declaration {
                            ZIndex(z_index) => render_style.z_index = *z_index,
                            Color(color) => render_style.color = *color,
                            Width(width) => render_style.width = *width,
                        }
                    }
                }
            }

            // way is invisible, so we don't need to calculate its pixels
            if render_style.width.0 <= 0.0 {
                continue;
            }

            let nodes = way
                .refs
                .iter()
                .inspect(|nid| {
                    id_to_render_style.insert(**nid, render_style);
                })
                .map(|nid| nid_to_node_data.get(nid).unwrap())
                .collect::<Vec<_>>();

            for node_list in nodes[..].windows(2) {
                if let [node_a, node_b] = node_list {
                    for (x, y) in line_drawing::Midpoint::<f64, i64>::new(
                        (
                            node_a.lat * image_resolution_factor,
                            node_a.lon * image_resolution_factor,
                        ),
                        (
                            node_b.lat * image_resolution_factor,
                            node_b.lon * image_resolution_factor,
                        ),
                    )
                    .map(|(x, y)| (x as u32, y as u32))
                    {
                        // make the line a bit thicker

                        let mut distance_to_origin_pixel = render_style.width.0 as u32;

                        pixels.push((node_a.nid, x, y));

                        while distance_to_origin_pixel > 1 {
                            distance_to_origin_pixel -= 1;
                            pixels.push((
                                node_a.nid,
                                x + distance_to_origin_pixel,
                                y + distance_to_origin_pixel,
                            ));
                            pixels.push((
                                node_a.nid,
                                x + distance_to_origin_pixel,
                                y - distance_to_origin_pixel,
                            ));
                            pixels.push((
                                node_a.nid,
                                x - distance_to_origin_pixel,
                                y + distance_to_origin_pixel,
                            ));
                            pixels.push((
                                node_a.nid,
                                x - distance_to_origin_pixel,
                                y - distance_to_origin_pixel,
                            ));
                        }
                    }
                } else {
                    panic!("Windows iterator does not deliver expected size!");
                }
            }
        }

        nid_to_node_data
            .values()
            // Filter out nodes we've already drawn through the ways
            .filter(|node_data| node_data.way.is_none())
            .for_each(|node_data| {
                // match mapcss rules

                let node_ref_data =
                    NodeDataRef::new_opt(node_data.node_ref(), Node::as_element).unwrap();
                let mut render_style = RenderStyle::default();

                for mapcss_rule in mapcss_rules.iter() {
                    if mapcss_rule.original_rule.selectors.matches(&node_ref_data) {
                        for rule_declaration in &mapcss_rule.original_rule.declarations {
                            use MapCssPropertyDeclaration::*;

                            match rule_declaration {
                                ZIndex(z_index) => render_style.z_index = *z_index,
                                Color(color) => render_style.color = *color,
                                Width(width) => render_style.width = *width,
                            }
                        }
                    }
                }

                // node is invisible, don't draw it
                if render_style.width.0 <= 0.0 {
                    return;
                }

                for (x, y) in line_drawing::Midpoint::<f64, i64>::new(
                    (
                        node_data.lat * image_resolution_factor,
                        node_data.lon * image_resolution_factor,
                    ),
                    (
                        node_data.lat * image_resolution_factor,
                        node_data.lon * image_resolution_factor,
                    ),
                )
                .map(|(x, y)| (x as u32, y as u32))
                {
                    let mut distance_to_origin_pixel = render_style.width.0 as u32;

                    pixels.push((node_data.nid, x, y));

                    while distance_to_origin_pixel > 1 {
                        distance_to_origin_pixel -= 1;
                        pixels.push((
                            node_data.nid,
                            x + distance_to_origin_pixel,
                            y + distance_to_origin_pixel,
                        ));
                        pixels.push((
                            node_data.nid,
                            x + distance_to_origin_pixel,
                            y - distance_to_origin_pixel,
                        ));
                        pixels.push((
                            node_data.nid,
                            x - distance_to_origin_pixel,
                            y + distance_to_origin_pixel,
                        ));
                        pixels.push((
                            node_data.nid,
                            x - distance_to_origin_pixel,
                            y - distance_to_origin_pixel,
                        ));
                    }
                }

                id_to_render_style.insert(node_data.nid, render_style);
            });

        // sort pixels by z-index (ascending)
        let instant = Instant::now();
        pixels.sort_unstable_by(|a, b| {
            let a_style_data = id_to_render_style.get(&a.0).unwrap_or_else(|| {
                panic!(
                    "No render style found for node #{} when comparing z-indexes!",
                    a.0
                )
            });

            let b_style_data = id_to_render_style.get(&b.0).unwrap_or_else(|| {
                panic!(
                    "No render style found for node #{} when comparing z-indexes!",
                    b.0
                )
            });

            a_style_data.z_index.cmp(&b_style_data.z_index)
        });
        println!("Sorting by z-index took {:.2?}", instant.elapsed());

        let (_, x_sample, y_sample) = &pixels
            .iter()
            .next()
            .expect("At least one pixel needs to be drawn!");

        let mut min_x = x_sample;
        let mut max_x = x_sample;
        let mut min_y = y_sample;
        let mut max_y = y_sample;

        let instant = Instant::now();
        for (_, x, y) in pixels.iter() {
            min_x = cmp::min(x, min_x);
            max_x = cmp::max(x, max_x);

            min_y = cmp::min(y, min_y);
            max_y = cmp::max(y, max_y);
        }

        dbg!(min_x, max_x);
        dbg!(min_y, max_y);
        println!("Finding the min/max values took {:.2?}", instant.elapsed());

        let image_width = crate::round_up_to((max_x - min_x) + 1, crate::IMAGE_PART_SIZE);
        let image_height = crate::round_up_to((max_y - min_y) + 1, crate::IMAGE_PART_SIZE);

        dbg!(image_width, image_height);

        // order is changed to account for rotating by 270 degrees
        let mut image = image::ImageBuffer::new(image_height, image_width);

        for (nid, pixel_x, pixel_y) in pixels
            .iter()
            .map(|(nid, pixel_x, pixel_y)| (nid, pixel_x - min_x, pixel_y - min_y))
            // rotate by 270 degress
            .map(|(nid, pixel_x, pixel_y)| (nid, pixel_y, image_width - 1 - pixel_x))
        {
            let render_style = id_to_render_style
                .get(&nid)
                .expect("No render style found for node!");

            let r = render_style.color.red;
            let g = render_style.color.green;
            let b = render_style.color.blue;

            image.put_pixel(pixel_x, pixel_y, image::Rgb([r, g, b]));
        }

        // save image and return the filename
        let file_name = format!(
            "test-{}.png",
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        image.save(&file_name).unwrap();

        file_name
    }
}
