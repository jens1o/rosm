use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MapCssError {
    UnknownDeclarationName(String),
    IllegalDeclaration {
        declaration_name: String,
        illegal_context: &'static str,
    },
}

impl Error for MapCssError {}

impl fmt::Display for MapCssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MapCssError::*;

        match self {
            UnknownDeclarationName(declaration_name) => write!(
                f,
                "Dropped unknown declaration name \"{}\".",
                declaration_name
            ),
            IllegalDeclaration {
                declaration_name,
                illegal_context,
            } => write!(
                f,
                "Declaration {} must not appear in {} block! Block dropped.",
                declaration_name, illegal_context
            ),
        }
    }
}
