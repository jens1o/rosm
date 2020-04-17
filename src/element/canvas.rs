use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToColorValue,
    RGBA,
};
use crate::mapcss::selectors::{SelectorCondition, SelectorType};

pub struct CanvasElement {}

impl CanvasElement {
    pub fn background_color(&self, mapcss_declarations: &MapCssDeclarationList) -> RGBA {
        mapcss_declarations
            .search_or_default(
                &SelectorType::Canvas,
                // TODO: Calculate current conditions
                &SelectorCondition::No,
                &MapCssDeclarationProperty::BackgroundColor,
                &MapCssDeclarationValueType::Color(RGBA {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                }),
            )
            .to_color()
    }
}
