use crate::representations::loops::Loop;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Mapping {
    pub mapping: HashMap<String, Option<MappingType>>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum MappingType {
    Spatial,
    InterTile,
    IntraTile,
}

impl Mapping {
    pub fn from_yaml(yaml_str: &str) -> Self {
        serde_yaml::from_str(yaml_str).expect("Failed to deserialize YAML")
    }

    pub fn add_mapping(mut self, loop_: &Loop, mapping_type: MappingType) -> Self {
        self.mapping
            .insert(loop_.iter_name.clone(), Some(mapping_type));
        self
    }

    pub fn get_mapping_by_iter_name(&self, iter_name: &str) -> Option<&MappingType> {
        self.mapping.get(iter_name).unwrap().as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn test_deserialize() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // Construct the file path to loopprob.yaml
        let file_path = Path::new(manifest_dir).join("example/mapping.yaml");
        println!("File path: {:?}", file_path);
        let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
        let mapping = Mapping::from_yaml(&yaml_str);
        println!("{:#?}", mapping);
    }
}
