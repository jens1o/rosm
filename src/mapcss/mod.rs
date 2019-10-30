use cssparser::CowRcStr;
use cssparser::Parser;
use std::error::Error;

#[derive(Debug)]
pub enum MapCssError<'i> {
    InvalidSelector,
    CurrentColorInColor,
    EmptyBorder,
    UnknownPropertyName(CowRcStr<'i>),
    UnknownLengthUnit(CowRcStr<'i>),
}

pub type MapCssParseError<'i> = cssparser::ParseError<'i, MapCssError<'i>>;

struct MapCssParser;

#[derive(Debug)]
pub enum MapCssPropertyDeclaration {}

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
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, cssparser::ParseError<'i, MapCssError<'i>>> {
        dbg!(name);

        Err(input.new_custom_error(MapCssError::UnknownPropertyName(name.clone())))
    }
}

impl<'i> cssparser::AtRuleParser<'i> for PropertyDeclarationParser {
    type PreludeBlock = ();
    type PreludeNoBlock = ();
    type AtRule = Vec<MapCssPropertyDeclaration>;
    type Error = MapCssParseError<'i>;
}

pub fn parse_declarations<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<MapCssPropertyDeclaration>, Box<dyn Error>> {
    let mut declarations = Vec::new();
    let iter = cssparser::DeclarationListParser::new(input, PropertyDeclarationParser);

    dbg!(iter.next());
    // for declaration_list in iter {
    //     let declaration_list = match declaration_list {
    //         Ok(l) => l,
    //         Err(e) => {
    //             eprintln!("CSS declaration dropped: {:?}", e);
    //             continue;
    //         }
    //     };
    //     for declaration in declaration_list {
    //         declarations.push(declaration);
    //     }
    // }
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
