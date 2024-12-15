use crate::representations::{
    instruction::Instruction,
    loops::{LoopIter, LoopNest},
    property::*,
};
pub struct Workspace<'a> {
    pub iter_properties: Vec<Vec<Box<dyn IterProperty>>>,
    pub inst_properties: Vec<Vec<Box<dyn InstProperty>>>,
    pub loop_properties: Vec<Box<dyn LoopProperty>>,
    pub loop_nest: &'a LoopNest,
}
pub trait Pass {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_properties(&self) -> Vec<Box<dyn Property>>;
    fn produced_properties(&self) -> Vec<Box<dyn Property>>;
    fn run(&self, workspace: &mut Workspace);
}

pub trait InstAnalysis: Pass {
    fn analyze_inst(&self, inst: &Instruction) -> Vec<Box<dyn InstProperty>>;
    fn analyze(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for inst in workspace.loop_nest.body.iter() {
            let properties = self.analyze_inst(inst);
            for property in properties {
                workspace.add_inst_property_for(inst, property)?;
            }
        }
        Ok(())
    }
}

pub trait IterAnalysis: Pass {
    fn analyze_iter(&self, iter: &LoopIter) -> Vec<Box<dyn IterProperty>>;
    fn analyze(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for iter in workspace.loop_nest.iters.iter() {
            let properties = self.analyze_iter(iter);
            for property in properties {
                workspace.add_iter_property_for(iter, property)?;
            }
        }
        Ok(())
    }
}

pub trait LoopAnalysis: Pass {
    fn analyze_loop(&self, loop_nest: &LoopNest) -> Vec<Box<dyn LoopProperty>>;
    fn analyze(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        let properties = self.analyze_loop(workspace.loop_nest);
        for property in properties {
            workspace.add_loop_property(property);
        }
        Ok(())
    }
}

pub trait InstTransform: Pass {
    fn transform_inst(&self, inst: &Instruction) -> Instruction;
}

pub trait IterTransform: Pass {
    fn transform_iter(&self, iter: &LoopIter) -> LoopIter;
}

pub trait LoopTransform: Pass {
    fn transform_loop(&self, loop_nest: &LoopNest) -> LoopNest;
}

impl<'a> Workspace<'a> {
    pub fn new(loop_nest: &'a LoopNest) -> Self {
        let num_iter = loop_nest.iters.len();
        let num_inst = loop_nest.body.len();
        Workspace {
            iter_properties: (0..num_iter).map(|_| Vec::new()).collect(),
            inst_properties: (0..num_inst).map(|_| Vec::new()).collect(),
            loop_properties: Vec::new(),
            loop_nest,
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
}
