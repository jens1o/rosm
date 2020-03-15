use crate::mapcss::parser::FloatSize;
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

    Text(String),

    Color(RGBA),
    BackgroundColor(RGBA),
    FontFamily(String),

    Opacity(FloatSize),
    FillOpacity(FloatSize),
    Width(FloatSize),
}
