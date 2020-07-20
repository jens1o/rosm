use crate::data::{ElementData, ElementID};
use crate::mapcss::parser::{FloatSize, IntSize};
use crate::mapcss::selectors::{SelectorCondition, SelectorType};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

mod color;

pub use color::RGBA;

pub type MapCssDeclaration = (MapCssDeclarationProperty, MapCssDeclarationValueType);

pub trait ToFloatValue {
    fn to_float(&self) -> FloatSize;
}

pub trait ToColorValue {
    fn to_color(&self) -> RGBA;
}

pub trait ToBooleanValue {
    fn to_bool(&self) -> bool;
}

static DID_BLAME_ZOOM_LEVEL_NOT_SUPPORTED: AtomicBool = AtomicBool::new(false);
static DID_BLAME_HAS_PSEUDO_CLASS_NOT_SUPPORTED: AtomicBool = AtomicBool::new(false);
static DID_BLAME_HAS_DESCENDANT_NOT_SUPPORTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct MapCssDeclarationList {
    declarations: HashMap<SelectorType, HashMap<SelectorCondition, Vec<MapCssDeclaration>>>,
}

// TODO: Add merge(MapCssDeclarationList) method merging the current list with the latter (latter wins) => cascading properties
// being used in the rendering process
impl MapCssDeclarationList {
    pub fn new(
        declarations: HashMap<SelectorType, HashMap<SelectorCondition, Vec<MapCssDeclaration>>>,
    ) -> MapCssDeclarationList {
        MapCssDeclarationList { declarations }
    }

    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    fn check_condition(element_data: &Box<dyn ElementData>, condition: &SelectorCondition) -> bool {
        use SelectorCondition::*;

        match condition {
            No => true,

            ExactZoomLevel(_) | MinZoomLevel(_) | RangeZoomLevel(_, _) | MaxZoomLevel(_) => {
                if !DID_BLAME_ZOOM_LEVEL_NOT_SUPPORTED.swap(true, Ordering::SeqCst) {
                    warn!("Zoom level specific declarations are currently not supported. Discarding these conditions.");
                }

                true
            }
            List(_) => {
                dbg!(condition, element_data);
                unreachable!("Lists MUST NOT be passed to check_condition()")
            }
            GenericPseudoClass(_pseudo_class) => {
                if !DID_BLAME_HAS_PSEUDO_CLASS_NOT_SUPPORTED.swap(true, Ordering::SeqCst) {
                    warn!("Pseudo classes are currently not supported. Discarded condition.");
                }

                true
            }
            HasTag(condition_tag_key) => element_data
                .tags()
                .iter()
                .any(|(element_tag_key, _element_tag_value)| element_tag_key == condition_tag_key),
            HasExactTagValue(condition_tag_key, condition_tag_value) => element_data
                .tags()
                .iter()
                .any(|(element_tag_key, element_tag_value)| {
                    element_tag_key == condition_tag_key && element_tag_value == condition_tag_value
                }),
            HasDescendant(_) => {
                if !DID_BLAME_HAS_DESCENDANT_NOT_SUPPORTED.swap(true, Ordering::SeqCst) {
                    warn!("The HasDescendant(_) condition is currently not supported. Discarding…");
                }

                true
            }
            _ => false,
        }
    }

    fn search_cascading(
        &self,
        element_data: Box<dyn ElementData>,
        declaration_property_name: &MapCssDeclarationProperty,
    ) -> Option<&MapCssDeclarationValueType> {
        // needs to be ordered from the less specific (* selector) to the most specific one (area)
        let selectors: Vec<SelectorType> = match element_data.id() {
            ElementID::Canvas => [SelectorType::Any, SelectorType::Canvas].into(),
            ElementID::Node(_) => [SelectorType::Any, SelectorType::Node].into(),
            ElementID::Relation(_) => [SelectorType::Any, SelectorType::Relation].into(), // TODO: Support area
            ElementID::Way(_) => match element_data.is_closed() {
                true => [SelectorType::Any, SelectorType::Way, SelectorType::Area].into(),
                false => [SelectorType::Any, SelectorType::Way, SelectorType::Line].into(),
            },
        };

        let mut matching_selector_set_declaration_values: Vec<&MapCssDeclarationValueType> =
            Vec::new();

        for selector in selectors.iter() {
            for declaration_list in self.declarations.get(selector) {
                'declarationListLoop: for (selector_condition, declaration_property_to_value) in
                    declaration_list
                {
                    if let SelectorCondition::List(condition_list) = selector_condition {
                        for condition in condition_list {
                            if let SelectorCondition::List(_) = condition {
                                panic!("Sub-Lists in a SelectorCondition::List are not supported!");
                            } else if !MapCssDeclarationList::check_condition(
                                &element_data,
                                condition,
                            ) {
                                continue 'declarationListLoop;
                            }
                        }
                    } else {
                        if !MapCssDeclarationList::check_condition(
                            &element_data,
                            selector_condition,
                        ) {
                            continue;
                        }
                    }

                    // selector matches to our element, search for declarations that set our target property
                    for (set_declaration_name, set_declaration_value) in
                        declaration_property_to_value
                    {
                        if set_declaration_name == declaration_property_name {
                            matching_selector_set_declaration_values.push(set_declaration_value);
                        }
                    }
                }
            }
        }

        matching_selector_set_declaration_values.pop()
    }

    pub fn search_cascading_or_panic(
        &self,
        element_data: Box<dyn ElementData>,
        declaration_property_name: &MapCssDeclarationProperty,
    ) -> &MapCssDeclarationValueType {
        match self.search_cascading(element_data, declaration_property_name) {
            Some(value) => value,
            None => panic!(
                "Could not find required MapCSS declaration {:?} item.",
                declaration_property_name
            ),
        }
    }

    pub fn search_or_default<'a>(
        &'a self,
        element_data: Box<dyn ElementData>,
        declaration_property_name: &MapCssDeclarationProperty,
        default: &'a MapCssDeclarationValueType,
    ) -> &'a MapCssDeclarationValueType {
        match self.search_cascading(element_data, declaration_property_name) {
            Some(value) => value,
            None => default,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MapCssDeclarationProperty {
    // meta {}
    Title,
    Version,
    Description,
    Acknowledgement,

    /// whether to show all lines by default or only the ones that are styled
    /// only able to be used on the canvas element
    DefaultLines,
    /// whether to show all points by default or only the ones that are styled
    /// only able to be used on the canvas element
    DefaultPoints,

    Linecap,
    Linejoin,

    AllowOverlap,

    Dashes,

    Text,
    TextColor,
    TextPosition,
    TextHaloColor,
    TextHaloRadius,
    TextWrapWidth,
    TextSpacing,

    BackgroundColor,
    Color,
    FontSize,
    FontColor,
    FontFamily,

    IconImage,
    PatternImage,

    Opacity,
    FillOpacity,
    FillColor,
    FillImage,
    Width,

    ZIndex,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MapCssDeclarationValueType {
    Boolean(bool),
    String(String),
    Color(RGBA),
    LinecapDeclarationVariant(LinecapDeclarationVariant),
    LinejoinDeclarationVariant(LinejoinDeclarationVariant),
    TextPositionDeclarationVariant(TextPositionDeclarationVariant),
    IntegerArray(Vec<IntSize>),
    Integer(IntSize),
    Float(FloatSize),
}

impl fmt::Display for MapCssDeclarationValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MapCssDeclarationValueType::*;

        match self {
            Boolean(boolean) => write!(f, "{}", if *boolean { "true" } else { "false" }),
            String(string) => write!(f, "{}", string),
            Color(color) => write!(f, "{}", color),
            LinecapDeclarationVariant(linecap) => write!(f, "{}", linecap),
            LinejoinDeclarationVariant(linejoin) => write!(f, "{}", linejoin),
            TextPositionDeclarationVariant(text_pos) => write!(f, "{}", text_pos),
            IntegerArray(ints) => write!(f, "{:?}", ints),
            Integer(int) => write!(f, "{}", int),
            Float(float) => write!(f, "{}", float),
        }
    }
}

impl ToFloatValue for MapCssDeclarationValueType {
    fn to_float(&self) -> FloatSize {
        use MapCssDeclarationValueType::*;

        match self {
            Float(float) => *float,
            Integer(int) => *int as FloatSize,

            _ => panic!("Unable to {:?} convert to float!", &self),
        }
    }
}

impl ToColorValue for MapCssDeclarationValueType {
    fn to_color(&self) -> RGBA {
        use MapCssDeclarationValueType::*;

        match self {
            Color(color) => *color,

            _ => panic!("Unable to {:?} convert to color!", &self),
        }
    }
}

impl ToBooleanValue for MapCssDeclarationValueType {
    fn to_bool(&self) -> bool {
        use MapCssDeclarationValueType::*;

        match self {
            Boolean(bool) => *bool,

            _ => panic!("Unable to {:?} convert to boolean!", &self),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LinecapDeclarationVariant {
    None,
    Round,
    Square,
}

impl Default for LinecapDeclarationVariant {
    fn default() -> Self {
        LinecapDeclarationVariant::None
    }
}

impl fmt::Display for LinecapDeclarationVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LinecapDeclarationVariant::*;

        write!(
            f,
            "{}",
            match self {
                None => "none",
                Round => "round",
                Square => "square",
            }
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LinejoinDeclarationVariant {
    Round,
    Miter,
    Bevel,
}

impl Default for LinejoinDeclarationVariant {
    fn default() -> Self {
        LinejoinDeclarationVariant::Round
    }
}

impl fmt::Display for LinejoinDeclarationVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LinejoinDeclarationVariant::*;

        write!(
            f,
            "{}",
            match self {
                Round => "round",
                Miter => "miter",
                Bevel => "bevel",
            }
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TextPositionDeclarationVariant {
    Center,
    Line,
}

impl Default for TextPositionDeclarationVariant {
    fn default() -> Self {
        TextPositionDeclarationVariant::Center
    }
}

impl fmt::Display for TextPositionDeclarationVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TextPositionDeclarationVariant::*;

        write!(
            f,
            "{}",
            match self {
                Center => "center",
                Line => "line",
            }
        )
    }
}
