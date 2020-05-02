use crate::data::ElementID;
use crate::data::{NodeData, RelationData, RelationMember, RelationMemberType, WayData};
use crate::element::canvas::CanvasElement;
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToFloatValue,
};
use crate::mapcss::selectors::{Selector, SelectorCondition, SelectorType};
use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;
use std::f64;
use std::num::NonZeroI64;
use std::time::Instant;
// fix for rust-analyzer being able to provide suggestions
#[allow(unused)]
use piston_window::*;
use std::collections::HashMap;

pub struct Gui {
    gl: GlGraphics,
    canvas: CanvasElement,
    ast: MapCssDeclarationList,
    zoom_level: f64,

    min_lat: f64,
    max_lat: f64,

    min_lon: f64,
    max_lon: f64,

    nid_to_node_data: HashMap<NonZeroI64, NodeData>,
    wid_to_way_data: HashMap<NonZeroI64, WayData>,
    rid_to_relation_data: HashMap<NonZeroI64, RelationData>,

    draw_lines_default: bool,
    mouse_movement_start: [f64; 2],
    mouse_movement_relative: [f64; 2],
    mouse_button_state: ButtonState,
}

impl Gui {
    pub fn new(
        gl: GlGraphics,
        canvas: CanvasElement,
        ast: MapCssDeclarationList,
        nid_to_node_data: HashMap<NonZeroI64, NodeData>,
        wid_to_way_data: HashMap<NonZeroI64, WayData>,
        rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
        zoom_level: f64,
    ) -> Gui {
        assert!(!nid_to_node_data.is_empty());
        assert!(!ast.is_empty());

        let mut min_lat: f64 = f64::INFINITY;
        let mut max_lat: f64 = f64::MIN;

        let mut min_lon: f64 = f64::INFINITY;
        let mut max_lon: f64 = f64::MIN;

        for (lat, lon) in nid_to_node_data.values().map(|x| (x.lat, x.lon)) {
            min_lat = min_lat.min(lat);
            max_lat = max_lat.max(lat);

            min_lon = min_lon.min(lon);
            max_lon = max_lon.max(lon);
        }

        Gui {
            draw_lines_default: dbg!(ast
                .search_or_default(
                    &SelectorType::Canvas,
                    &SelectorCondition::No,
                    &MapCssDeclarationProperty::DefaultLines,
                    &MapCssDeclarationValueType::Boolean(true),
                )
                .to_bool()),
            gl,
            canvas,
            ast,
            nid_to_node_data,
            wid_to_way_data,
            rid_to_relation_data,
            zoom_level,
            min_lat,
            max_lat,
            min_lon,
            max_lon,
            mouse_movement_relative: [0.0, 0.0],
            mouse_movement_start: [0.0, 0.0],
            mouse_button_state: ButtonState::Release,
        }
    }
}

impl Gui {
    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        let render_start = Instant::now();

        let background_color = self.canvas.background_color(&self.ast).into();

        const WAY_COLOR: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

        let mut context = self.gl.draw_begin(args.viewport());

        context = context
            .trans(
                self.mouse_movement_relative[0],
                self.mouse_movement_relative[1],
            )
            .zoom(self.zoom_level)
            .store_view();

        clear(background_color, &mut self.gl);

        let node_data = &self.nid_to_node_data;

        for way in self.wid_to_way_data.values().take(100_000) {
            if !way
                .tags()
                .contains(&("highway".to_owned(), "motorway".to_owned()))
            {
                continue;
            }

            let refs = way
                .refs()
                .iter()
                .map(|x| node_data.get(&x).unwrap())
                .collect::<Vec<_>>();
            let line_width = self
                .ast
                .search_or_default(
                    &SelectorType::Way,
                    &SelectorCondition::No,
                    &MapCssDeclarationProperty::Width,
                    if self.draw_lines_default {
                        &MapCssDeclarationValueType::Float(1.0_f64)
                    } else {
                        &MapCssDeclarationValueType::Float(0.0_f64)
                    },
                )
                .to_float();

            if line_width <= 0.0_f64 {
                continue;
            }

            for window in refs.windows(2) {
                if let [node_a, node_b] = window {
                    line_from_to(
                        WAY_COLOR,
                        line_width,
                        [node_a.lat - self.min_lat, node_a.lon - self.min_lon],
                        [node_b.lat - self.min_lat, node_b.lon - self.min_lon],
                        context.transform,
                        &mut self.gl,
                    );
                }
            }
        }

        self.gl.draw_end();
        // dbg!(render_start.elapsed());
    }

    pub fn mouse_scroll(&mut self, args: &[f64; 2]) {
        self.zoom_level += args[1] * 4.;
    }

    pub fn mouse_move(&mut self, args: &[f64; 2]) {
        match self.mouse_button_state {
            ButtonState::Press => {
                if &self.mouse_movement_start == args {
                    return;
                }

                self.mouse_movement_relative = [
                    self.mouse_movement_start[0] - args[0],
                    self.mouse_movement_start[1] - args[1],
                ];
            }
            ButtonState::Release => {
                self.mouse_movement_start = *args;
            }
        }
    }

    pub fn mouse_button(&mut self, args: &ButtonArgs) {
        if args.button == Button::Mouse(MouseButton::Left) {
            self.mouse_button_state = args.state;
        }
    }
}
