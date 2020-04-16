use crate::element::canvas::CanvasElement;
use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;

pub struct Gui {
    pub gl: GlGraphics,
    pub canvas: CanvasElement,
}

impl Gui {
    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let background_color = self.canvas.background_color().into();
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(background_color, gl);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, c.transform, gl);
        });
    }
}
