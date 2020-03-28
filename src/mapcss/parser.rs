use super::declaration::{
    MapCssDeclaration, MapCssDeclarationProperty, MapCssDeclarationValueType,
};
use super::error::MapCssError;
use super::rule::MapCssRule;
use super::selectors::{Selector, SelectorCondition};
use super::MapCssAcknowledgement;
use pest::Parser;
use std::rc::Rc;

pub type FloatSize = f32;
pub type IntSize = i32;

#[derive(Parser)]
#[grammar = "grammar/mapcss.pest"]
pub struct MapCssParser;

impl MapCssParser {
    pub fn parse_mapcss(mapcss: &str) -> (Option<MapCssAcknowledgement>, Vec<MapCssRule>) {
        let pairs = MapCssParser::parse(Rule::rule_list, mapcss).unwrap();

        let mapcss_rules: Vec<MapCssRule> = Vec::new();
        let mut acknowledgement = None;

        for rule in pairs {
            match rule.as_rule() {
                Rule::rule => {
                    let rule_contents = rule.into_inner();
                    let mut selector: Option<Selector> = None;
                    let mut declarations: Vec<MapCssDeclaration> = Vec::new();

                    for rule_content in rule_contents {
                        match rule_content.as_rule() {
                            Rule::rule_selector => {
                                selector = Some(handle_selector(rule_content));
                            }
                            Rule::rule_declaration => match handle_declaration(rule_content) {
                                Ok(dec) => {
                                    declarations.push(dec);
                                }
                                Err(err) => {
                                    eprintln!("{}", err);
                                }
                            },
                            _ => unreachable!(),
                        };
                    }

                    // handle meta information like the meta mapcss block
                    debug_assert!(selector.is_some());

                    if let Some(Selector::Meta(selector_condition)) = selector {
                        if selector_condition != SelectorCondition::No {
                            panic!("MapCSS meta tag must not have any selector condition!");
                        }

                        acknowledgement =
                            MapCssAcknowledgement::from_declarations(declarations.into()).ok();
                    } else {
                        // dbg!(selector, declarations);
                    }
                }
                Rule::EOI => break,
                _ => unreachable!(),
            };
        }

        (acknowledgement, vec![])
    }
}

fn handle_selector(selectors: pest::iterators::Pair<'_, Rule>) -> Selector {
    let mut rule_selectors = selectors.into_inner();

    let main_selector = rule_selectors.next().unwrap();

    debug_assert_eq!(main_selector.as_rule(), Rule::selector);

    let main_selector = selector_span_to_type(
        main_selector.as_span().as_str(),
        selector_condition_from_rule_selectors(&mut rule_selectors.clone()),
    );

    let mut main_selector_conditions = main_selector.clone().conditions();

    for descendant_selectors in rule_selectors.filter(|x| x.as_rule() == Rule::rule_descendant) {
        let descendant_selector =
            handle_selector(descendant_selectors.into_inner().next().unwrap());

        main_selector_conditions = main_selector_conditions.add_condition(
            SelectorCondition::HasDescendant(Rc::new(descendant_selector)),
        );
    }

    main_selector.set_conditions(main_selector_conditions)
}

#[inline]
fn selector_span_to_type(span: &str, selector_conditions: SelectorCondition) -> Selector {
    match span {
        "*" => Selector::Any(selector_conditions),
        "area" => Selector::Area(selector_conditions),
        "canvas" => Selector::Canvas(selector_conditions),
        "line" => Selector::Line(selector_conditions),
        "meta" => Selector::Meta(selector_conditions),
        "node" => Selector::Node(selector_conditions),
        "relation" => Selector::Relation(selector_conditions),
        "way" => Selector::Way(selector_conditions),

        _ => unreachable!(),
    }
}

fn selector_condition_from_rule_selectors(
    rules: &mut pest::iterators::Pairs<'_, Rule>,
) -> SelectorCondition {
    if rules.peek().is_none() {
        return SelectorCondition::No;
    }

    let mut selector_conditions = Vec::new();

    for rule in rules {
        match rule.as_rule() {
            Rule::rule_descendant => {
                continue;
            }
            Rule::selector_tests => {
                let mut inner_rules = rule.into_inner().peekable();

                let inner_rule = inner_rules.next();

                if let Some(inner_rule) = inner_rule {
                    match inner_rule.as_rule() {
                        Rule::tag_value => {
                            let next_rule = inner_rules.peek();

                            match next_rule.map(|r| r.as_rule()) {
                                Some(Rule::comparison) => {
                                    let operator = inner_rules.next().unwrap().as_span().as_str();

                                    operator_to_condition(
                                        operator,
                                        inner_rule,
                                        inner_rules
                                            .next()
                                            .expect("Target required when doing an comparison!"),
                                        &mut selector_conditions,
                                    );
                                }
                                None => selector_conditions.push(SelectorCondition::HasTag(
                                    inner_rule.as_span().as_str().to_owned(),
                                )),
                                _ => unreachable!(),
                            }
                        }
                        Rule::selector_test_zoom_level => {
                            let selector_test = inner_rule.into_inner().next().unwrap();
                            let span = selector_test.as_span().as_str();

                            match selector_test.as_rule() {
                                Rule::selector_test_zoom_level_exact => {
                                    // "|z8"
                                    selector_conditions.push(SelectorCondition::ExactZoomLevel(
                                        span[2..span.len()].parse::<u8>().unwrap(),
                                    ));
                                }
                                // use rfind because the minus is always located more towards the end
                                Rule::selector_test_zoom_level_closed_range => {
                                    // "|z10-12"
                                    let minus_pos = span.rfind('-').unwrap();

                                    let min_level = &span[2..minus_pos].parse::<u8>().unwrap();
                                    // skip the minus itself
                                    let max_level =
                                        &span[minus_pos + 1..span.len()].parse::<u8>().unwrap();

                                    debug_assert!(max_level > min_level);

                                    selector_conditions.push(SelectorCondition::RangeZoomLevel(
                                        *min_level, *max_level,
                                    ));
                                }
                                Rule::selector_test_zoom_level_open_right_range => {
                                    // "|z14-" or "|z4-"
                                    let zoom_level =
                                        &span[2..span.rfind('-').unwrap()].parse::<u8>().unwrap();

                                    selector_conditions
                                        .push(SelectorCondition::MinZoomLevel(*zoom_level));
                                }
                                _ => {
                                    dbg!(selector_test);
                                    todo!();
                                }
                            }
                        }
                        _ => {
                            dbg!(inner_rule);
                            todo!();
                        }
                    }
                }
            }
            Rule::selector_pseudo_classes => {
                let pseudo_class = rule
                    .into_inner()
                    .next()
                    .expect("A pseudo class needs a content for it!");

                match pseudo_class.as_rule() {
                    Rule::closed_pseudo_class => {
                        selector_conditions.push(SelectorCondition::ClosedPath);
                    }
                    Rule::generic_pseudo_class => {
                        let maybe_selector = pseudo_class.into_inner().next().unwrap();

                        if maybe_selector.as_rule() == Rule::selector {
                            let selector = maybe_selector;
                            let span_str = selector.as_span().as_str();

                            // we can ignore the `*` subclass, as it does not make any difference
                            if span_str != "*" {
                                selector_conditions.push(SelectorCondition::GenericPseudoClass(
                                    span_str.to_owned(),
                                ));
                            }
                        } else {
                            // TODO: Find out what it does in other implementations
                            selector_conditions.push(SelectorCondition::GenericPseudoClass(
                                maybe_selector.as_span().as_str().to_owned(),
                            ));
                        }
                    }
                    _ => {
                        dbg!(pseudo_class);
                        unreachable!();
                    }
                }
            }
            _ => {
                dbg!(&rule, selector_conditions);
                todo!();
            }
        }
    }

    match selector_conditions.len() {
        0 => SelectorCondition::No,
        1 => selector_conditions.into_iter().next().unwrap(),
        _ => SelectorCondition::List(selector_conditions),
    }
}

fn operator_to_condition(
    operator: &str,
    target: pest::iterators::Pair<'_, Rule>,
    expected: pest::iterators::Pair<'_, Rule>,
    condition_list: &mut Vec<SelectorCondition>,
) {
    match operator {
        "=" => condition_list.push(SelectorCondition::HasExactTagValue(
            target.as_span().as_str().to_owned(),
            expected.as_span().as_str().to_owned(),
        )),
        "!=" => condition_list.push(SelectorCondition::HasNotTagValue(
            target.as_span().as_str().to_owned(),
            expected.as_span().as_str().to_owned(),
        )),
        ">" => condition_list.push(SelectorCondition::ValueGreaterThan(
            target.as_span().as_str().to_owned(),
            expected.as_span().as_str().to_owned(),
        )),
        ">=" => condition_list.push(SelectorCondition::ValueGreaterThanEqual(
            target.as_span().as_str().to_owned(),
            expected.as_span().as_str().to_owned(),
        )),
        "<" => condition_list.push(SelectorCondition::ValueLessThan(
            target.as_span().as_str().to_owned(),
            expected.as_span().as_str().to_owned(),
        )),
        "<=" => condition_list.push(SelectorCondition::ValueLessThanEqual(
            target.as_span().as_str().to_owned(),
            expected.as_span().as_str().to_owned(),
        )),
        _ => {
            dbg!(operator);
            todo!();
        }
    }
}

fn handle_declaration(
    declaration: pest::iterators::Pair<'_, Rule>,
) -> Result<MapCssDeclaration, MapCssError> {
    debug_assert_eq!(declaration.as_rule(), Rule::rule_declaration);

    let mut inner = declaration.into_inner();

    let declaration_name = inner.next().unwrap().as_span().as_str();
    let inner = inner.next().unwrap();
    let inner_rule = inner.as_rule();

    macro_rules! to_string {
        () => {
            // remove quotations
            if inner_rule == Rule::double_quoted_string || inner_rule == Rule::single_quoted_string
            {
                let as_str = inner.as_span().as_str();
                MapCssDeclarationValueType::String(as_str[1..as_str.len() - 1].to_owned())
            } else {
                MapCssDeclarationValueType::String(inner.as_span().as_str().to_owned())
            }
        };
    }

    macro_rules! to_float {
        () => {
            if inner_rule == Rule::float || inner_rule == Rule::int {
                MapCssDeclarationValueType::Float(
                    inner
                        .as_span()
                        .as_str()
                        .to_owned()
                        .parse::<FloatSize>()
                        .unwrap(),
                )
            } else {
                dbg!(declaration_name, inner_rule);
                panic!("Invalid float AST!");
            }
        };
    }

    macro_rules! to_int {
        () => {
            // TODO: Catch error
            MapCssDeclarationValueType::Integer(
                inner.as_span().as_str().parse::<IntSize>().unwrap(),
            )
        };
    }

    macro_rules! to_bool {
        () => {
            // TODO: Catch error
            MapCssDeclarationValueType::Boolean(match inner.as_span().as_str() {
                "true" | "1" => true,
                "false" | "0" => false,
                _ => panic!("Invalid boolean value"),
            })
        };
    }

    macro_rules! to_color {
        () => {
            // TODO: Catch error
            MapCssDeclarationValueType::Color(
                inner
                    .as_span()
                    .as_str()
                    .parse::<crate::mapcss::declaration::RGBA>()
                    .unwrap(),
            )
        };
    }

    macro_rules! maybe_url_to_string {
        () => {
            // TODO: Catch error
            MapCssDeclarationValueType::String(if inner_rule == Rule::url {
                let url_string = inner.into_inner().as_str();
                url_string[1..url_string.len() - 1].to_owned()
            } else if inner_rule == Rule::double_quoted_string
                || inner_rule == Rule::single_quoted_string
            {
                let as_str = inner.as_span().as_str();
                as_str[1..as_str.len() - 1].to_owned()
            } else {
                inner.as_span().as_str().to_owned()
            })
        };
    }

    use crate::mapcss::declaration::{
        LinecapDeclarationVariant, LinejoinDeclarationVariant, TextPositionDeclarationVariant,
    };

    Ok(match declaration_name.to_ascii_lowercase().as_str() {
        "title" => (MapCssDeclarationProperty::Title, to_string!()),
        "version" => (MapCssDeclarationProperty::Version, to_string!()),
        "description" => (MapCssDeclarationProperty::Description, to_string!()),
        "acknowledgement" => (MapCssDeclarationProperty::Acknowledgement, to_string!()),

        "allow_overlap" => (MapCssDeclarationProperty::AllowOverlap, to_bool!()),

        "dashes" => {
            // TODO: Make sure that the syntax is right
            // TODO: Prevent DoS?!

            (
                MapCssDeclarationProperty::Dashes,
                MapCssDeclarationValueType::IntegerArray(
                    inner
                        .as_span()
                        .as_str()
                        .split(',')
                        .map(|x| x.parse::<IntSize>().unwrap())
                        .collect::<Vec<IntSize>>(),
                ),
            )
        }

        "default-lines" => (MapCssDeclarationProperty::DefaultLines, to_bool!()),
        "default-points" => (MapCssDeclarationProperty::DefaultPoints, to_bool!()),

        "text" => (MapCssDeclarationProperty::Text, to_string!()),
        "text-color" => (MapCssDeclarationProperty::TextColor, to_color!()),
        "text-position" => (
            MapCssDeclarationProperty::TextPosition,
            MapCssDeclarationValueType::TextPositionDeclarationVariant(
                match inner.as_span().as_str() {
                    "center" => TextPositionDeclarationVariant::Center,
                    "line" => TextPositionDeclarationVariant::Line,
                    _ => panic!(),
                },
            ),
        ),
        "text-spacing" => (MapCssDeclarationProperty::TextSpacing, to_int!()),
        "text-halo-color" => (MapCssDeclarationProperty::TextHaloColor, to_color!()),
        "text-halo-radius" => (MapCssDeclarationProperty::TextHaloRadius, to_int!()),
        "text-wrap-width" => (MapCssDeclarationProperty::TextWrapWidth, to_int!()),

        "background-color" => (MapCssDeclarationProperty::BackgroundColor, to_color!()),
        "color" => (MapCssDeclarationProperty::Color, to_color!()),
        // in MapCSS, the font size is always given in absolute pixels
        "font-size" => (MapCssDeclarationProperty::FontSize, to_int!()),
        "font-color" => (MapCssDeclarationProperty::FontColor, to_color!()),
        "font-family" => (MapCssDeclarationProperty::FontFamily, to_string!()),

        "linecap" => (
            MapCssDeclarationProperty::Linecap,
            MapCssDeclarationValueType::LinecapDeclarationVariant(match inner.as_span().as_str() {
                "none" => LinecapDeclarationVariant::None,
                "round" => LinecapDeclarationVariant::Round,
                "square" => LinecapDeclarationVariant::Square,
                _ => panic!(),
            }),
        ),

        "linejoin" => (
            MapCssDeclarationProperty::Linejoin,
            MapCssDeclarationValueType::LinejoinDeclarationVariant(
                match inner.as_span().as_str() {
                    "round" => LinejoinDeclarationVariant::Round,
                    "miter" => LinejoinDeclarationVariant::Miter,
                    "bevel" => LinejoinDeclarationVariant::Bevel,
                    _ => panic!(),
                },
            ),
        ),

        "fill-opacity" => (MapCssDeclarationProperty::FillOpacity, to_float!()),
        "fill-color" => (MapCssDeclarationProperty::FillColor, to_color!()),
        "fill-image" => (MapCssDeclarationProperty::FillImage, maybe_url_to_string!()),

        "icon-image" => (MapCssDeclarationProperty::IconImage, maybe_url_to_string!()),

        "pattern-image" => (
            MapCssDeclarationProperty::PatternImage,
            maybe_url_to_string!(),
        ),

        "opacity" => (MapCssDeclarationProperty::Opacity, to_float!()),
        "width" => (MapCssDeclarationProperty::Width, to_float!()),

        "z-index" => (MapCssDeclarationProperty::ZIndex, to_float!()),

        _ => {
            return Err(MapCssError::UnknownDeclarationName(
                declaration_name.to_owned(),
            ))
        }
    })
}
