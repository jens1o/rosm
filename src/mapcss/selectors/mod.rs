use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
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
    ValueGreaterThan(String, String),
    ValueGreaterThanEqual(String, String),
    ValueLessThan(String, String),
    ValueLessThanEqual(String, String),
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

        if new == No {
            return self;
        }

        match self {
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

#[derive(Debug, PartialEq, Clone)]
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

impl Selector {
    pub fn set_conditions(&self, conditions: SelectorCondition) -> Selector {
        use Selector::*;

        match self {
            Any(_) => Any(conditions),
            Meta(_) => Meta(conditions),
            Node(_) => Node(conditions),
            Way(_) => Way(conditions),
            Relation(_) => Relation(conditions),
            Area(_) => Area(conditions),
            Line(_) => Line(conditions),
            Canvas(_) => Canvas(conditions),
        }
    }

    pub fn conditions(self) -> SelectorCondition {
        use Selector::*;

        match self {
            Any(cond) => cond,
            Meta(cond) => cond,
            Node(cond) => cond,
            Way(cond) => cond,
            Relation(cond) => cond,
            Area(cond) => cond,
            Line(cond) => cond,
            Canvas(cond) => cond,
        }
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
