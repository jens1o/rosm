use crate::mapcss::parser::{FloatSize, IntSize};
use crate::mapcss::selectors::{SelectorCondition, SelectorType};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

mod color;

pub use color::RGBA;

pub type MapCssDeclaration = (MapCssDeclarationProperty, MapCssDeclarationValueType);

pub trait ToFloatValue {
    fn to_float(&self) -> FloatSize;
}

pub trait ToColorValue {
    fn to_color(&self) -> RGBA;
}

#[derive(Debug, Clone)]
pub struct MapCssDeclarationList {
    declarations: HashMap<SelectorType, HashMap<SelectorCondition, Vec<MapCssDeclaration>>>,
}

// TODO: Add merge(MapCssDeclarationList) method merging the current list with the latter (latter wins) => cascading properties
// being used in the rendering process
impl MapCssDeclarationList {
    pub fn new(
        declarations: HashMap<SelectorType, HashMap<SelectorCondition, Vec<MapCssDeclaration>>>,
    ) -> MapCssDeclarationList {
        MapCssDeclarationList { declarations }
    }

    pub fn search_cascading_or_panic(
        &self,
        selector_type: &SelectorType,
        selector_condition: &SelectorCondition,
        declaration_property_name: &MapCssDeclarationProperty,
    ) -> &MapCssDeclarationValueType {
        if selector_condition != &SelectorCondition::No {
            todo!("Selector matching not yet implemented");
        }

        // TODO: Take SelectorCondition::Any into account as it applies to all elements

        self.declarations
            .get(selector_type)
            .and_then(|declaration_list| {
                declaration_list
                    .get(&SelectorCondition::No)
                    .and_then(|map_css_declaration_list| {
                        map_css_declaration_list
                            .iter()
                            .rfind(|(name, _value)| name == declaration_property_name)
                            .and_then(|(_name, value)| Some(value))
                    })
            })
            .unwrap_or_else(|| {
                panic!(
                    "Could not find required MapCSS declaration {:?} item!",
                    declaration_property_name
                );
            })
    }

    pub fn search_or_default<'a>(
        &'a self,
        selector_type: &SelectorType,
        selector_condition: &SelectorCondition,
        declaration_property_name: &MapCssDeclarationProperty,
        default: &'a MapCssDeclarationValueType,
    ) -> &'a MapCssDeclarationValueType {
        if selector_condition != &SelectorCondition::No {
            todo!("Selector matching not yet implemented");
        }

        self.declarations
            .get(&selector_type)
            .and_then(|declaration_list| {
                declaration_list
                    .get(&SelectorCondition::No)
                    .and_then(|map_css_declaration_list| {
                        map_css_declaration_list
                            .iter()
                            .rfind(|(name, _value)| name == declaration_property_name)
                            .and_then(|(_name, value)| Some(value))
                    })
            })
            .unwrap_or(default)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MapCssDeclarationProperty {
    // meta {}
    Title,
    Version,
    Description,
    Acknowledgement,

    /// whether to show all lines by default or only the ones that are styled
    /// only able to be used on the canvas element
    DefaultLines,
    /// whether to show all points by default or only the ones that are styled
    /// only able to be used on the canvas element
    DefaultPoints,

    Linecap,
    Linejoin,

    AllowOverlap,

    Dashes,

    Text,
    TextColor,
    TextPosition,
    TextHaloColor,
    TextHaloRadius,
    TextWrapWidth,
    TextSpacing,

    BackgroundColor,
    Color,
    FontSize,
    FontColor,
    FontFamily,

    IconImage,
    PatternImage,

    Opacity,
    FillOpacity,
    FillColor,
    FillImage,
    Width,

    ZIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MapCssDeclarationValueType {
    Boolean(bool),
    String(String),
    Color(RGBA),
    LinecapDeclarationVariant(LinecapDeclarationVariant),
    LinejoinDeclarationVariant(LinejoinDeclarationVariant),
    TextPositionDeclarationVariant(TextPositionDeclarationVariant),
    IntegerArray(Vec<IntSize>),
    Integer(IntSize),
    Float(FloatSize),
}

impl fmt::Display for MapCssDeclarationValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MapCssDeclarationValueType::*;

        match self {
            Boolean(boolean) => write!(f, "{}", if *boolean { "true" } else { "false" }),
            String(string) => write!(f, "{}", string),
            Color(color) => write!(f, "{}", color),
            LinecapDeclarationVariant(linecap) => write!(f, "{}", linecap),
            LinejoinDeclarationVariant(linejoin) => write!(f, "{}", linejoin),
            TextPositionDeclarationVariant(text_pos) => write!(f, "{}", text_pos),
            IntegerArray(ints) => write!(f, "{:?}", ints),
            Integer(int) => write!(f, "{}", int),
            Float(float) => write!(f, "{}", float),
        }
    }
}

impl ToFloatValue for MapCssDeclarationValueType {
    fn to_float(&self) -> FloatSize {
        use MapCssDeclarationValueType::*;

        match self {
            Float(float) => *float,
            Integer(int) => *int as FloatSize,

            _ => panic!("Unable to {:?} convert to float!", &self),
        }
    }
}

impl ToColorValue for MapCssDeclarationValueType {
    fn to_color(&self) -> RGBA {
        use MapCssDeclarationValueType::*;

        match self {
            Color(color) => *color,

            _ => panic!("Unable to {:?} convert to color!", &self),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LinecapDeclarationVariant {
    None,
    Round,
    Square,
}

impl Default for LinecapDeclarationVariant {
    fn default() -> Self {
        LinecapDeclarationVariant::None
    }
}

impl fmt::Display for LinecapDeclarationVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LinecapDeclarationVariant::*;

        write!(
            f,
            "{}",
            match self {
                None => "none",
                Round => "round",
                Square => "square",
            }
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LinejoinDeclarationVariant {
    Round,
    Miter,
    Bevel,
}

impl Default for LinejoinDeclarationVariant {
    fn default() -> Self {
        LinejoinDeclarationVariant::Round
    }
}

impl fmt::Display for LinejoinDeclarationVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LinejoinDeclarationVariant::*;

        write!(
            f,
            "{}",
            match self {
                Round => "round",
                Miter => "miter",
                Bevel => "bevel",
            }
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TextPositionDeclarationVariant {
    Center,
    Line,
}

impl Default for TextPositionDeclarationVariant {
    fn default() -> Self {
        TextPositionDeclarationVariant::Center
    }
}

impl fmt::Display for TextPositionDeclarationVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TextPositionDeclarationVariant::*;

        write!(
            f,
            "{}",
            match self {
                Center => "center",
                Line => "line",
            }
        )
    }
}
