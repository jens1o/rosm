use crate::data::{NodeData, RelationData, RelationMember, RelationMemberType, WayData};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroI64;

/// Extracts all data available, with a focus on completeness. That is that RelationData only contains elements that also exist in the other datasets.
pub fn extract_data_from_filepath(
    file_path: String,
) -> Result<
    (
        HashMap<NonZeroI64, NodeData>,
        HashMap<NonZeroI64, WayData>,
        HashMap<NonZeroI64, RelationData>,
    ),
    osmpbf::Error,
> {
    let reader = osmpbf::ElementReader::from_path(file_path)?;

    let mut nid_to_node_data: HashMap<NonZeroI64, NodeData> = HashMap::new();
    let mut wid_to_way_data: HashMap<NonZeroI64, WayData> = HashMap::new();
    let mut rid_to_relation_data: HashMap<NonZeroI64, RelationData> = HashMap::new();

    let mut relation_types: HashMap<String, u32> = HashMap::new();

    #[derive(Debug)]
    struct MostTaggedNode {
        id: Option<NonZeroI64>,
        tag_count: usize,
    }

    let mut most_tagged_node = MostTaggedNode {
        id: None,
        tag_count: 0,
    };

    reader.for_each(|element| {
        if let osmpbf::Element::Relation(relation) = element {
            let rid = NonZeroI64::new(relation.id()).expect("A relation must not have the ID 0!");
            let mut members = Vec::with_capacity(relation.members().len());

            for relation_member in relation.members() {
                let member_id = NonZeroI64::new(relation_member.member_id)
                    .expect("Any member of a relation must not have an ID of zero!");

                use osmpbf::elements::RelMemberType;
                members.push(RelationMember {
                    member: match relation_member.member_type {
                        RelMemberType::Relation => RelationMemberType::Relation(member_id),
                        RelMemberType::Node => RelationMemberType::Node(member_id),
                        RelMemberType::Way => RelationMemberType::Way(member_id),
                    },
                    role: relation_member.role().unwrap_or("").to_string(),
                });
            }

            rid_to_relation_data.insert(
                rid,
                RelationData {
                    rid,
                    tags: relation
                        .tags()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        // TODO: Debug
                        .inspect(|(k, v)| {
                            if k == &"type" {
                                *relation_types.entry(v.to_string()).or_default() += 1;
                            }
                        })
                        .collect::<Vec<_>>(),
                    members,
                },
            );
        } else if let osmpbf::Element::Node(_) = element {
            panic!("OSM-Nodes not supported (yet), use data extractions with DenseNodes instead!");
        } else if let osmpbf::Element::DenseNode(node) = element {
            let mut tags = node
                .tags()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>();
            tags.sort();

            if tags.len() > most_tagged_node.tag_count {
                most_tagged_node.id =
                    Some(NonZeroI64::new(node.id).expect("Node id must not be zero!"));
                most_tagged_node.tag_count = tags.len();
            }

            nid_to_node_data.insert(
                NonZeroI64::new(node.id).expect("Node id must not zero!"),
                NodeData {
                    // unwrap is safe because we would panic in inserting the key into this hashmap already
                    nid: NonZeroI64::new(node.id).unwrap(),
                    tags,
                    lat: node.lat(),
                    lon: node.lon(),
                    way: None,
                },
            );
        } else if let osmpbf::Element::Way(way) = element {
            let wid = NonZeroI64::new(way.id()).expect("Way id must not be zero!");
            let ref_len = way.refs().len();

            // ignore invalid ways
            if ref_len < 2 {
                warn!(
                    "Dropped way #{} as it has less than two referenced nodes!",
                    wid
                );
                return;
            }

            let mut refs = Vec::with_capacity(ref_len);

            for reference in way.refs() {
                refs.push(NonZeroI64::new(reference).expect("Node id must not be zero"));
            }

            wid_to_way_data.insert(
                wid,
                WayData::new(
                    wid,
                    way.tags()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<Vec<_>>(),
                    refs,
                ),
            );
        }
    })?;

    wid_to_way_data.values().for_each(|way_data| {
        way_data.refs().iter().for_each(|nid| {
            if let Some(node_data) = nid_to_node_data.get_mut(nid) {
                node_data.way = Some(way_data.way_id());
            } else {
                panic!(
                    "No node found for #{} (belonging to way #{})!",
                    nid,
                    way_data.way_id()
                );
            }
        });
    });

    nid_to_node_data.shrink_to_fit();
    wid_to_way_data.shrink_to_fit();
    rid_to_relation_data.shrink_to_fit();

    dbg!(relation_types);
    dbg!(most_tagged_node);

    // check whether we have all the data for the relations referencing them
    // otherwise remove them (because we possibly only deal with a data extract)
    let relation_ids = rid_to_relation_data.keys().copied().collect::<HashSet<_>>();
    rid_to_relation_data.retain(|_, relation_data| {
        for rel_member in &relation_data.members {
            let entry_exists = match rel_member.member {
                RelationMemberType::Way(wid) => wid_to_way_data.contains_key(&wid),
                RelationMemberType::Relation(rid) => relation_ids.contains(&rid),
                RelationMemberType::Node(nid) => nid_to_node_data.contains_key(&nid),
            };

            if !entry_exists {
                return false;
            }
        }

        true
    });

    Ok((nid_to_node_data, wid_to_way_data, rid_to_relation_data))
}
