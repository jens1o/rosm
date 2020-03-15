#[derive(Debug)]
pub enum SelectorCondition {
    No,

    ExactZoomLevel(u8),
    MinZoomLevel(u8),
    RangeZoomLevel(u8, u8),
    MaxZoomLevel(u8),
    Not(Box<Selector>),
    GenericPseudoClass(String),
    HasTag(String),
    HasExactTagValue(String, String),
    HasNotTagValue(String, String),
    ValueGreaterThan(String, String),
    ValueGreaterThanEqual(String, String),
    ValueLessThan(String, String),
    ValueLessThanEqual(String, String),
    ClosedPath,

    List(Vec<SelectorCondition>),
}

#[derive(Debug)]
pub enum Selector {
    Any(SelectorCondition),
    Meta(SelectorCondition),
    Node(SelectorCondition),
    Way(SelectorCondition),
    Relation(SelectorCondition),
    Area(SelectorCondition),
    Line(SelectorCondition),
    Canvas(SelectorCondition),
}
