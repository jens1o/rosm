pub mod declaration;
pub mod error;
pub mod parser;
pub mod rule;
pub mod selectors;

use declaration::{MapCssDeclaration, MapCssDeclarationProperty};
use selectors::{SelectorCondition, SelectorType};

#[derive(Debug, Default)]
pub struct MapCssAcknowledgement {
    pub title: String,
    // FIXME: Should be a SemVer struct
    pub version: String,
    pub description: String,
    pub acknowledgement: String,
}

impl MapCssAcknowledgement {
    pub fn from_declarations(
        declarations: Vec<MapCssDeclaration>,
    ) -> Result<MapCssAcknowledgement, MapCssParseError> {
        let mut title: Option<String> = None;
        let mut version: Option<String> = None;
        let mut description: Option<String> = None;
        let mut acknowledgement: Option<String> = None;

        for (property, declaration) in declarations.iter() {
            match property {
                MapCssDeclarationProperty::Title => title = Some(declaration.to_string()),
                MapCssDeclarationProperty::Version => version = Some(declaration.to_string()),
                MapCssDeclarationProperty::Description => {
                    description = Some(declaration.to_string())
                }
                MapCssDeclarationProperty::Acknowledgement => {
                    acknowledgement = Some(declaration.to_string())
                }

                _ => (),
            }
        }

        // TODO: Write error messages
        Ok(MapCssAcknowledgement {
            title: title.unwrap(),
            version: version.unwrap(),
            description: description.unwrap(),
            acknowledgement: acknowledgement.unwrap(),
        })
    }

    // TODO: Return &'a str
    pub fn title(&self) -> String {
        self.title.clone()
    }
}

#[derive(Debug)]
pub enum MapCssParseError {
    InvalidSelector,
    CurrentColorInColor,
    /// holds the property name that is unknown
    UnknownPropertyName(String),
    /// holds the expected unit
    InvalidUnit(&'static str),
    OutOfRange,
}
