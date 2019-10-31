mod style;

use cssparser::CowRcStr;
use cssparser::Parser;
use std::error::Error;
use std::u8;
use style::Size;

#[derive(Debug)]
pub enum MapCssError<'i> {
    InvalidSelector,
    CurrentColorInColor,
    UnknownPropertyName(CowRcStr<'i>),
    // expected unit
    InvalidUnit(&'i str),
    OutOfRange,
}

pub type MapCssParseError<'i> = cssparser::ParseError<'i, MapCssError<'i>>;

struct MapCssParser;

#[derive(Debug)]
pub enum MapCssPropertyDeclaration {
    ZIndex(u8),
    // Colorcode
    Color(cssparser::RGBA),
    Width(Size),
}

#[derive(Debug)]
struct MapCssStyleRule {
    selectors: kuchiki::Selectors,
    declarations: Vec<MapCssPropertyDeclaration>,
}

impl<'i> cssparser::AtRuleParser<'i> for MapCssParser {
    type PreludeBlock = ();
    type PreludeNoBlock = ();
    type AtRule = MapCssStyleRule;
    type Error = MapCssError<'i>;

    // Default methods reject everything.
}

impl<'i> cssparser::QualifiedRuleParser<'i> for MapCssParser {
    type Prelude = kuchiki::Selectors;
    type QualifiedRule = MapCssStyleRule;
    type Error = MapCssError<'i>;

    #[inline]
    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, MapCssParseError<'i>> {
        let location = input.current_source_location();
        let position = input.position();
        while input.next().is_ok() {}
        kuchiki::Selectors::compile(input.slice_from(position))
            .map_err(|()| location.new_custom_error(MapCssError::InvalidSelector))
    }

    #[inline]
    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        _location: cssparser::SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<MapCssStyleRule, MapCssParseError<'i>> {
        Ok(MapCssStyleRule {
            selectors,
            declarations: parse_declarations(input).unwrap(),
        })
    }
}

#[derive(Debug)]
struct PropertyDeclarationParser;
impl<'i> cssparser::DeclarationParser<'i> for PropertyDeclarationParser {
    type Declaration = Vec<MapCssPropertyDeclaration>;
    type Error = MapCssError<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        mut input: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, cssparser::ParseError<'i, MapCssError<'i>>> {
        match name.to_ascii_lowercase().as_str() {
            "color" => parse_rgba(&mut input)
                .and_then(|rgba| Ok(vec![MapCssPropertyDeclaration::Color(rgba)])),
            "width" => parse_size(&mut input)
                .and_then(|width| Ok(vec![MapCssPropertyDeclaration::Width(width)])),
            "z-index" => parse_z_index(&mut input)
                .and_then(|z_index| Ok(vec![MapCssPropertyDeclaration::ZIndex(z_index)])),
            _ => Err(input.new_custom_error(MapCssError::UnknownPropertyName(name.clone()))),
        }
    }
}

impl<'i> cssparser::AtRuleParser<'i> for PropertyDeclarationParser {
    type PreludeBlock = ();
    type PreludeNoBlock = ();
    type AtRule = Vec<MapCssPropertyDeclaration>;
    type Error = MapCssError<'i>;
}

pub fn parse_declarations<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<MapCssPropertyDeclaration>, Box<dyn Error>> {
    let mut declarations = Vec::new();
    let mut iter = cssparser::DeclarationListParser::new(input, PropertyDeclarationParser);

    for declaration_list in iter {
        let declaration_list = match declaration_list {
            Ok(l) => l,
            Err(e) => {
                eprintln!("CSS declaration dropped: {:?}", e);
                continue;
            }
        };
        for declaration in declaration_list {
            dbg!(&declaration);
            declarations.push(declaration);
        }
    }

    Ok(declarations)
}

pub fn parse_mapcss(mapcss: &str) {
    let mut parser_input = cssparser::ParserInput::new(mapcss);
    let mut parser = cssparser::Parser::new(&mut parser_input);

    let mut rule_list_parser =
        cssparser::RuleListParser::new_for_stylesheet(&mut parser, MapCssParser);

    while let Some(token) = rule_list_parser.next() {
        dbg!(token);
    }
}

fn parse_color<'i>(input: &mut Parser<'i, '_>) -> Result<cssparser::Color, MapCssParseError<'i>> {
    Ok(cssparser::Color::parse(input)?)
}

fn parse_rgba<'i>(input: &mut Parser<'i, '_>) -> Result<cssparser::RGBA, MapCssParseError<'i>> {
    let color = parse_color(input)?;
    match color {
        cssparser::Color::RGBA(rgba) => Ok(rgba),
        cssparser::Color::CurrentColor => {
            Err(input.new_custom_error(MapCssError::CurrentColorInColor))
        }
    }
}

// TODO: parse eval(2*3)
fn parse_size<'i>(input: &mut Parser<'i, '_>) -> Result<Size, MapCssParseError<'i>> {
    let location = input.current_source_location();
    match *input.next()? {
        cssparser::Token::Number { value, .. } => return Ok(Size(value)),
        ref t => Err(location.new_unexpected_token_error(t.clone())),
    }
}

fn parse_z_index<'i>(input: &mut Parser<'i, '_>) -> Result<u8, MapCssParseError<'i>> {
    let location = input.current_source_location();
    match *input.next()? {
        cssparser::Token::Number { int_value, .. } => {
            if let Some(int_value) = int_value {
                if int_value >= u8::min_value() as i32 && int_value <= u8::max_value() as i32 {
                    return Ok(int_value as u8);
                }

                return Err(input.new_custom_error(MapCssError::OutOfRange));
            }

            Err(input.new_custom_error(MapCssError::InvalidUnit("px")))
        }
        ref t => Err(location.new_unexpected_token_error(t.clone())),
    }
}
