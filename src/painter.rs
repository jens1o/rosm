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
    ) -> String;
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
    ) -> String {
        const IMAGE_PART_SIZE: u32 = 256;
        const DRAWN_WAYS: usize = 1_000_000;

        let canvas = CanvasElement {};

        let mut rendered_ways = 0;

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for way_refs in wid_to_way_data.values().take(DRAWN_WAYS).map(|x| x.refs()) {
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

        let background_color: image::Rgba<u8> = canvas.background_color(&mapcss_ast).into();

        let mut image_buffer: image::RgbaImage =
            image::ImageBuffer::from_pixel(image_height, image_width, background_color);

        let render_start_instant = Instant::now();

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
            .collect::<Vec<_>>();

        z_index_ordered_ways.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        info!("Rasterizing ways…");
        for way_data in z_index_ordered_ways.iter().map(|x| x.way_data) {
            let way_refs = way_data.refs();

            let way_color: image::Rgba<u8> = mapcss_ast
                .search_or_default(
                    Box::new(way_data.clone()),
                    &MapCssDeclarationProperty::Color,
                    &MapCssDeclarationValueType::Color(RGBA {
                        red: 200,
                        green: 200,
                        blue: 200,
                        alpha: 255,
                    }),
                )
                .to_color()
                .into();

            let is_closed_way = way_data.has_closed_path();

            let mut pixeled_boundaries = HashSet::new();
            let mut pixeled_min_x_coordinates: (u32, u32) = (u32::MAX, u32::MAX);
            let mut pixeled_max_x_coordinates: (u32, u32) = (u32::MIN, u32::MIN);

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
                        let image_x = (y - min_y + 4) as u32;
                        let image_y = image_width - 4 - (x - min_x) as u32;

                        image_buffer.put_pixel(
                            image_x,
                            image_y,
                            image::Rgba([way_color[0], way_color[1], way_color[2], 255]),
                        );

                        if is_closed_way {
                            pixeled_boundaries.insert((image_x, image_y));

                            // speedup "are we inside the drawn way boundaries?"
                            if image_x > pixeled_max_x_coordinates.0 {
                                pixeled_max_x_coordinates = (image_x, image_y);
                            }

                            if image_x < pixeled_min_x_coordinates.0 {
                                pixeled_min_x_coordinates = (image_x, image_y);
                            }
                        }
                    }
                } else {
                    unreachable!();
                }
            }

            // flood fill the closed way using the algorithm specified here: https://en.wikipedia.org/wiki/Flood_fill#Span_Filling
            if is_closed_way {
                fn is_inside(
                    (mut x, y): (i64, i64),
                    min_x: i64,
                    max_x: i64,
                    pixeled_boundaries: &HashSet<(u32, u32)>,
                ) -> bool {
                    if x < min_x {
                        return false;
                    }

                    let mut intersected_boundary_times: u8 = 0;

                    loop {
                        if x > max_x {
                            break;
                        }
                        x += 1;

                        if pixeled_boundaries
                            .contains(&(x.try_into().unwrap(), y.try_into().unwrap()))
                        {
                            intersected_boundary_times += 1;
                        }
                    }

                    intersected_boundary_times % 2 != 0
                }

                fn get_flood_filled_pixels(
                    (mut x, y): (i64, i64),
                    min_x: i64,
                    max_x: i64,
                    pixeled_boundaries: &HashSet<(u32, u32)>,
                ) -> Vec<(u32, u32)> {
                    if !is_inside((x, y), min_x, max_x, pixeled_boundaries) {
                        warn!("initial value is not inside the boundaries");
                        return Vec::new();
                    }

                    info!("Flood filling polynom…");

                    let mut flood_filled_pixels: Vec<(u32, u32)> = Vec::new();

                    let mut stack: Vec<(i64, i64, i64, i64)> = Vec::new();

                    stack.push((x, x, y, 1));
                    stack.push((x, x, y - 1, -1));

                    while let Some((mut x1, x2, y, dy)) = stack.pop() {
                        x = x1;

                        debug!("Substep 1, stack size: {}", stack.len());

                        if is_inside((x, y), min_x, max_x, pixeled_boundaries) {
                            while x > 1 && is_inside((x - 1, y), min_x, max_x, pixeled_boundaries) {
                                flood_filled_pixels
                                    .push(((x - 1).try_into().unwrap(), y.try_into().unwrap()));

                                x -= 1;
                            }
                        }

                        if x < x1 {
                            stack.push((x, x1 - 1, y - dy, -dy));
                        }

                        debug!("Substep 2, stack size: {}", stack.len());

                        while x1 < x2 {
                            while is_inside((x1, y), min_x, max_x, pixeled_boundaries) {
                                flood_filled_pixels
                                    .push((x1.try_into().unwrap(), y.try_into().unwrap()));

                                x1 += 1;
                            }

                            stack.push((x, x1 - 1, y + dy, dy));

                            if x1 - 1 > x2 {
                                stack.push((x2 + 1, x1 - 1, y - dy, -dy));
                            }

                            while x1 < x2 && !is_inside((x1, y), min_x, max_x, pixeled_boundaries) {
                                x1 += 1;
                            }

                            x = x1;
                        }

                        if stack.len() > 15 {
                            warn!("stack size exploded, not filling polynom");
                            return Vec::new();
                        }
                    }

                    info!("Filled polynom with {} pixels", flood_filled_pixels.len());

                    flood_filled_pixels
                }

                for flood_filled_pixel in get_flood_filled_pixels(
                    (
                        (pixeled_max_x_coordinates.0 - 1).into(),
                        pixeled_max_x_coordinates.1.into(),
                    ),
                    pixeled_min_x_coordinates.1.into(),
                    pixeled_max_x_coordinates.1.into(),
                    &pixeled_boundaries,
                ) {
                    image_buffer.put_pixel(
                        flood_filled_pixel.0,
                        flood_filled_pixel.1,
                        image::Rgba([way_color[0], way_color[1], way_color[2], 255]),
                    );
                }
            }

            rendered_ways += 1;

            if rendered_ways % 15000 == 0 {
                info!("{} ways rendered…", rendered_ways);
            }
        }

        let render_duration = render_start_instant.elapsed();

        info!(
            "Rendering took {:.2}s. {:.2} ways/sec. Saving …",
            render_duration.as_secs_f32(),
            rendered_ways as f64 / (render_duration.as_nanos() as f64 * 1e-9)
        );

        dbg!(image_height);
        dbg!(image_width);

        let filename = String::from("test.png");

        image_buffer.save(&filename).unwrap();

        info!("Image saved successfully.");

        filename
    }
}
