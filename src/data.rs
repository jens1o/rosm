use std::fmt;
use std::num::NonZeroI64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementID {
    Canvas,
    Relation(NonZeroI64),
    Node(NonZeroI64),
    Way(NonZeroI64),
}

impl From<&NodeData> for ElementID {
    fn from(val: &NodeData) -> Self {
        ElementID::Node(val.nid)
    }
}

impl From<WayData> for ElementID {
    fn from(val: WayData) -> Self {
        ElementID::Way(val.wid)
    }
}

impl From<&RelationData> for ElementID {
    fn from(val: &RelationData) -> Self {
        ElementID::Relation(val.rid)
    }
}

#[derive(Debug, Clone)]
pub enum RelationMemberType {
    Relation(NonZeroI64),
    Node(NonZeroI64),
    Way(NonZeroI64),
}

#[derive(Debug, Clone)]
pub struct RelationMember {
    pub member: RelationMemberType,
    // TODO: Transform to enum?
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct RelationData {
    pub rid: NonZeroI64,
    pub tags: Vec<(String, String)>,
    pub members: Vec<RelationMember>,
}

/// Holds an extract of the data from the Protobuf-file,
/// containing application-important data over nodes.
#[derive(Debug, Clone)]
pub struct NodeData {
    pub nid: NonZeroI64,
    pub lat: f64,
    pub lon: f64,
    /// Guaranteed to be sorted
    pub tags: Vec<(String, String)>,
    pub way: Option<NonZeroI64>,
}
#[derive(Debug, Clone)]
pub struct WayData {
    pub wid: NonZeroI64,
    /// true if this way encloses an area (i.e. the first and the last node is the same) or area=yes is supplied
    is_closed: bool,

    tags: Vec<(String, String)>,
    refs: Vec<NonZeroI64>,
}

impl WayData {
    pub fn new(wid: NonZeroI64, tags: Vec<(String, String)>, refs: Vec<NonZeroI64>) -> WayData {
        assert!(refs.len() >= 2);

        WayData {
            wid,
            is_closed: refs.get(0).unwrap() == refs.get(refs.len() - 1).unwrap()
                || tags
                    .iter()
                    .any(|(tag_key, tag_value)| tag_key == "area" && tag_value == "yes"),
            tags,
            refs,
        }
    }

    pub fn way_id(&self) -> NonZeroI64 {
        self.wid
    }

    pub fn refs(&self) -> &Vec<NonZeroI64> {
        &self.refs
    }
}

pub trait ElementData: fmt::Debug {
    fn tags(&self) -> &[(String, String)];

    fn id(&self) -> ElementID;

    fn has_closed_path(&self) -> bool;
}

impl ElementData for WayData {
    fn tags(&self) -> &[(String, String)] {
        &self.tags
    }

    fn id(&self) -> ElementID {
        ElementID::Way(self.wid)
    }

    fn has_closed_path(&self) -> bool {
        self.is_closed
    }
}

impl ElementData for NodeData {
    fn tags(&self) -> &[(String, String)] {
        &self.tags
    }

    fn id(&self) -> ElementID {
        self.into()
    }

    fn has_closed_path(&self) -> bool {
        false
    }
}

impl ElementData for RelationData {
    fn tags(&self) -> &[(String, String)] {
        &self.tags
    }

    fn id(&self) -> ElementID {
        self.into()
    }

    fn has_closed_path(&self) -> bool {
        todo!();
    }
}
