use crate::data::{ElementData, NodeData, RelationData, WayData};
use crate::element::canvas::CanvasElement;
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToColorValue,
    ToIntegerValue,
};
use crate::mapcss::parser::IntSize;
use std::cmp;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::num::NonZeroI64;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

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
                .unwrap_or(0);

            if way_width == 0 {
                continue;
            }

            let is_closed_way = way_data.has_closed_path();

            let mut pixeled_min_x_coordinates: (u32, u32) = (u32::MAX, u32::MAX);
            let mut pixeled_max_x_coordinates: (u32, u32) = (u32::MIN, u32::MIN);

            // draw the outline of the way and remember it
            let mut outline_pixels: HashMap<u32, BTreeSet<u32>> = HashMap::new();

            assert!(!way_refs.is_empty());

            for ref_node_ids in way_refs[..].windows(2) {
                let [node_a_id, node_b_id] = ref_node_ids else { unreachable!() };

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
                            outline_pixels.entry(image_y).or_default().insert(image_x);

                            // speedup "are we inside the drawn way boundaries?"
                            if image_x > pixeled_max_x_coordinates.0 {
                                pixeled_max_x_coordinates = (image_x, image_y);
                            }

                            if image_x < pixeled_min_x_coordinates.0 {
                                pixeled_min_x_coordinates = (image_x, image_y);
                            }
                        }
                    } else {
                        let start_x = if way_width / 2 > image_x {
                            0
                        } else {
                            image_x - way_width / 2
                        };
                        let end_x = (image_x + way_width / 2).min(image_width - 1);

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
                                outline_pixels.entry(image_y).or_default().insert(x);

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
            }

            // determine whether we can fill the area
            if is_closed_way
                && pixeled_min_x_coordinates != pixeled_max_x_coordinates
                && !are_image_coordinates_horizontally_next_to_each_other(
                    pixeled_min_x_coordinates,
                    pixeled_max_x_coordinates,
                )
            {
                let min_y = pixeled_min_x_coordinates.1.min(pixeled_max_x_coordinates.1);
                let max_y = pixeled_min_x_coordinates.1.max(pixeled_max_x_coordinates.1);

                if let Some((inner_x, inner_y)) = get_inside_point(min_y, max_y, &outline_pixels) {
                    if inner_x >= image_buffer.width() || inner_y >= image_buffer.height() {
                        panic!("inner point is bigger than the image's dimension(s)!");
                    }

                    for flood_filled_pixel in
                        get_flood_filled_pixels((inner_x, inner_y), &mut outline_pixels)
                    {
                        // validate results
                        if flood_filled_pixel.0 >= image_buffer.width()
                            || flood_filled_pixel.1 >= image_buffer.height()
                        {
                            warn!("Ignoring flood filled pixel that is out of the image canvas.");
                            continue;
                        }

                        flood_fill_pixel_count += 1;

                        image_buffer.put_pixel(
                            flood_filled_pixel.0,
                            flood_filled_pixel.1,
                            image::Rgb([way_fill_color[0], way_fill_color[1], way_fill_color[2]]),
                        );
                    }
                } else {
                    no_inside_count += 1;
                    // dbg!(pixeled_min_x_coordinates, pixeled_max_x_coordinates);
                }
            }
        }

        let render_duration = render_start_instant.elapsed();

        info!(
            "Rendering took {:.2} s. {:.2} ways/sec. Saving …",
            render_duration.as_secs_f32(),
            processed_ways as f64 / (render_duration.as_nanos() as f64 * 1e-9)
        );

        dbg!(image_height);
        dbg!(image_width);
        dbg!(flood_fill_pixel_count);

        let file_path = PathBuf::from(format!(
            "render-{}.png",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        let save_start_instant = Instant::now();

        image_buffer.save(&file_path).unwrap();

        info!(
            "Image saved successfully, took {:.2} s.",
            save_start_instant.elapsed().as_secs_f32()
        );
        if no_inside_count > 0 {
            warn!("couldn't find a starting point to flood fill {no_inside_count} time(s)");
        }

        file_path
    }
}

/// Determines whether the given point is inside the outline given by `outline_pixels`,
/// taking into consideration that there can be multiple outlines at the same level.
fn is_inside(image_point: &(u32, u32), outline_pixels: &HashMap<u32, BTreeSet<u32>>) -> bool {
    if let Some(outline_x_pixels) = outline_pixels.get(&image_point.1) {
        // TODO: Clarify what happens if we have something like this:
        // "x |", where x is the left-most pixel possible and hence the x is completely
        // bordered by the outline
        if outline_x_pixels.len() < 2 {
            return false;
        }

        // Expected results:
        // |   x  | => is inside
        // |  x => is not inside (? see TODO above) - tbd
        // x | => is not inside (? see TODO above) - tbd
        // |  |  x => is not inside
        // |  |   |  x  | => is inside
        // |  | x |     | => is not inside

        (outline_x_pixels.first().unwrap()..outline_x_pixels.last().unwrap())
            .contains(&&image_point.0)
            && !outline_x_pixels.contains(&image_point.0)
    } else {
        false
    }
}

fn are_image_coordinates_horizontally_next_to_each_other(a: (u32, u32), b: (u32, u32)) -> bool {
    if a.1 != b.1 {
        // not horizontally on the same line
        return false;
    }

    // check if they are next to each other
    a.0.abs_diff(b.0) == 1
}

fn get_inside_point(
    min_y: u32,
    max_y: u32,
    outline_pixels: &HashMap<u32, BTreeSet<u32>>,
) -> Option<(u32, u32)> {
    debug_assert!(min_y < max_y);

    for y in min_y..=max_y {
        // find hole in the outlined pixels at the height of the y
        if let Some(x_pixels) = outline_pixels.get(&y) {
            if x_pixels.len() < 2 {
                continue;
            }

            let mut last_x = x_pixels.first().unwrap();

            // Invariant: The iterator is sorted ascending.
            for x_pixel in x_pixels.iter().skip(1) {
                if x_pixel - last_x != 1 {
                    return Some((last_x + 1, y));
                }

                last_x = x_pixel;
            }
        }
    }

    None
}

/// Runs a flood fill implementation based on the given (x, y) point, filling it until hitting
/// the outline everywhere
fn get_flood_filled_pixels(
    (x, y): (u32, u32),
    outline_pixels: &HashMap<u32, BTreeSet<u32>>,
) -> HashSet<(u32, u32)> {
    // dbg!((x, y), &outline_pixels);

    if !is_inside(&(x, y), outline_pixels) {
        unreachable!("starting point must be inside");
    }

    let mut flood_filled_pixels = HashSet::new();

    // we will put at least 4 more items on the stack during the first loop (assuming that we are inside the outline),
    // take account for that
    // after that, it is not really possible before-hand to guess how much capacity we'll need overall
    let mut stack = Vec::with_capacity(4);

    // use given point as a starting point for flood filling
    stack.push((x, y));

    let mut iterations = 0;

    while let Some((x, y)) = stack.pop() {
        iterations += 1;

        if iterations == 100_000 {
            // TODO: Check whether there is a logic error somewhere that may result into an infinite loop
            warn!("Giving up flood filling to possibly prevent infinite loop");
            return flood_filled_pixels;
        }

        if flood_filled_pixels.contains(&(x, y)) || !is_inside(&(x, y), outline_pixels) {
            continue;
        }

        flood_filled_pixels.insert((x, y));

        stack.push((x, y + 1));
        stack.push((x, y - 1));
        stack.push((x + 1, y));
        stack.push((x - 1, y));
    }

    flood_filled_pixels
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use super::is_inside;

    #[test]
    fn test_is_inside_simple() {
        // ^
        // y
        // 4
        // 3
        // 2   | p |
        // 1
        // 0 1 2 3 4 x ->
        assert!(is_inside(
            &(3, 2),                                       // = p
            &HashMap::from([(2, BTreeSet::from([2, 4]))])  // = borders
        ));

        // ^
        // y
        // 4
        // 3
        // 2 p |   |
        // 1
        // 0 1 2 3 4 x ->
        assert!(!is_inside(
            &(1, 2),                                       // = p
            &HashMap::from([(2, BTreeSet::from([2, 4]))])  // = borders
        ));
    }

    #[test]
    fn test_is_inside_complex() {
        // ^
        // y
        // 4
        // 3 |     |
        // 2 q | p |
        // 1 | o |   | |
        // 0 1 2 3 4 5 6 x ->
        let outline_representation = HashMap::from([
            (1, BTreeSet::from([1, 3, 5, 6])),
            (2, BTreeSet::from([2, 4])),
            (3, BTreeSet::from([1, 4])),
        ]);
        let q = ((1, 2), false);
        let p = ((3, 2), true);
        let o = ((2, 1), true);

        for (coords, expected) in [q, p, o] {
            assert_eq!(is_inside(&coords, &outline_representation), expected);
        }
    }
}
