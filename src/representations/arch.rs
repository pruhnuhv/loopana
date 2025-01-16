use std::fmt::Display;

use crate::passes::property::PropertyHook;
use property_hood_id_derive::PropertyHook;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Dimension {
    pub name: String,
    pub shape: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PropertyHook)]
pub struct Arch {
    pub pe_arch: PEArch,
    pub dimensions: Vec<Dimension>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PropertyHook)]
pub struct PEArch {
    pub data_ports: Vec<DataPort>,
    pub data_width: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum DataPort {
    NocPort(NocPort),
    MemoryWritePort(MemoryPort),
    MemoryReadPort(MemoryPort),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct NocPort {
    pub name: String,
    pub topology: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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

impl Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arch: \n{}\n", serde_yaml::to_string(self).unwrap())?;
        Ok(())
    }
}

impl Display for PEArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PEArch: \n{}\n", serde_yaml::to_string(self).unwrap())?;
        Ok(())
    }
}

impl Arch {
    pub fn data_ports(&self) -> &Vec<DataPort> {
        &self.pe_arch.data_ports
    }
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
        let file_path = Path::new(manifest_dir).join("example/mesh_distributed-mem.arch");
        let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
        let _arch: Arch = serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
    }
}
