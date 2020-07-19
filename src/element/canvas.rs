use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToColorValue, RGBA,
};
use crate::mapcss::selectors::{SelectorCondition, SelectorType};

pub struct CanvasElement {}

impl CanvasElement {
    pub fn background_color(&self, mapcss_declarations: &MapCssDeclarationList) -> RGBA {
        todo!()
    }

    /// Returns true if all lines are drawn by default. Returns false when only those with a matching rule shall be drawn.
    pub fn draw_lines_by_default(&self, mapcss_declarations: &MapCssDeclarationList) -> bool {
        todo!()
    }
}
