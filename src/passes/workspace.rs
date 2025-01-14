use std::fmt::Display;

use crate::representations::instruction::Instruction;
use crate::representations::loops::LoopNest;
use crate::representations::{arch::Arch, loops::LoopIter};

use super::{
    feature::Feature,
    property::{InstProperty, IterProperty, LoopProperty, WorkspaceProperty},
};

pub struct Workspace<'a> {
    pub iter_properties: Vec<Vec<Box<dyn IterProperty>>>,
    pub inst_properties: Vec<Vec<Box<dyn InstProperty>>>,
    pub loop_properties: Vec<Box<dyn LoopProperty>>,
    pub workspace_properties: Vec<Box<dyn WorkspaceProperty>>,
    pub loop_nest: &'a LoopNest,
    pub arch: &'a Option<Arch>,
    pub available_features: Vec<Feature>,
}

impl<'a> Workspace<'a> {
    pub fn new(loop_nest: &'a LoopNest, arch: &'a Option<Arch>) -> Self {
        let num_iter = loop_nest.iters.len();
        let num_inst = loop_nest.body.len();
        Workspace {
            iter_properties: (0..num_iter).map(|_| Vec::new()).collect(),
            inst_properties: (0..num_inst).map(|_| Vec::new()).collect(),
            loop_properties: Vec::new(),
            workspace_properties: Vec::new(),
            loop_nest,
            arch,
            available_features: Vec::new(),
        }
    }

    /// Add iterator property at specific index
    pub fn add_iter_property(&mut self, index: usize, property: Box<dyn IterProperty>) {
        if index < self.iter_properties.len() {
            self.iter_properties[index].push(property);
        }
    }

    /// Add loop property at specific index
    pub fn add_inst_property(&mut self, index: usize, property: Box<dyn InstProperty>) {
        if index < self.inst_properties.len() {
            self.inst_properties[index].push(property);
        }
    }

    /// Add loop property
    pub fn add_loop_property(&mut self, property: Box<dyn LoopProperty>) {
        self.loop_properties.push(property);
    }

    pub fn add_workspace_property(&mut self, property: Box<dyn WorkspaceProperty>) {
        self.workspace_properties.push(property);
    }

    // Get iterator properties for given index
    pub fn get_iter_properties(&self, index: usize) -> Option<&Vec<Box<dyn IterProperty>>> {
        self.iter_properties.get(index)
    }

    // Get instruction properties for given index
    pub fn get_inst_properties(&self, index: usize) -> Option<&Vec<Box<dyn InstProperty>>> {
        self.inst_properties.get(index)
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

    // Add iterator property for specific LoopIter
    pub fn add_iter_property_for(
        &mut self,
        iter: &LoopIter,
        property: Box<dyn IterProperty>,
    ) -> Result<(), &'static str> {
        if let Some(index) = self.find_iter_index(iter) {
            self.add_iter_property(index, property);
            Ok(())
        } else {
            Err("LoopIter not found in LoopNest")
        }
    }

    // Add loop property for specific Instruction
    pub fn add_inst_property_for(
        &mut self,
        instruction: &Instruction,
        property: Box<dyn InstProperty>,
    ) -> Result<(), &'static str> {
        if let Some(index) = self.find_instruction_index(instruction) {
            self.add_inst_property(index, property);
            Ok(())
        } else {
            Err("Instruction not found in LoopNest")
        }
    }

    // Get properties for specific LoopIter
    pub fn get_iter_properties_for(&self, iter: &LoopIter) -> Option<&Vec<Box<dyn IterProperty>>> {
        self.find_iter_index(iter)
            .and_then(|idx| self.get_iter_properties(idx))
    }

    // Get properties for specific Instruction
    pub fn get_inst_properties_for(
        &self,
        instruction: &Instruction,
    ) -> Option<&Vec<Box<dyn InstProperty>>> {
        self.find_instruction_index(instruction)
            .and_then(|idx| self.get_inst_properties(idx))
    }

    pub fn has_property(&self, property_name: String) -> bool {
        self.loop_properties
            .iter()
            .any(|p| p.name() == property_name)
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

impl Display for Workspace<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem: \n{}\n", self.loop_nest)?;
        let loop_props: Vec<String> = self
            .loop_properties
            .iter()
            .map(|prop| prop.to_str(self.loop_nest))
            .collect();
        write!(f, "LoopProperties: \n{}", loop_props.join("\n"))?;
        write!(f, "\n\n")?;

        write!(f, "IterProperties:\n")?;
        for iter in self.loop_nest.iters.iter() {
            write!(f, "{}\n", iter)?;
            let iter_index = self.find_iter_index(iter).unwrap();
            let iter_props: Vec<String> = self.iter_properties[iter_index]
                .iter()
                .map(|prop| prop.inline_to_str(iter))
                .collect();
            write!(f, "\t{}\n", iter_props.join("\n"))?;
        }
        write!(f, "\n\n")?;

        write!(f, "InstProperties:\n")?;
        for inst in self.loop_nest.body.iter() {
            write!(f, "{}\n", inst)?;
            let inst_index = self.find_instruction_index(inst).unwrap();
            let inst_props: Vec<String> = self.inst_properties[inst_index]
                .iter()
                .map(|prop| prop.inline_to_str(inst))
                .collect();
            write!(f, "\t{}\n\n", inst_props.join("\t\n"))?;
        }
        write!(f, "\n\n")?;

        Ok(())
    }
}
