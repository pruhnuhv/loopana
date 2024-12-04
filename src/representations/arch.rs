use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Arch {
    pub dims_name: Vec<String>,
    pub dims_shape: Vec<i32>,
    pub pe_arch: PEArch,
}

#[derive(Debug, Deserialize)]
pub struct PEArch {
    pub data_ports: Vec<DataPort>,
    pub data_width: i32,
}

#[derive(Debug, Deserialize)]
pub enum DataPort {
    NocPort(NocPort),
    MemoryWritePort(MemoryPort),
    MemoryReadPort(MemoryPort),
}

#[derive(Debug, Deserialize)]
pub struct NocPort {
    pub name: String,
    pub topology: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryPort {
    pub name: String,
    pub mem_name: String,
}

pub enum ControlType {
    SkipNZ,
    Shuffle,
}
pub struct Control {
    pub ctrl_type: ControlType,
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
        let file_path = Path::new(manifest_dir).join("example/arch.yaml");
        println!("File path: {:?}", file_path);
        let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
        let arch: Arch = serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
        println!("{:#?}", arch);
    }
}
