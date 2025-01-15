use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::representations::instruction::Instruction;
use crate::representations::loops::LoopNest;
use crate::representations::{arch::Arch, loops::LoopIter};

use super::feature::Feature;
use super::property::{Property, PropertyHook};

pub struct Workspace {
    pub properties: HashMap<String, Vec<Box<dyn Property>>>,
    pub loop_nest: LoopNest,
    pub arch: Option<Arch>,
    pub available_features: Vec<Feature>,
}

impl Workspace {
    pub fn new(loop_nest: LoopNest, arch: Option<Arch>) -> Self {
        Workspace {
            properties: HashMap::new(),
            loop_nest,
            arch,
            available_features: Vec::new(),
        }
    }

    pub fn add_property(&mut self, property_hook: impl PropertyHook, property: Box<dyn Property>) {
        let property_hook_id = property_hook.property_hook_id();
        self.properties
            .entry(property_hook_id)
            .or_insert(Vec::new())
            .push(property);
    }

    pub fn add_global_property(&mut self, property: Box<dyn Property>) {
        self.properties
            .entry(self.property_hook_id())
            .or_insert(Vec::new())
            .push(property);
    }

    pub fn get_properties(
        &self,
        property_hook: impl PropertyHook,
    ) -> Option<&Vec<Box<dyn Property>>> {
        self.properties.get(&property_hook.property_hook_id())
    }

    // Find LoopIter index in LoopNest
    fn find_iter_index(&self, iter: &LoopIter) -> Option<usize> {
        self.loop_nest
            .iters
            .iter()
            .position(|x| std::ptr::eq(x, iter))
    }

    // Find Instruction index in LoopNest
    fn find_instruction_index(&self, instruction: &Instruction) -> Option<usize> {
        self.loop_nest
            .body
            .iter()
            .position(|x| std::ptr::eq(x, instruction))
    }

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
        Ok(())
    }
}

impl PropertyHook for Workspace {
    fn property_hook_id(&self) -> String {
        "Workspace".to_string()
    }
}
