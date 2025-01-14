use core::fmt;

use crate::representations::instruction::Instruction;
use crate::representations::loops::LoopNest;

use crate::representations::loops::LoopIter;

use super::workspace::Workspace;

pub trait Property: fmt::Display {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

pub trait InstProperty: Property {
    fn inline_to_str(&self, inst: &Instruction) -> String;
}

pub trait IterProperty: Property {
    fn inline_to_str(&self, iter: &LoopIter) -> String;
}

pub trait LoopProperty: Property {
    fn to_str(&self, loop_nest: &LoopNest) -> String;
}

pub trait WorkspaceProperty: Property {
    fn to_str(&self, workspace: &Workspace) -> String;
}
