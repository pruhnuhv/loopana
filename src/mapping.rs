use crate::loop_prob::Loop;
use std::collections::HashMap;

pub struct Mapping {
    pub mapping: HashMap<String, MappingType>,
}

pub enum MappingType {
    Spatial,
    InterTile,
    IntraTile,
}

impl Mapping {
    pub fn new() -> Self {
        Mapping {
            mapping: HashMap::new(),
        }
    }

    pub fn add_mapping(mut self, loop_: &Loop, mapping_type: MappingType) -> Self {
        self.mapping.insert(loop_.iter_name.clone(), mapping_type);
        self
    }

    pub fn get_mapping_by_iter_name(&self, iter_name: &str) -> Option<&MappingType> {
        self.mapping.get(iter_name)
    }
}
