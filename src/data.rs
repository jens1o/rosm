use kuchiki::{Attribute, ExpandedName, NodeRef};
use markup5ever::{LocalName, Namespace, QualName};

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

/// Holds an extract of the data from the Protobuf-file,
/// containing application-important data over nodes.
#[derive(Debug)]
pub struct NodeData {
    pub nid: i64,
    pub lat: f64,
    pub lon: f64,
    pub tags: Vec<(String, String)>,
    pub way: Option<i64>,
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
    pub wid: i64,
    pub tags: Vec<(String, String)>,
    pub refs: Vec<i64>,
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
