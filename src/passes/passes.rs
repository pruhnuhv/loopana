use crate::representations::{instruction::Instruction, loops::LoopIter};

use super::{property::Property, workspace::Workspace};

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

pub trait InstPass: PassRun {
    fn pass_inst(&self, inst: &Instruction) -> Vec<Box<dyn Property>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for inst in workspace.loop_nest.body.clone().iter() {
            let properties = self.pass_inst(inst);
            for property in properties {
                workspace.add_property(inst, property);
            }
        }
        Ok(())
    }
}

pub trait IterPass: PassRun {
    fn pass_iter(&self, iter: &LoopIter) -> Vec<Box<dyn Property>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for iter in workspace.loop_nest.iters.clone().iter() {
            let properties = self.pass_iter(iter);
            for property in properties {
                workspace.add_property(iter, property);
            }
        }
        Ok(())
    }
}

// pub trait LoopAnalysis {
//     fn analyze_loop(&self, loop_nest: &LoopNest) -> Vec<Box<dyn Property>>;
//     fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
//         let properties = self.analyze_loop(workspace.loop_nest);
//         for property in properties {
//             workspace.add_property(loop_nest, property);
//         }
//         Ok(())
//     }
// }

pub trait WorkspacePass: PassRun {
    fn pass_workspace(&self, workspace: &mut Workspace) -> Vec<Box<dyn Property>>;
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        let properties = self.pass_workspace(workspace);
        for property in properties {
            workspace.add_global_property(property);
        }
        Ok(())
    }
}

// pub trait InstTransform {
//     fn transform_inst(&self, inst: &Instruction) -> Instruction;
// }

// pub trait IterTransform {
//     fn transform_iter(&self, iter: &LoopIter) -> LoopIter;
// }

// pub trait LoopTransform {
//     fn transform_loop(&self, loop_nest: &LoopNest) -> LoopNest;
// }
