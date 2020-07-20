use crate::data::{NodeData, RelationData, WayData};
use crate::element::canvas::CanvasElement;
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToColorValue, ToFloatValue, RGBA,
};
use std::cmp;
use std::collections::HashMap;
use std::num::NonZeroI64;
use std::time::Instant;

pub trait Painter {
    /// Paints the given data styled by the mapcss ast and returns the filename of the saved file.
    fn paint(
        &mut self,
        image_resolution_factor: f32,
        mapcss_ast: MapCssDeclarationList,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
    ) -> String;
}

#[derive(Default)]
pub struct PngPainter {}

impl PngPainter {
    pub fn paint(
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

        dbg!(min_x, max_x, min_y, max_y);

        let image_width =
            crate::round_up_to((((max_x - min_x) as f64).abs()) as u32, IMAGE_PART_SIZE);
        let image_height =
            crate::round_up_to((((max_y - min_y) as f64).abs()) as u32, IMAGE_PART_SIZE);

        let background_color: image::Rgba<u8> = canvas.background_color(&mapcss_ast).into();

        let mut image_buffer: image::RgbaImage =
            image::ImageBuffer::from_pixel(image_height, image_width, background_color);

        let render_start_instant = Instant::now();

        #[derive(Debug, Copy, Clone)]
        struct AccumulatedPixelColor {
            pub red: u16,
            pub green: u16,
            pub blue: u16,
            pub alpha: u16,

            pub pixel_count: u16,
        }

        let mut pixel_accumulator: Vec<Vec<AccumulatedPixelColor>> = vec![
            vec![
                    AccumulatedPixelColor {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,

                        pixel_count: 0,
                    };
                    image_height as usize
                ];
            image_width as usize
        ];

        for (_way_id, way_data) in wid_to_way_data.into_iter().take(DRAWN_WAYS) {
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

                        let accumulated_pixel: &mut AccumulatedPixelColor;

                        debug_assert!(image_x <= image_height && image_y <= image_width);

                        // Safe as we initalize the array with all possible values
                        unsafe {
                            accumulated_pixel = pixel_accumulator
                                .get_unchecked_mut(image_y as usize)
                                .get_unchecked_mut(image_x as usize);
                        }

                        accumulated_pixel.pixel_count += 1;

                        accumulated_pixel.red += way_color[0] as u16;
                        accumulated_pixel.green += way_color[1] as u16;
                        accumulated_pixel.blue += way_color[2] as u16;
                        accumulated_pixel.alpha += 255;
                    }
                } else {
                    panic!();
                }
            }

            rendered_ways += 1;

            if rendered_ways % 10000 == 0 {
                println!("{}…", rendered_ways);
            }
        }

        image_buffer
            .enumerate_pixels_mut()
            .for_each(|(x, y, pixel)| {
                let accumulated_pixel: &AccumulatedPixelColor;

                unsafe {
                    accumulated_pixel = pixel_accumulator
                        .get_unchecked(y as usize)
                        .get_unchecked(x as usize);
                }

                if accumulated_pixel.pixel_count != 0 {
                    pixel[0] = (accumulated_pixel.red / accumulated_pixel.pixel_count) as u8;
                    pixel[1] = (accumulated_pixel.green / accumulated_pixel.pixel_count) as u8;
                    pixel[2] = (accumulated_pixel.blue / accumulated_pixel.pixel_count) as u8;
                    pixel[3] = (accumulated_pixel.alpha / accumulated_pixel.pixel_count) as u8;
                }
            });

        let render_duration = render_start_instant.elapsed();

        info!(
            "Rendering took {:.2?}. {:.2} ways/sec",
            render_duration,
            rendered_ways as f64 / (render_duration.as_nanos() as f64 * 1e-9)
        );

        let filename = String::from("test.png");

        image_buffer.save(&filename).unwrap();

        filename
    }
}
