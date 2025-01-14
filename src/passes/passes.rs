use std::fmt::{write, Display};

use crate::representations::{
    arch::Arch,
    instruction::Instruction,
    loops::{LoopIter, LoopNest},
};

use super::{
    feature::Feature,
    property::{InstProperty, IterProperty, LoopProperty, WorkspaceProperty},
    workspace::Workspace,
};

pub trait PassInfo {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_features(&self) -> Vec<String>;
    fn produced_features(&self) -> Vec<String>;
}

pub trait PassRun {
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str>;
}

pub trait Pass: PassInfo + PassRun {}
impl<T> Pass for T where T: PassInfo + PassRun {}

pub trait InstAnalysis {
    fn analyze_inst(&self, inst: &Instruction) -> Vec<Box<dyn InstProperty>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for inst in workspace.loop_nest.body.iter() {
            let properties = self.analyze_inst(inst);
            for property in properties {
                workspace.add_inst_property_for(inst, property)?;
            }
        }
        Ok(())
    }
}

pub trait IterAnalysis {
    fn analyze_iter(&self, iter: &LoopIter) -> Vec<Box<dyn IterProperty>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for iter in workspace.loop_nest.iters.iter() {
            let properties = self.analyze_iter(iter);
            for property in properties {
                workspace.add_iter_property_for(iter, property)?;
            }
        }
        Ok(())
    }
}

pub trait LoopAnalysis {
    fn analyze_loop(&self, loop_nest: &LoopNest) -> Vec<Box<dyn LoopProperty>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        let properties = self.analyze_loop(workspace.loop_nest);
        for property in properties {
            workspace.add_loop_property(property);
        }
        Ok(())
    }
}

pub trait WorkspaceAnalysis {
    fn analyze_workspace(&self, workspace: &mut Workspace) -> Vec<Box<dyn LoopProperty>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        let properties = self.analyze_workspace(workspace);
        for property in properties {
            workspace.add_loop_property(property);
        }
        Ok(())
    }
}

pub trait InstTransform {
    fn transform_inst(&self, inst: &Instruction) -> Instruction;
}

pub trait IterTransform {
    fn transform_iter(&self, iter: &LoopIter) -> LoopIter;
}

pub trait LoopTransform {
    fn transform_loop(&self, loop_nest: &LoopNest) -> LoopNest;
}
