use std::num::NonZeroI64;

#[derive(Debug)]
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
#[derive(Debug)]
pub struct NodeData {
    pub nid: NonZeroI64,
    pub lat: f64,
    pub lon: f64,
    pub tags: Vec<(String, String)>,
    pub way: Option<NonZeroI64>,
}
#[derive(Debug)]
pub struct WayData {
    pub wid: NonZeroI64,
    pub tags: Vec<(String, String)>,
    pub refs: Vec<NonZeroI64>,
}

impl WayData {
    /// returns true if this way encloses an area (i.e. the first and the last node is the same)
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        debug_assert!(self.refs.len() >= 2);

        self.refs[0] == self.refs[self.refs.len() - 1]
    }
}
