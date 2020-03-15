use crate::mapcss::parser::{FloatSize, IntSize};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;

mod color;

pub use color::RGBA;

#[derive(Debug)]
pub enum MapCssDeclaration {
    // meta {}
    Title(String),
    Version(String),
    Description(String),
    Acknowledgement(String),

    Linecap(LinecapDeclarationVariant),
    Linejoin(LinejoinDeclarationVariant),

    AllowOverlap(bool),

    Dashes(Vec<IntSize>),

    Text(String),
    TextColor(RGBA),
    TextPosition(TextPositionDeclarationVariant),
    TextHaloColor(RGBA),
    TextHaloRadius(IntSize),
    TextWrapWidth(IntSize),
    TextSpacing(IntSize),

    BackgroundColor(RGBA),
    Color(RGBA),
    FontSize(IntSize),
    FontColor(RGBA),
    FontFamily(String),

    IconImage(String),
    PatternImage(String),

    Opacity(FloatSize),
    FillOpacity(FloatSize),
    FillColor(RGBA),
    FillImage(String),
    Width(FloatSize),

    ZIndex(FloatSize),
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum TextPositionDeclarationVariant {
    Center,
    Line,
}

impl Default for TextPositionDeclarationVariant {
    fn default() -> Self {
        TextPositionDeclarationVariant::Center
    }
}
