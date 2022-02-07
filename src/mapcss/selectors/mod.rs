use std::cmp::Eq;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum SelectorCondition {
    No,

    ExactZoomLevel(u8),
    MinZoomLevel(u8),
    RangeZoomLevel(u8, u8),
    MaxZoomLevel(u8),
    Not(Rc<Selector>),
    HasDescendant(Rc<Selector>),
    GenericPseudoClass(String),
    HasTag(String),
    HasExactTagValue(String, String),
    HasNotTagValue(String, String),
    ValueGreaterThan(String, isize),
    ValueGreaterThanEqual(String, isize),
    ValueLessThan(String, isize),
    ValueLessThanEqual(String, isize),
    ClosedPath,

    List(Vec<SelectorCondition>),
}

impl Default for SelectorCondition {
    fn default() -> Self {
        SelectorCondition::No
    }
}

impl SelectorCondition {
    /// Merges two conditionsets together
    pub fn add_condition(self, new: SelectorCondition) -> SelectorCondition {
        use SelectorCondition::*;

        // TODO: Check whether merging the two conditions actually makes sense and simplify
        // e.g. merge(SelectorCondition::MaxZoomLevel(10), SelectorCondition::MinZoomLevel(11)) => SelectorCondition::No

        if new == No {
            return self;
        }

        match self {
            // merge lists together instead of creating sublists
            List(mut conditions) => {
                if let List(new_conditions) = new {
                    conditions.extend(new_conditions);
                } else {
                    conditions.push(new);
                }

                SelectorCondition::List(conditions)
            }
            No => new,

            _ => SelectorCondition::List(vec![self, new]),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Selector {
    selector_type: SelectorType,
    conditions: SelectorCondition,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Clone, Eq, Hash, Copy)]
pub enum SelectorType {
    Any,
    Meta,
    Node,
    Way,
    Relation,
    Area,
    Line,
    Canvas,
}

impl Selector {
    pub fn new(selector_type: SelectorType, conditions: SelectorCondition) -> Selector {
        Selector {
            selector_type,
            conditions,
        }
    }

    pub fn set_conditions(&mut self, conditions: SelectorCondition) {
        self.conditions = conditions;
    }

    pub fn conditions(&self) -> &SelectorCondition {
        &self.conditions
    }

    pub fn selector_type(&self) -> SelectorType {
        self.selector_type
    }
}

#[cfg(test)]
mod tests {
    use super::SelectorCondition;

    #[test]
    fn test_selector_condition_merge_no_simple() {
        let a = SelectorCondition::No;

        let b = SelectorCondition::HasExactTagValue("jens".into(), "awesome".into());

        assert_eq!(a.add_condition(b.clone()), b);
    }

    #[test]
    fn test_selector_condition_merge_no() {
        let a = SelectorCondition::No;

        let b = SelectorCondition::List(vec![
            SelectorCondition::HasExactTagValue("jens".into(), "awesome".into()),
            SelectorCondition::ClosedPath,
        ]);

        // b should still win no matter the order
        assert_eq!(a.clone().add_condition(b.clone()), b);
        assert_eq!(b.clone().add_condition(a.clone()), b);
    }

    #[test]
    fn test_selector_condition_lists() {
        let a = SelectorCondition::List(vec![
            SelectorCondition::MinZoomLevel(8),
            SelectorCondition::MaxZoomLevel(10),
        ]);

        let b = SelectorCondition::List(vec![
            SelectorCondition::HasExactTagValue("jens".into(), "awesome".into()),
            SelectorCondition::ClosedPath,
        ]);

        assert_eq!(
            a.clone().add_condition(b.clone()),
            SelectorCondition::List(vec![
                SelectorCondition::MinZoomLevel(8),
                SelectorCondition::MaxZoomLevel(10),
                SelectorCondition::HasExactTagValue("jens".into(), "awesome".into()),
                SelectorCondition::ClosedPath,
            ])
        );

        assert_eq!(
            b.clone().add_condition(a.clone()),
            SelectorCondition::List(vec![
                SelectorCondition::HasExactTagValue("jens".into(), "awesome".into()),
                SelectorCondition::ClosedPath,
                SelectorCondition::MinZoomLevel(8),
                SelectorCondition::MaxZoomLevel(10),
            ])
        );
    }
}
