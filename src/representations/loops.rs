use super::mapping::MappingType;
use std::collections::HashMap;

use super::instruction::Instruction;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoopNest {
    pub iters: Vec<LoopIter>,
    pub body: Vec<Instruction>,
    pub properties: Option<LoopProperties>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoopIter {
    pub iter_name: String,
    pub bounds: (i32, i32),
    pub step: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoopProperties {
    pub mapping: HashMap<String, MappingType>,
}

#[cfg(test)]
mod tests {
    use serde_yaml;
    use std::{fs, path::Path};

    use super::*;

    #[test]
    fn test_deserialize() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // Construct the file path to loopprob.yaml
        let file_path = Path::new(manifest_dir).join("example/prob.yaml");
        println!("File path: {:?}", file_path);
        let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
        let loop_prob: LoopNest =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
        println!("{:#?}", loop_prob);
    }

    #[test]
    fn test_serde() {
        let loop_prob_str = r#"iters:
  - iter_name: "m"
    bounds: [0, 100]
    step: 1
  - iter_name: "k"
    step: 1
    bounds: [0, 300]
  - iter_name: "n"
    bounds: [0, 200]
    step: 1
body:
  - !DataLoad
    array_name: "A"
    addr: "k + 300m"
    reg: "Ra"
  - !DataLoad
    array_name: "B"
    addr: "n + 200k"
    reg: "Rb"
  - !DataLoad
    array_name: "C"
    addr: "n + 200m"
    reg: "Rc"
  - !Compute
    op: "mac"
    src: ["Ra", "Rb", "Rc"]
    dst: "Rc"
  - !DataStore
    array_name: "store_C"
    addr: "n + 200m"
    reg: "Rc"

conditionals:
  - cond_comp:
      - !DataLoad
        array_name: "load_A"
        addr: "k + 300m"
        reg: "Ra"
      - !Compute
        op: "cmp"
        src: ["Ra"]
        dst: "Rcmp"
    skipped_loops: ["n"]
    prob: 0.5
    "#;
        let loop_prob: LoopNest = serde_yaml::from_str(loop_prob_str).unwrap();
        let serialized = serde_yaml::to_string(&loop_prob).unwrap().clone();
        let deserialized: LoopNest = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(loop_prob, deserialized);
    }
}
