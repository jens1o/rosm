pub mod declaration;
pub mod error;
pub mod parser;
pub mod rule;
pub mod selectors;
pub mod style;

use cssparser::CowRcStr;
use cssparser::Parser;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::u16;
use style::Size;

use declaration::{MapCssDeclarationList, MapCssDeclarationProperty, ToFloatValue};

#[derive(Debug, Default)]
pub struct MapCssAcknowledgement {
    pub title: String,
    pub version: parser::FloatSize,
    pub description: String,
    pub acknowledgement: String,
}

impl MapCssAcknowledgement {
    pub fn from_declarations(
        declarations: MapCssDeclarationList,
    ) -> Result<MapCssAcknowledgement, MapCssParseError> {
        let title = declarations.search_cascading_or_panic(&MapCssDeclarationProperty::Title);
        let version = declarations.search_cascading_or_panic(&MapCssDeclarationProperty::Version);
        let description =
            declarations.search_cascading_or_panic(&MapCssDeclarationProperty::Description);
        let acknowledgement =
            declarations.search_cascading_or_panic(&MapCssDeclarationProperty::Acknowledgement);

        Ok(MapCssAcknowledgement {
            title: title.to_string(),
            version: version.to_float(),
            description: description.to_string(),
            acknowledgement: acknowledgement.to_string(),
        })
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }
}

#[derive(Debug)]
pub enum MapCssParseError {
    InvalidSelector,
    CurrentColorInColor,
    UnknownPropertyName(String),
    // expected unit
    InvalidUnit(&'static str),
    OutOfRange,
}
