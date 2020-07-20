use crate::data::{ElementData, ElementID};
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToColorValue, RGBA,
};
use crate::mapcss::selectors::{SelectorCondition, SelectorType};

#[derive(Debug)]
pub struct CanvasElement {}

impl Into<ElementID> for CanvasElement {
    fn into(self) -> ElementID {
        ElementID::Canvas
    }
}

impl ElementData for CanvasElement {
    fn tags(&self) -> &[(String, String)] {
        &[]
    }

    fn id(&self) -> ElementID {
        ElementID::Canvas
    }

    fn is_closed(&self) -> bool {
        false
    }
}

impl CanvasElement {
    pub fn background_color(self, mapcss_declarations: &MapCssDeclarationList) -> RGBA {
        mapcss_declarations
            .search_or_default(
                Box::new(self),
                &MapCssDeclarationProperty::BackgroundColor,
                &MapCssDeclarationValueType::Color(RGBA {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                }),
            )
            .to_color()
    }

    /// Returns true if all lines are drawn by default. Returns false when only those with a matching rule shall be drawn.
    pub fn draw_lines_by_default(&self, mapcss_declarations: &MapCssDeclarationList) -> bool {
        todo!()
    }
}
