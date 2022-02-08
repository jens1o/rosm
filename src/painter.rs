use crate::data::{NodeData, RelationData, WayData};
use crate::element::canvas::CanvasElement;
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToColorValue, ToIntegerValue, RGBA,
};
use crate::mapcss::parser::IntSize;
use std::cmp;
use std::collections::HashMap;
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
        const IMAGE_PADDING: u32 = IMAGE_PART_SIZE / 4;
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
                    }
                } else {
                    unreachable!();
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

        let filename = String::from("test.png");

        image_buffer.save(&filename).unwrap();

        info!("Image saved successfully.");

        filename
    }
}
