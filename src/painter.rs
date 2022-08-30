use crate::data::{ElementData, NodeData, RelationData, WayData};
use crate::element::canvas::CanvasElement;
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToColorValue, ToIntegerValue, RGBA,
};
use crate::mapcss::parser::IntSize;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::num::NonZeroI64;
use std::path::PathBuf;
use std::time::Instant;

pub trait Painter {
    /// Paints the given data styled by the mapcss ast and returns the filename of the saved file.
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        mapcss_ast: MapCssDeclarationList,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
    ) -> PathBuf;
}

#[derive(Default)]
pub struct PngPainter {}

impl Painter for PngPainter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        mapcss_ast: MapCssDeclarationList,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
    ) -> PathBuf {
        const IMAGE_PART_SIZE: u32 = 64;

        let canvas = CanvasElement {};

        let mut processed_ways = 0;

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for way_refs in wid_to_way_data.values().map(|x| x.refs()) {
            for ref_node_id in way_refs {
                let ref_data = nid_to_node_data.get(ref_node_id).unwrap();

                let lat = (ref_data.lat * image_resolution_factor).ceil() as i32;
                let lon = (ref_data.lon * image_resolution_factor).ceil() as i32;

                min_x = cmp::min(min_x, lat);
                max_x = cmp::max(max_x, lat);

                min_y = cmp::min(min_y, lon);
                max_y = cmp::max(max_y, lon);
            }
        }

        let image_width =
            crate::round_up_to((((max_x - min_x) as f64).abs()) as u32, IMAGE_PART_SIZE);
        let image_height =
            crate::round_up_to((((max_y - min_y) as f64).abs()) as u32, IMAGE_PART_SIZE);

        let background_color: image::Rgb<u8> = canvas.background_color(&mapcss_ast).into();

        let mut image_buffer: image::RgbImage =
            image::ImageBuffer::from_pixel(image_height, image_width, background_color);

        let render_start_instant = Instant::now();
        let mut no_inside_count: u32 = 0;
        let mut flood_fill_pixel_count: u32 = 0;

        struct WayWithZIndex<'a> {
            z_index: IntSize,
            way_data: &'a WayData,
        }

        info!("Sorting by z-index…");
        let mut z_index_ordered_ways = wid_to_way_data
            .values()
            .map(|way_data| WayWithZIndex {
                z_index: mapcss_ast
                    .search_or_default(
                        Box::new(way_data.clone()),
                        &MapCssDeclarationProperty::ZIndex,
                        &MapCssDeclarationValueType::Integer(0),
                    )
                    .to_integer(),
                way_data,
            })
            .filter(|way| way.way_data.way_id() == NonZeroI64::new(366476165).unwrap())
            .collect::<Vec<_>>();

        dbg!(z_index_ordered_ways.len());

        z_index_ordered_ways.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        info!("Rasterizing ways…");
        for way_data in z_index_ordered_ways.iter().map(|x| x.way_data) {
            processed_ways += 1;

            if processed_ways % 15000 == 0 {
                info!("{} ways rendered…", processed_ways);
            }

            let way_refs = way_data.refs();

            let way_color: Option<image::Rgba<u8>> = mapcss_ast
                .search_cascading(
                    Box::new(way_data.clone()),
                    &MapCssDeclarationProperty::Color,
                )
                .map(|x| x.to_color().into());

            if way_color.is_none() {
                continue;
            }

            let way_color = way_color.unwrap();

            let way_fill_color = mapcss_ast
                .search_cascading(
                    Box::new(way_data.clone()),
                    &MapCssDeclarationProperty::FillColor,
                )
                .map(|x| x.to_color().into())
                .unwrap_or(way_color);

            let way_width: u32 = mapcss_ast
                .search_cascading(
                    Box::new(way_data.clone()),
                    &MapCssDeclarationProperty::Width,
                )
                .map(|x| x.to_integer())
                .and_then(|x| x.try_into().ok())
                .unwrap_or(1);

            if way_width == 0 {
                continue;
            }
            info!("checking whether way is closed");

            let is_closed_way = way_data.has_closed_path();

            let mut pixeled_min_x_coordinates: (u32, u32) = (u32::MAX, u32::MAX);
            let mut pixeled_max_x_coordinates: (u32, u32) = (u32::MIN, u32::MIN);

            // draw the outline of the way and remember it
            let mut outline_pixels: HashMap<u32, Vec<u32>> = HashMap::new();

            assert!(way_refs.len() > 0);

            for ref_node_ids in way_refs[..].windows(2) {
                if let [node_a_id, node_b_id] = ref_node_ids {
                    let node_a = nid_to_node_data.get(node_a_id).unwrap();
                    let node_b = nid_to_node_data.get(node_b_id).unwrap();

                    for ((x, y), _alpha) in line_drawing::XiaolinWu::<f64, i32>::new(
                        (
                            node_a.lat * image_resolution_factor,
                            node_a.lon * image_resolution_factor,
                        ),
                        (
                            node_b.lat * image_resolution_factor,
                            node_b.lon * image_resolution_factor,
                        ),
                    ) {
                        // rotate image by 270 degrees means we need to swap x and y and the y pixel needs to be subtracted from the image width.
                        // Subtract / Add 2 from it as the image buffer uses 0-indexing AND we may not write to the last pixel.
                        let image_x = (y - min_y + 2) as u32;
                        let image_y = image_width - 2 - (x - min_x) as u32;

                        if way_width == 1 {
                            image_buffer.put_pixel(
                                image_x,
                                image_y,
                                image::Rgb([way_color[0], way_color[1], way_color[2]]),
                            );

                            if is_closed_way {
                                outline_pixels.entry(image_y).or_default().push(image_x);

                                // speedup "are we inside the drawn way boundaries?"
                                if image_x > pixeled_max_x_coordinates.0 {
                                    pixeled_max_x_coordinates = (image_x, image_y);
                                }

                                if image_x < pixeled_min_x_coordinates.0 {
                                    pixeled_min_x_coordinates = (image_x, image_y);
                                }
                            }
                        } else {
                            let start_x = (image_x + way_width / 2).min(image_width - 1);
                            let end_x = if way_width / 2 > image_x {
                                0
                            } else {
                                image_x - way_width / 2
                            };

                            for x in start_x..=end_x {
                                image_buffer.put_pixel(
                                    x,
                                    image_y,
                                    image::Rgb([
                                        way_fill_color[0],
                                        way_fill_color[1],
                                        way_fill_color[2],
                                    ]),
                                );

                                if is_closed_way {
                                    outline_pixels.entry(image_y).or_default().push(x);

                                    // speedup "are we inside the drawn way boundaries?"
                                    if image_x > pixeled_max_x_coordinates.0 {
                                        pixeled_max_x_coordinates = (image_x, image_y);
                                    }

                                    if image_x < pixeled_min_x_coordinates.0 {
                                        pixeled_min_x_coordinates = (image_x, image_y);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    unreachable!();
                }
            }

            dbg!(pixeled_max_x_coordinates, pixeled_min_x_coordinates);

            // flood fill the closed way using the algorithm specified here: https://en.wikipedia.org/wiki/Flood_fill#Span_Filling
            if is_closed_way
                && pixeled_min_x_coordinates != pixeled_max_x_coordinates
                && !are_image_coordinates_horizontally_next_to_each_other(
                    pixeled_min_x_coordinates,
                    pixeled_max_x_coordinates,
                )
            {
                info!("trying to find inside to start flood filling");

                fn get_flood_filled_pixels(
                    (x, y): (u32, u32),
                    outline_pixels: &HashMap<u32, Vec<u32>>,
                ) -> Vec<(u32, u32)> {
                    if !is_inside(&(x, y), outline_pixels) {
                        warn!("initial value is not inside the boundaries");
                        return Vec::new();
                    }

                    let mut x = x as i64;
                    let y = y as i64;

                    let mut flood_filled_pixels: Vec<(u32, u32)> = Vec::new();

                    let mut stack: Vec<(i64, i64, i64, i64)> =
                        vec![(x, x, y, 1), (x, x, y - 1, -1)];

                    while let Some((mut x1, x2, y, dy)) = stack.pop() {
                        x = x1;

                        if is_inside(
                            &(x.try_into().unwrap(), y.try_into().unwrap()),
                            outline_pixels,
                        ) {
                            while is_inside(
                                &((x - 1).try_into().unwrap(), y.try_into().unwrap()),
                                outline_pixels,
                            ) {
                                flood_filled_pixels
                                    .push(((x - 1).try_into().unwrap(), y.try_into().unwrap()));

                                x -= 1;
                            }
                        }

                        if x < x1 {
                            stack.push((x, x1 - 1, (y - dy), -dy));
                        }

                        while x1 <= x2 {
                            while is_inside(
                                &(x1.try_into().unwrap(), y.try_into().unwrap()),
                                outline_pixels,
                            ) {
                                flood_filled_pixels
                                    .push((x1.try_into().unwrap(), y.try_into().unwrap()));

                                x1 += 1;

                                stack.push((x, x1 - 1, (y + dy), dy));

                                if x1 - 1 > x2 {
                                    stack.push((x2 + 1, x1 - 1, y - dy, -dy));
                                }
                            }

                            x1 += 1;

                            while x1 < x2
                                && !is_inside(
                                    &(x1.try_into().unwrap(), y.try_into().unwrap()),
                                    outline_pixels,
                                )
                            {
                                x1 += 1;
                            }

                            x = x1;

                            if stack.len() > 10_000 {
                                warn!("stack size exploded, not filling polygon");
                                return Vec::new();
                            }
                        }
                    }
                    assert!(flood_filled_pixels.len() > 0);

                    flood_filled_pixels
                }

                let mut has_found_inside = false;

                for (x, y) in [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    // (0, 0) is always not inside the outline
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ] {
                    let new_x: u32;
                    if let Ok(x) = (pixeled_min_x_coordinates.0 as i64 + x).try_into() {
                        new_x = x;
                    } else {
                        continue;
                    }

                    let new_y: u32;
                    if let Ok(y) = (pixeled_min_x_coordinates.1 as i64 + y).try_into() {
                        new_y = y;
                    } else {
                        continue;
                    }

                    if new_x >= image_buffer.width() || new_y >= image_buffer.height() {
                        continue;
                    }

                    if is_inside(&(new_x, new_y), &outline_pixels) {
                        has_found_inside = true;

                        info!("has found inside");

                        for flood_filled_pixel in
                            get_flood_filled_pixels((new_x, new_y), &outline_pixels)
                        {
                            dbg!(&flood_filled_pixel);

                            // validate results
                            if flood_filled_pixel.0 >= image_buffer.width()
                                || flood_filled_pixel.1 >= image_buffer.height()
                            {
                                continue;
                            }

                            flood_fill_pixel_count += 1;

                            image_buffer.put_pixel(
                                flood_filled_pixel.0,
                                flood_filled_pixel.1,
                                image::Rgb([
                                    way_fill_color[0],
                                    way_fill_color[1],
                                    way_fill_color[2],
                                ]),
                            );
                        }

                        break;
                    }
                }

                if !has_found_inside {
                    no_inside_count += 1;
                }
            }
        }

        let render_duration = render_start_instant.elapsed();

        info!(
            "Rendering took {:.2}s. {:.2} ways/sec. Saving …",
            render_duration.as_secs_f32(),
            processed_ways as f64 / (render_duration.as_nanos() as f64 * 1e-9)
        );

        dbg!(image_height);
        dbg!(image_width);
        dbg!(flood_fill_pixel_count);

        let file_path = PathBuf::from("test.png");

        let save_start_instant = Instant::now();

        image_buffer.save(&file_path).unwrap();

        info!(
            "Image saved successfully, took {:.2}s.",
            save_start_instant.elapsed().as_secs_f32()
        );
        if no_inside_count > 0 {
            warn!("couldn't find a starting point to flood fill {no_inside_count} time(s)");
        }

        file_path
    }
}

/// Determines whether the given point is inside the outline given by `outline_pixels`.
fn is_inside(image_point: &(u32, u32), outline_pixels: &HashMap<u32, Vec<u32>>) -> bool {
    if let Some(x_pixels) = outline_pixels.get(&image_point.1) {
        if x_pixels.len() < 2 {
            return false;
        }

        let mut x_pixels = x_pixels.clone();
        x_pixels.sort_unstable();
        // TODO: Check whether there is a case where we need dedup()?

        let partition_point = x_pixels.partition_point(|x| x < &image_point.0);
        partition_point > 0 && partition_point < x_pixels.len()
    } else {
        false
    }
}

fn are_image_coordinates_horizontally_next_to_each_other(a: (u32, u32), b: (u32, u32)) -> bool {
    if a.1 != b.1 {
        return false;
    }

    return a.0.max(b.0) - a.0.min(b.0) == 1;
}
