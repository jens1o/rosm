use crate::data::{NodeData, ToNodeRef, WayData};
use crate::mapcss::style::Size;
use crate::mapcss::{MapCssPropertyDeclaration, MapCssRule};
use kuchiki::{Node, NodeDataRef};
use rayon::prelude::*;
use std::cmp::{self, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::num::NonZeroI64;
use std::time::{self, Instant};

#[derive(Clone, Copy)]
pub struct RenderStyle {
    z_index: u16,
    color: cssparser::RGBA,
    fill_color: cssparser::RGBA,
    width: Size,
}

impl Default for RenderStyle {
    fn default() -> Self {
        RenderStyle {
            color: cssparser::RGBA::new(255, 255, 255, 0),
            fill_color: cssparser::RGBA::new(0, 0, 0, 0),
            z_index: 0,
            width: Size(1.0),
        }
    }
}

pub trait Painter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        mapcss_rules: Vec<MapCssRule>,
    ) -> String;
}

#[derive(Default)]
pub struct PngPainter {}

impl Painter for PngPainter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        mapcss_rules: Vec<MapCssRule>,
    ) -> String {
        #[derive(PartialEq, Eq, Hash, Debug)]
        enum ElementIDType {
            Way(NonZeroI64),
            Node(NonZeroI64),
            // TODO: Relation(i64),
        };
        enum RenderPixelType {
            /// This is just a single node referencing its ID
            Node(NonZeroI64),
            /// Represents a pixel of the border of this (either closed or open) way
            WayBorder(NonZeroI64),
            /// This pixel represents a filling for the given (closed) way ID
            Filling(NonZeroI64),
        }

        impl RenderPixelType {
            pub fn id(&self) -> ElementIDType {
                use RenderPixelType::*;
                match *self {
                    Node(id) => ElementIDType::Node(id),
                    WayBorder(id) => ElementIDType::Node(id),
                    Filling(id) => ElementIDType::Way(id),
                }
            }
        }

        struct RenderPixel {
            pub render_type: RenderPixelType,
            pub x: u32,
            pub y: u32,
        }

        // make a good guess for all the pixels
        let mut pixels: Vec<RenderPixel> =
            Vec::with_capacity((wid_to_way_data.len() + nid_to_node_data.len()) * 12);
        // TODO: Refactor into two lists as the IDs are not unique (both a way and a node MAY have the same ID)
        let mut id_to_render_style: HashMap<ElementIDType, RenderStyle> =
            HashMap::with_capacity(nid_to_node_data.len());

        println!("Preparing to draw the ways");

        // TODO: Refactor into several methods
        for way in wid_to_way_data.values() {
            if way.refs.len() < 2 {
                // ignore invalid ways
                continue;
            }

            let way_ref_data = NodeDataRef::new_opt(way.node_ref(), Node::as_element).unwrap();
            let mut render_style = RenderStyle::default();

            for mapcss_rule in mapcss_rules.iter() {
                if mapcss_rule.original_rule.selectors.matches(&way_ref_data) {
                    for rule_declaration in &mapcss_rule.original_rule.declarations {
                        use MapCssPropertyDeclaration::*;

                        match rule_declaration {
                            Color(color) => render_style.color = *color,
                            FillColor(fill_color) => render_style.fill_color = *fill_color,
                            Width(width) => render_style.width = *width,
                            ZIndex(z_index) => render_style.z_index = *z_index,
                        }
                    }
                }
            }

            // way is invisible, so we don't need to calculate its pixels
            if render_style.width.0 <= 0.0 {
                continue;
            }

            id_to_render_style.insert(ElementIDType::Way(way.wid), render_style);

            let nodes = way
                .refs
                .iter()
                .inspect(|nid| {
                    id_to_render_style.insert(ElementIDType::Node(**nid), render_style);
                })
                .map(|nid| nid_to_node_data.get(nid).unwrap())
                .collect::<Vec<_>>();

            let mut wall_pixels = Vec::new();

            nodes[..].windows(2).for_each(|node_list| {
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

                        wall_pixels.push(RenderPixel {
                            render_type: RenderPixelType::WayBorder(node_a.nid),
                            x,
                            y,
                        });

                        // TODO: Don't draw an X, but fill everything out and make it solid
                        // by making it kind of like filling an area
                        while distance_to_origin_pixel > 1 {
                            distance_to_origin_pixel -= 1;
                            pixels.push(RenderPixel {
                                render_type: RenderPixelType::WayBorder(node_a.nid),
                                x: x + distance_to_origin_pixel,
                                y: y + distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: RenderPixelType::WayBorder(node_a.nid),
                                x: x + distance_to_origin_pixel,
                                y: y - distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: RenderPixelType::WayBorder(node_a.nid),
                                x: x - distance_to_origin_pixel,
                                y: y + distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: RenderPixelType::WayBorder(node_a.nid),
                                x: x - distance_to_origin_pixel,
                                y: y - distance_to_origin_pixel,
                            });
                        }
                    }
                } else {
                    panic!("Windows iterator does not deliver expected size!");
                }
            });

            // fill the way if the way is closed
            if way.is_closed() {
                let RenderPixel { y: y_sample, .. } =
                    &wall_pixels.iter().next().unwrap_or_else(|| {
                        panic!("At least one pixel needs to be drawn (way #{})!", way.wid)
                    });

                let mut y_min = y_sample;
                let mut y_max = y_sample;

                let mut intersection_y_points: HashMap<u32, BinaryHeap<Reverse<u32>>> =
                    HashMap::new();

                for RenderPixel { x, y, .. } in wall_pixels.iter() {
                    if y_max < y {
                        y_max = y;
                    }

                    if y_min > y {
                        y_min = y;
                    }

                    intersection_y_points
                        .entry(*y)
                        .or_default()
                        .push(Reverse(*x));
                }

                debug_assert!(y_max > y_min);

                let mut y = *y_min;

                while &y < y_max {
                    // swapped as soon as an intersection is met
                    let mut is_drawing = true;

                    let current_heap = intersection_y_points
                        .get_mut(&y)
                        .expect("No intersection points?!");

                    let Reverse(mut x) = match current_heap.pop() {
                        Some(x) => x,
                        None => break,
                    };

                    while let Some(Reverse(next_intersection)) = current_heap.pop() {
                        loop {
                            if is_drawing {
                                pixels.push(RenderPixel {
                                    render_type: RenderPixelType::Filling(way.wid),
                                    x,
                                    y,
                                });
                            }

                            if x == next_intersection {
                                is_drawing = !is_drawing;
                                break;
                            }

                            x += 1;
                        }
                    }

                    y += 1;
                }
            }

            pixels.extend(wall_pixels);
        }

        println!("Finished preparing of ways. Now preparing to draw any other nodes.");
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
                                Color(color) => render_style.color = *color,
                                FillColor(fill_color) => render_style.fill_color = *fill_color,
                                Width(width) => render_style.width = *width,
                                ZIndex(z_index) => render_style.z_index = *z_index,
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

                    pixels.push(RenderPixel {
                        render_type: RenderPixelType::Node(node_data.nid),
                        x,
                        y,
                    });

                    while distance_to_origin_pixel > 1 {
                        distance_to_origin_pixel -= 1;
                        pixels.push(RenderPixel {
                            render_type: RenderPixelType::Node(node_data.nid),
                            x: x + distance_to_origin_pixel,
                            y: y + distance_to_origin_pixel,
                        });
                        pixels.push(RenderPixel {
                            render_type: RenderPixelType::Node(node_data.nid),
                            x: x + distance_to_origin_pixel,
                            y: y - distance_to_origin_pixel,
                        });
                        pixels.push(RenderPixel {
                            render_type: RenderPixelType::Node(node_data.nid),
                            x: x - distance_to_origin_pixel,
                            y: y + distance_to_origin_pixel,
                        });
                        pixels.push(RenderPixel {
                            render_type: RenderPixelType::Node(node_data.nid),
                            x: x - distance_to_origin_pixel,
                            y: y - distance_to_origin_pixel,
                        });
                    }
                }

                id_to_render_style.insert(ElementIDType::Node(node_data.nid), render_style);
            });

        // sort pixels by z-index (ascending)
        println!("Now sorting by z-index.");
        let instant = Instant::now();
        // TODO: Remove all pixels where another pixel on the same position has a higher z-index
        pixels.par_sort_unstable_by(|a, b| {
            let a_style_data = id_to_render_style
                .get(&a.render_type.id())
                .unwrap_or_else(|| {
                    panic!(
                        "No render style found for element {:?} when comparing z-indexes!",
                        a.render_type.id()
                    )
                });

            let b_style_data = id_to_render_style
                .get(&b.render_type.id())
                .unwrap_or_else(|| {
                    panic!(
                        "No render style found for element {:?} when comparing z-indexes!",
                        b.render_type.id()
                    )
                });

            a_style_data.z_index.cmp(&b_style_data.z_index)
        });
        println!("Sorting by z-index took {:.2?}", instant.elapsed());

        let RenderPixel {
            x: x_sample,
            y: y_sample,
            ..
        } = &pixels
            .iter()
            .next()
            .expect("At least one pixel needs to be drawn!");

        let mut min_x = x_sample;
        let mut max_x = x_sample;
        let mut min_y = y_sample;
        let mut max_y = y_sample;

        let instant = Instant::now();
        for RenderPixel { x, y, .. } in pixels.iter() {
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

        println!("Now drawing {} pixels to canvas.", pixels.len());
        let instant = Instant::now();

        for (id, render_pixel, pixel_x, pixel_y) in pixels
            .iter()
            .map(|render_pixel| {
                (
                    render_pixel.render_type.id(),
                    render_pixel,
                    render_pixel.x - min_x,
                    render_pixel.y - min_y,
                )
            })
            // rotate by 270 degress
            .map(|(id, render_pixel, pixel_x, pixel_y)| {
                (id, render_pixel, pixel_y, image_width - 1 - pixel_x)
            })
        {
            let render_style = id_to_render_style
                .get(&id)
                .expect("No render style found for element!");

            let r;
            let g;
            let b;

            if let RenderPixelType::Filling(_) = render_pixel.render_type {
                let cssparser::RGBA {
                    red, green, blue, ..
                } = render_style.fill_color;

                r = red;
                g = green;
                b = blue;
            } else {
                r = render_style.color.red;
                g = render_style.color.green;
                b = render_style.color.blue;
            }

            image.put_pixel(pixel_x, pixel_y, image::Rgb([r, g, b]));
        }

        println!("Drawing took {:.2?}", instant.elapsed());

        // release some memory asap
        drop(pixels);
        drop(id_to_render_style);

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
