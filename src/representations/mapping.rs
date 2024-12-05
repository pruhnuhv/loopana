use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mapping {
    pub mapping: HashMap<String, Option<MappingType>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum MappingType {
    Spatial,
    InterTile,
    IntraTile,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::{fs, path::Path};

//     #[test]
//     fn test_deserialize() {
//         let manifest_dir = env!("CARGO_MANIFEST_DIR");
//         // Construct the file path to loopprob.yaml
//         let file_path = Path::new(manifest_dir).join("example/mapping.yaml");
//         println!("File path: {:?}", file_path);
//         let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
//         let mapping = Mapping::from_yaml(&yaml_str);
//         println!("{:#?}", mapping);
//     }
// }
