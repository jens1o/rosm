use super::declaration::{
    MapCssDeclaration, MapCssDeclarationList, MapCssDeclarationProperty, MapCssDeclarationValueType,
};
use super::error::MapCssError;
use super::selectors::{Selector, SelectorCondition, SelectorType};
use super::MapCssAcknowledgement;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::collections::HashMap;
use std::rc::Rc;

pub type FloatSize = f64;
pub type IntSize = i32;

#[derive(Parser)]
#[grammar = "grammar/mapcss.pest"]
pub struct MapCssParser;

impl MapCssParser {
    // TODO: Use this method for testing against various stylesheets (maybe with errors?)
    pub fn lex(mapcss: &str) -> Pairs<Rule> {
        MapCssParser::parse(Rule::rule_list, mapcss).unwrap()
    }

    pub fn parse_mapcss(
        mapcss: &str,
    ) -> Result<
        (
            Option<MapCssAcknowledgement>,
            HashMap<SelectorType, HashMap<SelectorCondition, Vec<MapCssDeclaration>>>,
        ),
        MapCssError,
    > {
        let pairs = MapCssParser::lex(mapcss);

        let mut selector_to_declaration_list: HashMap<
            SelectorType,
            HashMap<SelectorCondition, Vec<MapCssDeclaration>>,
        > = HashMap::new();
        let mut acknowledgement = None;

        for rule in pairs {
            match rule.as_rule() {
                Rule::COMMENT => {
                    // ignore comments for now
                }
                Rule::rule => {
                    let rule_contents = rule.into_inner();
                    let mut selector_list: Vec<Selector> = Vec::with_capacity(2);
                    let mut declarations: Vec<MapCssDeclaration> = Vec::new();

                    for rule_content in rule_contents {
                        match rule_content.as_rule() {
                            Rule::rule_selector => {
                                selector_list.push(handle_selector(rule_content));
                            }
                            Rule::rule_declaration => match handle_declaration(rule_content) {
                                Ok(dec) => {
                                    declarations.push(dec);
                                }
                                Err(err) => {
                                    eprintln!("{}", err);
                                }
                            },
                            Rule::COMMENT => (),
                            _ => todo!("{rule_content}"),
                        };
                    }

                    // handle meta information like the meta mapcss block
                    debug_assert!(!selector_list.is_empty());
                    let selector_list_len = selector_list.len();

                    for selector in selector_list.into_iter() {
                        let selector_type = selector.selector_type();
                        if selector_type == SelectorType::Meta {
                            // TODO: Bail semantic error (the meta block has unique MapCSS properties that may not appear in any other ruleset)
                            debug_assert_eq!(selector_list_len, 1);

                            if selector.conditions() == &SelectorCondition::True {
                                acknowledgement =
                                    MapCssAcknowledgement::from_declarations(declarations.clone())
                                        .ok();
                                break;
                            } else {
                                warn!("The meta {{}} block must not have any selector conditions, ignoring!");
                            }
                        } else {
                            selector_to_declaration_list
                                .entry(selector_type)
                                .or_default()
                                .entry(selector.conditions().clone())
                                .or_default()
                                .extend(declarations.clone());
                        }
                    }
                }
                Rule::EOI => break,
                _ => unreachable!(),
            };
        }

        Ok((acknowledgement, selector_to_declaration_list))
    }
}

fn handle_selector(selectors: Pair<'_, Rule>) -> Selector {
    let mut rule_selectors = selectors.into_inner();

    let main_selector = rule_selectors.next().unwrap();

    assert_eq!(main_selector.as_rule(), Rule::selector);

    let mut main_selector = selector_span_to_type(
        main_selector.as_span().as_str(),
        selector_condition_from_rule_selectors(&mut rule_selectors.clone()),
    );

    let mut main_selector_conditions = main_selector.clone().conditions().clone();

    for descendant_selectors in rule_selectors.filter(|x| x.as_rule() == Rule::rule_descendant) {
        let descendant_selector =
            handle_selector(descendant_selectors.into_inner().next().unwrap());

        main_selector_conditions = main_selector_conditions.add_condition(
            SelectorCondition::HasDescendant(Rc::new(descendant_selector)),
        );
    }

    main_selector.set_conditions(main_selector_conditions);

    main_selector
}

#[inline]
fn selector_span_to_type(span: &str, selector_conditions: SelectorCondition) -> Selector {
    let selector_type = match span {
        "*" => SelectorType::Any,
        "area" => SelectorType::Area,
        "canvas" => SelectorType::Canvas,
        "line" => SelectorType::Line,
        "meta" => SelectorType::Meta,
        "node" => SelectorType::Node,
        "relation" => SelectorType::Relation,
        "way" => SelectorType::Way,

        _ => unreachable!(),
    };

    Selector::new(selector_type, selector_conditions)
}

fn selector_condition_from_rule_selectors(
    rules: &mut pest::iterators::Pairs<'_, Rule>,
) -> SelectorCondition {
    if rules.peek().is_none() {
        return SelectorCondition::True;
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
                    Rule::not_pseudo_class => {
                        selector_conditions.push(SelectorCondition::Not(Rc::new(handle_selector(
                            pseudo_class,
                        ))));
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
        0 => SelectorCondition::True,
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
    let target = target.as_span().as_str().to_owned();
    let expected = expected.as_span().as_str().to_owned();

    condition_list.push(match operator {
        "=" => SelectorCondition::HasExactTagValue(target, expected),
        "!=" => SelectorCondition::HasNotTagValue(target, expected),
        ">" | ">=" | "<" | "<=" => {
            let numeric_expected_value = expected.parse::<isize>().ok();

            match numeric_expected_value {
                Some(numeric_value) => match operator {
                    ">" => SelectorCondition::ValueGreaterThan(target, numeric_value),
                    ">=" => SelectorCondition::ValueGreaterThanEqual(target, numeric_value),
                    "<" => SelectorCondition::ValueLessThan(target, numeric_value),
                    "<=" => SelectorCondition::ValueLessThanEqual(target, numeric_value),
                    _ => unreachable!(),
                },
                None => return,
            }
        }
        _ => {
            dbg!(operator);
            todo!();
        }
    })
}

fn handle_declaration(
    declaration: pest::iterators::Pair<'_, Rule>,
) -> Result<MapCssDeclaration, MapCssError> {
    assert_eq!(declaration.as_rule(), Rule::rule_declaration);

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
