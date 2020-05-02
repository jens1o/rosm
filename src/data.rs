use std::num::NonZeroI64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementID {
    Relation(NonZeroI64),
    Node(NonZeroI64),
    Way(NonZeroI64),
}

impl Into<ElementID> for NodeData {
    fn into(self) -> ElementID {
        ElementID::Node(self.nid)
    }
}

impl Into<ElementID> for &WayData {
    fn into(self) -> ElementID {
        ElementID::Way(self.way_id())
    }
}

impl Into<ElementID> for RelationData {
    fn into(self) -> ElementID {
        ElementID::Relation(self.rid)
    }
}

#[derive(Debug, Clone)]
pub enum RelationMemberType {
    Relation(NonZeroI64),
    Node(NonZeroI64),
    Way(NonZeroI64),
}

#[derive(Debug)]
pub struct RelationMember {
    pub member: RelationMemberType,
    // TODO: Transform to enum?
    pub role: String,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct WayData {
    wid: NonZeroI64,
    /// true if this way encloses an area (i.e. the first and the last node is the same)
    is_closed: bool,

    tags: Vec<(String, String)>,
    refs: Vec<NonZeroI64>,
}

impl WayData {
    pub fn new(wid: NonZeroI64, tags: Vec<(String, String)>, refs: Vec<NonZeroI64>) -> WayData {
        debug_assert!(refs.len() >= 2);

        WayData {
            wid,
            is_closed: refs.get(0).unwrap() == refs.get(refs.len() - 1).unwrap(),
            tags,
            refs,
        }
    }

    pub fn way_id(&self) -> NonZeroI64 {
        self.wid
    }

    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn refs(&self) -> &Vec<NonZeroI64> {
        &self.refs
    }

    pub fn tags(&self) -> &[(String, String)] {
        &self.tags
    }
}
