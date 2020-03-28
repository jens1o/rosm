pub mod declaration;
pub mod error;
pub mod parser;
pub mod rule;
pub mod selectors;

use declaration::{MapCssDeclarationList, MapCssDeclarationProperty};

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
            version: version.to_string(),
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
