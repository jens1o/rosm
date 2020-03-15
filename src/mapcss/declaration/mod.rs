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

    Dashes(Vec<IntSize>),

    Text(String),
    TextColor(RGBA),

    BackgroundColor(RGBA),
    Color(RGBA),
    FontSize(IntSize),
    FontColor(RGBA),
    FontFamily(String),

    Opacity(FloatSize),
    FillOpacity(FloatSize),
    FillColor(RGBA),
    Width(FloatSize),

    ZIndex(FloatSize),
}
