use crate::data::{NodeData, RelationData, RelationMember, RelationMemberType, WayData};
use crate::element::canvas::CanvasElement;
use crate::mapcss::declaration::MapCssDeclarationList;
use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;
use std::num::NonZeroI64;
// fix for rust-analyzer being able to provide suggestions
#[allow(unused)]
use piston_window::*;
use std::collections::HashMap;

pub struct Gui {
    pub gl: GlGraphics,
    pub canvas: CanvasElement,
    pub ast: MapCssDeclarationList,
    pub nid_to_node_data: HashMap<NonZeroI64, NodeData>,
    pub wid_to_way_data: HashMap<NonZeroI64, WayData>,
    pub rid_to_relation_data: HashMap<NonZeroI64, RelationData>,
    pub zoom_level: f64,
}

impl Gui {
    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let background_color = self.canvas.background_color(&self.ast).into();

        const WAY_COLOR: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);

        let context = self.gl.draw_begin(args.viewport());

        // Clear the screen.
        clear(background_color, &mut self.gl);
        let node_data = &self.nid_to_node_data;

        for way in self.wid_to_way_data.values() {
            let refs = way
                .refs
                .iter()
                .map(|x| node_data.get(&x).unwrap())
                .collect::<Vec<_>>();

            for window in refs.windows(2) {
                if let [node_a, node_b] = window {
                    line_from_to(
                        WAY_COLOR,
                        1.0_f64,
                        [node_a.lat, node_a.lon],
                        [node_b.lat, node_b.lon],
                        context.transform.zoom(self.zoom_level * 2_f64),
                        &mut self.gl,
                    );
                }
            }
        }

        self.gl.draw_end();
    }

    pub fn mouse_scroll(&mut self, args: &[f64; 2]) {
        self.zoom_level += args[1];
    }
}
