use crate::data::{ElementData, ElementID};
use crate::mapcss::declaration::{
    MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType, ToBooleanValue,
    ToColorValue, RGBA,
};

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

    fn has_closed_path(&self) -> bool {
        false
    }
}

impl CanvasElement {
    pub fn background_color(self, mapcss_declarations: &MapCssDeclarationList) -> RGBA {
        mapcss_declarations
            .search_or_default(
                Box::new(self),
                &MapCssDeclarationProperty::FillColor,
                &MapCssDeclarationValueType::Color(RGBA {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                }),
            )
            .to_color()
    }
}
