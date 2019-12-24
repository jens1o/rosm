use kuchiki::{Attribute, ExpandedName, NodeRef};
use markup5ever::{LocalName, Namespace, QualName};
use std::num::NonZeroI64;

pub trait ToNodeRef {
    fn node_ref(&self) -> NodeRef {
        NodeRef::new_element(
            QualName::new(None, Namespace::from(""), self.node_ref_local_name()),
            self.node_ref_attributes(),
        )
    }

    fn node_ref_attributes(&self) -> Vec<(ExpandedName, Attribute)>;

    fn node_ref_local_name(&self) -> LocalName;
}

#[derive(Debug)]
pub enum RelationMemberType {
    Relation(NonZeroI64),
    Node(NonZeroI64),
    Way(NonZeroI64),
}

#[derive(Debug)]
pub struct RelationMember {
    pub member_type: RelationMemberType,
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

impl ToNodeRef for NodeData {
    fn node_ref_attributes(&self) -> Vec<(ExpandedName, Attribute)> {
        self.tags
            .iter()
            .map(|(k, v)| {
                (
                    ExpandedName::new::<Namespace, String>(Namespace::from(""), k.into()),
                    Attribute {
                        prefix: None,
                        value: v.into(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }

    fn node_ref_local_name(&self) -> LocalName {
        LocalName::from("node")
    }
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
        debug_assert!(self.refs.len() > 2);

        self.refs[0] == self.refs[self.refs.len() - 1]
    }
}

impl ToNodeRef for WayData {
    fn node_ref_attributes(&self) -> Vec<(ExpandedName, Attribute)> {
        self.tags
            .iter()
            .map(|(k, v)| {
                (
                    ExpandedName::new::<Namespace, String>(Namespace::from(""), k.into()),
                    Attribute {
                        prefix: None,
                        value: v.into(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }

    fn node_ref_local_name(&self) -> LocalName {
        LocalName::from("way")
    }
}
