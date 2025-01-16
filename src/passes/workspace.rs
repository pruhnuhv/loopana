use std::fmt::Display;

use crate::representations::arch::Arch;
use crate::representations::loops::LoopNest;

use super::feature::Feature;
use super::property::{Property, PropertyHook, PropertyManager};

pub struct Workspace {
    pub properties: PropertyManager,
    pub loop_nest: LoopNest,
    pub arch: Option<Arch>,
    pub available_features: Vec<Feature>,
}

impl Workspace {
    pub fn new(loop_nest: LoopNest, arch: Option<Arch>) -> Self {
        let property_manager = PropertyManager::from_entries(
            loop_nest
                .body
                .iter()
                .map(|inst| inst.property_hook_id().clone())
                .collect(),
        );
        Workspace {
            properties: property_manager,
            loop_nest,
            arch,
            available_features: Vec::new(),
        }
    }

    pub fn add_property(&mut self, property_hook: impl PropertyHook, property: Box<dyn Property>) {
        self.properties
            .add_property_to_hook(property_hook, property);
    }

    pub fn add_global_property(&mut self, property: Box<dyn Property>) {
        self.properties
            .add_property_by_id(self.property_hook_id(), property);
    }

    pub fn get_properties(
        &self,
        property_hook: impl PropertyHook,
    ) -> Option<&Vec<Box<dyn Property>>> {
        self.properties.get_properties_by_hook(property_hook)
    }

    // // Find LoopIter index in LoopNest
    // fn find_iter_index(&self, iter: &LoopIter) -> Option<usize> {
    //     self.loop_nest
    //         .iters
    //         .iter()
    //         .position(|x| std::ptr::eq(x, iter))
    // }

    // // Find Instruction index in LoopNest
    // fn find_instruction_index(&self, instruction: &Instruction) -> Option<usize> {
    //     self.loop_nest
    //         .body
    //         .iter()
    //         .position(|x| std::ptr::eq(x, instruction))
    // }

    pub fn feature_available(&self, feature: &Feature) -> bool {
        self.available_features.contains(feature)
    }

    pub fn feature_available_str(&self, feature_str: &str) -> bool {
        self.available_features
            .iter()
            .any(|feature| feature.name == feature_str)
    }
}

impl Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem: \n{}\n", self.loop_nest)?;
        if let Some(arch) = &self.arch {
            write!(f, "Arch: \n{}\n", arch)?;
        }
        write!(f, "\nIters: \n")?;
        for iter in &self.loop_nest.iters {
            write!(f, "\n - {}\n", iter)?;
            let properties = self.properties.get_properties_by_hook(iter);
            if let Some(properties) = properties {
                for property in properties {
                    write!(f, "\t> {}\n ", property)?;
                }
            } else {
                write!(f, "\t>\n")?;
            }
        }
        write!(f, "\nBody: \n")?;
        for inst in &self.loop_nest.body {
            write!(f, "\n - {}\n", inst)?;
            let properties = self.properties.get_properties_by_hook(inst);
            if let Some(properties) = properties {
                for property in properties {
                    write!(f, "\t> {}\n ", property)?;
                }
            } else {
                write!(f, "\t>\n")?;
            }
        }
        Ok(())
    }
}

impl PropertyHook for Workspace {
    fn property_hook_id(&self) -> String {
        "Workspace".to_string()
    }
}
