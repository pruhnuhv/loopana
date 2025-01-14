use core::fmt;

use super::passes::*;
use super::workspace::Workspace;
use crate::representations::arch::Arch;

use crate::passes::property::*;
use crate::representations::loops::LoopNest;

#[derive(Clone)]
pub struct ArchInfo {
    pub arch: Arch,
}

impl Property for ArchInfo {
    fn name(&self) -> &str {
        "ArchInfo"
    }

    fn description(&self) -> &str {
        "Information about the architecture"
    }
}

impl fmt::Display for ArchInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let arch_str = serde_yaml::to_string(&self.arch).unwrap();
        write!(f, "{}", arch_str)
    }
}

impl LoopProperty for ArchInfo {
    fn to_str(&self, loop_nest: &LoopNest) -> String {
        format!("Arch: {}", self)
    }
}

pub struct ArchInfoBuilder {
    pub arch_info: ArchInfo,
}

impl ArchInfoBuilder {
    pub fn from_file(file_path: &str) -> Self {
        let arch = serde_yaml::from_str(file_path).unwrap();
        let arch_info = ArchInfo { arch };
        Self { arch_info }
    }
}

impl PassInfo for ArchInfoBuilder {
    fn name(&self) -> &str {
        "ArchInfoBuilder"
    }

    fn description(&self) -> &str {
        "Builds the architecture information"
    }

    fn required_features(&self) -> Vec<String> {
        vec![]
    }

    fn produced_features(&self) -> Vec<String> {
        vec!["ArchInfo".to_string()]
    }
}

impl PassRun for ArchInfoBuilder {
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        workspace.add_loop_property(Box::new(self.arch_info.clone()));
        Ok(())
    }
}
