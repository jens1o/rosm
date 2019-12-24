use crate::data::{NodeData, RelationData, ToNodeRef, WayData};
use crate::mapcss::style::Size;
use crate::mapcss::{MapCssPropertyDeclaration, MapCssRule};
use kuchiki::{Node, NodeDataRef, NodeRef};
use markup5ever::{Namespace, QualName};
use rayon::prelude::*;
use std::cmp::{self, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::num::NonZeroI64;
use std::time::{self, Instant};

#[derive(Clone, Copy)]
pub struct CanvasRenderStyle {
    background_color: cssparser::RGBA,
}

impl Default for CanvasRenderStyle {
    fn default() -> Self {
        CanvasRenderStyle {
            background_color: cssparser::RGBA::new(0, 0, 0, 0),
        }
    }
}

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
        rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
        mapcss_rules: Vec<MapCssRule>,
    ) -> String;
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum ElementIDType {
    Way(NonZeroI64),
    Node(NonZeroI64),
    Relation(i64),
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum RelationOrWayID {
    Relation(NonZeroI64),
    Way(NonZeroI64),
}

impl RelationOrWayID {
    pub fn id(&self) -> NonZeroI64 {
        use RelationOrWayID::*;
        match *self {
            Relation(id) => id,
            Way(id) => id,
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum RenderPixelType {
    /// This pixel represents a filling for the given (closed) way ID
    Filling(RelationOrWayID),
    /// Represents a pixel of the border of this (either closed or open) way
    WayBorder(NonZeroI64),
    /// This is just a single node referencing its ID
    Node(NonZeroI64),
}

impl RenderPixelType {
    pub fn id(&self) -> ElementIDType {
        use RenderPixelType::*;
        match *self {
            Node(id) => ElementIDType::Node(id),
            WayBorder(id) => ElementIDType::Node(id),
            Filling(id) => ElementIDType::Way(id.id()),
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Copy, Clone)]
struct RenderPixel {
    // DO NOT change the order of the struct properties, as they are relevant to the ordering
    // we want the highest z-index only
    pub z_index: u16,
    pub render_type: RenderPixelType,
    pub x: u32,
    pub y: u32,
}

#[derive(Default)]
pub struct PngPainter {}

impl Painter for PngPainter {
    fn paint(
        &mut self,
        image_resolution_factor: f64,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
        mapcss_rules: Vec<MapCssRule>,
    ) -> String {
        // make a good guess for all the pixels
        let mut pixels: Vec<RenderPixel> =
            Vec::with_capacity((wid_to_way_data.len() + nid_to_node_data.len()) * 12);
        let mut id_to_render_style: HashMap<ElementIDType, RenderStyle> =
            HashMap::with_capacity(nid_to_node_data.len());

        println!("Preparing to draw the ways");
        let instant = Instant::now();

        // TODO: Refactor into several methods
        for way in wid_to_way_data.values() {
            let render_style = get_render_style_for_element(way, &mapcss_rules);

            // way is invisible, so we don't need to calculate its pixels
            if render_style.width.0 < 1.0 {
                continue;
            }

            id_to_render_style.insert(ElementIDType::Way(way.wid), render_style);

            // add the nodes to the same render style as this way itself
            let nodes = way
                .refs
                .iter()
                .inspect(|nid| {
                    id_to_render_style.insert(ElementIDType::Node(**nid), render_style);
                })
                .map(|nid| nid_to_node_data.get(nid).unwrap())
                .collect::<Vec<_>>();

            let mut wall_pixels: Vec<RenderPixel> = Vec::new();

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
                            render_type: if render_style.width.0 < 1.0 {
                                RenderPixelType::WayBorder(way.wid)
                            } else {
                                RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                            },
                            z_index: render_style.z_index,
                            x,
                            y,
                        });

                        // true when we are at the most outern position
                        let mut is_border = true;

                        while distance_to_origin_pixel > 1 {
                            distance_to_origin_pixel -= 1;
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x: x + distance_to_origin_pixel,
                                y,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x,
                                y: y + distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x: x - distance_to_origin_pixel,
                                y,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x,
                                y: y - distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x: x - distance_to_origin_pixel,
                                y: y - distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x: x - distance_to_origin_pixel,
                                y: y + distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x: x + distance_to_origin_pixel,
                                y: y - distance_to_origin_pixel,
                            });
                            pixels.push(RenderPixel {
                                render_type: if is_border {
                                    RenderPixelType::WayBorder(node_a.nid)
                                } else {
                                    RenderPixelType::Filling(RelationOrWayID::Way(way.wid))
                                },
                                z_index: render_style.z_index,
                                x: x + distance_to_origin_pixel,
                                y: y + distance_to_origin_pixel,
                            });

                            if is_border {
                                is_border = false;
                            }
                        }
                    }
                } else {
                    panic!("Windows iterator does not deliver expected size!");
                }
            });

            // fill the way if the way is closed
            if way.is_closed() {
                let RenderPixel { y, .. } = &wall_pixels.iter().next().unwrap_or_else(|| {
                    panic!("At least one pixel needs to be drawn (way #{})?!", way.wid)
                });

                let mut y_min = y;
                let mut y_max = y;

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
                                    render_type: RenderPixelType::Filling(RelationOrWayID::Way(
                                        way.wid,
                                    )),
                                    z_index: render_style.z_index,
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

        println!(
            "Finished preparing of ways in {:.2?}. Now preparing to draw any other node.",
            instant.elapsed()
        );
        nid_to_node_data
            .values()
            // Filter out nodes we've already drawn through the ways
            .filter(|node_data| node_data.way.is_none())
            .for_each(|node_data| {
                // match mapcss rules
                let render_style = get_render_style_for_element(node_data, &mapcss_rules);

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
                    pixels.push(RenderPixel {
                        render_type: RenderPixelType::Node(node_data.nid),
                        z_index: render_style.z_index,
                        x,
                        y,
                    });

                    pixels.extend(get_distance_to_origin_pixels(
                        (x, y),
                        render_style.width.0 as u32,
                        RenderPixelType::Node(node_data.nid),
                        render_style.z_index,
                    ));

                    id_to_render_style.insert(ElementIDType::Node(node_data.nid), render_style);
                }
            });

        let RenderPixel { x, y, .. } = pixels
            .iter()
            .copied()
            .next()
            .expect("At least one pixel needs to be drawn!");

        let mut min_x = x;
        let mut max_x = x;
        let mut min_y = y;
        let mut max_y = y;

        let instant = Instant::now();
        for RenderPixel { x, y, .. } in pixels.iter() {
            min_x = cmp::min(*x, min_x);
            max_x = cmp::max(*x, max_x);

            min_y = cmp::min(*y, min_y);
            max_y = cmp::max(*y, max_y);
        }

        dbg!(min_x, max_x);
        dbg!(min_y, max_y);
        println!("Finding the min/max values took {:.2?}", instant.elapsed());

        // sort pixels by z-index (ascending)
        println!("Now sorting by z-index.");
        let instant = Instant::now();
        // TODO: Remove all pixels where another pixel on the same position has a higher z-index
        pixels.par_sort_unstable_by(|a, b| a.z_index.cmp(&b.z_index));
        println!("Sorting by z-index took {:.2?}", instant.elapsed());

        let image_width = crate::round_up_to((max_x - min_x) + 1, crate::IMAGE_PART_SIZE);
        let image_height = crate::round_up_to((max_y - min_y) + 1, crate::IMAGE_PART_SIZE);

        dbg!(image_width, image_height);

        // fetch canvas style data
        let mut canvas_render_style = CanvasRenderStyle::default();

        let canvas_mapcss = NodeDataRef::new_opt(
            NodeRef::new_element(
                QualName::new(None, Namespace::from(""), "canvas".into()),
                vec![], // TODO: Maybe add some arguments?
            ),
            Node::as_element,
        )
        .unwrap();

        for mapcss_rule in mapcss_rules.iter() {
            if mapcss_rule.original_rule.selectors.matches(&canvas_mapcss) {
                for rule_declaration in &mapcss_rule.original_rule.declarations {
                    use MapCssPropertyDeclaration::*;

                    match rule_declaration {
                        BackgroundColor(bg_color) => {
                            canvas_render_style.background_color = *bg_color
                        }
                        _ => panic!("Only background-color may be set for the canvas element!"),
                    }
                }
            }
        }

        // order is changed to account for rotating by 270 degrees
        let mut image = image::ImageBuffer::from_pixel(
            image_height,
            image_width,
            image::Rgb([
                canvas_render_style.background_color.red,
                canvas_render_style.background_color.green,
                canvas_render_style.background_color.blue,
            ]),
        );

        println!("Now drawing {} pixels to canvas.", pixels.len());
        let instant = Instant::now();

        for (id, render_pixel, pixel_x, pixel_y) in pixels
            .iter()
            .map(|pixel| {
                (
                    pixel.render_type.id(),
                    pixel,
                    pixel.x - min_x,
                    pixel.y - min_y,
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

#[inline]
fn get_render_style_for_element<T: ToNodeRef>(
    element: &T,
    mapcss_rules: &Vec<MapCssRule>,
) -> RenderStyle {
    let mut render_style = RenderStyle::default();

    let way_ref_data = NodeDataRef::new_opt(element.node_ref(), Node::as_element).unwrap();

    for mapcss_rule in mapcss_rules.iter() {
        if mapcss_rule.original_rule.selectors.matches(&way_ref_data) {
            for rule_declaration in &mapcss_rule.original_rule.declarations {
                use MapCssPropertyDeclaration::*;

                match rule_declaration {
                    Color(color) => render_style.color = *color,
                    FillColor(fill_color) => render_style.fill_color = *fill_color,
                    Width(width) => render_style.width = *width,
                    ZIndex(z_index) => render_style.z_index = *z_index,
                    BackgroundColor(_) => panic!(
                        "Cannot declare background-color on this element, use fill-color instead!"
                    ),
                }
            }
        }
    }

    render_style
}

#[inline(always)]
fn get_distance_to_origin_pixels(
    (start_x, start_y): (u32, u32),
    mut distance_to_origin_pixel: u32,
    render_pixel_type: RenderPixelType,
    z_index: u16,
) -> Vec<RenderPixel> {
    let mut pixels = Vec::with_capacity(distance_to_origin_pixel as usize * 8);

    while distance_to_origin_pixel > 1 {
        distance_to_origin_pixel -= 1;
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x + distance_to_origin_pixel,
            y: start_y,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x,
            y: start_y + distance_to_origin_pixel,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x - distance_to_origin_pixel,
            y: start_y,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x,
            y: start_y - distance_to_origin_pixel,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x - distance_to_origin_pixel,
            y: start_y - distance_to_origin_pixel,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x - distance_to_origin_pixel,
            y: start_y + distance_to_origin_pixel,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x + distance_to_origin_pixel,
            y: start_y - distance_to_origin_pixel,
        });
        pixels.push(RenderPixel {
            render_type: render_pixel_type.clone(),
            z_index,
            x: start_x + distance_to_origin_pixel,
            y: start_y + distance_to_origin_pixel,
        });
    }

    pixels
}
