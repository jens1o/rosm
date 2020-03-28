use super::declaration::MapCssDeclaration;
use super::selectors::Selector;

#[derive(Debug)]
pub struct MapCssRule {
    source_order: usize,
    selector: Selector,
    rules: Vec<MapCssDeclaration>,
}

impl MapCssRule {
    pub fn new(source_order: usize, selector: Selector, rules: Vec<MapCssDeclaration>) -> MapCssRule {
        MapCssRule {
            source_order, selector, rules
        }
    }
}

// TODO: Make thoughts about how to match and representate the data
