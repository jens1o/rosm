use crate::mapcss::parser::{FloatSize, IntSize};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

mod color;

pub use color::RGBA;

pub type MapCssDeclaration = (MapCssDeclarationProperty, MapCssDeclarationValueType);

pub trait ToFloatValue {
    fn to_float(&self) -> FloatSize;
}

#[derive(Debug)]
pub struct MapCssDeclarationList {
    declarations: Vec<MapCssDeclaration>,
}

impl MapCssDeclarationList {
    pub fn new(declarations: Vec<MapCssDeclaration>) -> MapCssDeclarationList {
        MapCssDeclarationList { declarations }
    }

    pub fn search_cascading_or_panic(
        &self,
        declaration_property_name: &MapCssDeclarationProperty,
    ) -> &MapCssDeclarationValueType {
        self.declarations
            .iter()
            .rfind(|(name, _value)| name == declaration_property_name)
            .and_then(|(_name, value)| Some(value))
            .unwrap_or_else(|| {
                panic!(
                    "Could not find required MapCSS declaration {:?} item!",
                    declaration_property_name
                );
            })
    }
}

impl From<Vec<MapCssDeclaration>> for MapCssDeclarationList {
    fn from(vec: Vec<MapCssDeclaration>) -> MapCssDeclarationList {
        MapCssDeclarationList::new(vec)
    }
}

#[derive(Debug, PartialEq)]
pub enum MapCssDeclarationProperty {
    // meta {}
    Title,
    Version,
    Description,
    Acknowledgement,

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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
