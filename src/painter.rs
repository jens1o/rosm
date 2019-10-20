use crate::data::{self, NodeData, WayData};
use std::cmp;
use std::collections::HashMap;
use std::time;

pub trait Painter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<i64, NodeData>,
        wid_to_way_data: HashMap<i64, WayData>,
    ) -> String;
}

#[derive(Default)]
pub struct PngPainter {}

impl Painter for PngPainter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<i64, NodeData>,
        mut wid_to_way_data: HashMap<i64, WayData>,
    ) -> String {
        let mut pixels = Vec::new();

        for way in wid_to_way_data.values() {
            let line_width = way.line_width();

            // way is invisible, so we don't need to calculate its pixels
            if line_width == 0 {
                continue;
            }

            let nodes = way
                .refs
                .iter()
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

                        let mut distance_to_origin_pixel = line_width as u32;

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
                    pixels.push((node_data.nid, x, y));
                }
            });

        let (_, x_sample, y_sample) = pixels
            .iter()
            .next()
            .expect("At least one pixel needs to be drawn!");

        let mut min_x = x_sample;
        let mut max_x = x_sample;
        let mut min_y = y_sample;
        let mut max_y = y_sample;

        for (_, x, y) in pixels.iter() {
            min_x = cmp::min(x, min_x);
            max_x = cmp::max(x, max_x);

            min_y = cmp::min(y, min_y);
            max_y = cmp::max(y, max_y);
        }

        dbg!(min_x, max_x);
        dbg!(min_y, max_y);

        let image_width = crate::round_up_to((max_x - min_x) as u32 + 1, crate::IMAGE_PART_SIZE);
        let image_height = crate::round_up_to((max_y - min_y) as u32 + 1, crate::IMAGE_PART_SIZE);
        let image_pixels = image_width * image_height;

        dbg!(image_width, image_height, image_pixels);

        // order is changed to account for rotating by 270 degrees
        let mut image = image::ImageBuffer::new(image_height, image_width);

        for (nid, pixel_x, pixel_y) in pixels
            .iter()
            .map(|(nid, pixel_x, pixel_y)| (nid, pixel_x - min_x, pixel_y - min_y))
            // rotate by 270 degress
            .map(|(nid, pixel_x, pixel_y)| (nid, pixel_y, image_width - 1 - pixel_x))
        {
            let pixel = image.get_pixel(pixel_x as u32, pixel_y as u32);

            if pixel != &data::BG_COLOR && pixel != &data::NORMAL_COLOR {
                continue;
            }

            let node_data = nid_to_node_data.get(nid);

            let way_data = node_data
                .and_then(|node_data| node_data.way)
                .and_then(|wid| wid_to_way_data.get_mut(&wid));

            image.put_pixel(
                pixel_x as u32,
                pixel_y as u32,
                // TODO: Mark cycleways
                way_data
                    .map(|way| way.draw_color())
                    .unwrap_or(data::NORMAL_COLOR),
            );
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
