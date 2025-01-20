use core::fmt;

use crate::representations::{affine_expr::AffineExpr, instruction::Instruction};
use super::property::Property;
use super::passes::{PassInfo, PassRun};
use super::workspace::Workspace;

pub struct FreeDimProp {
    pub free_dims: Vec<String>,
}

impl Property for FreeDimProp {
    fn property_id(&self) -> String {
        "FreeDimProp".to_string()
    }
}

impl fmt::Display for FreeDimProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Free Dims: {{{}}}", self.free_dims.join(", "))
    }
}

pub struct FreeDimAnalysis;

impl FreeDimAnalysis {
    fn get_accesses_from_affine_expr(expr: &AffineExpr) -> Vec<String> {
        match expr {
            AffineExpr::Var(var) => {
                vec![var.clone()]
            }
            AffineExpr::Const(_) => {
                vec![]
            }
            AffineExpr::Add(expr1, expr2) => {
                let mut accesses = Self::get_accesses_from_affine_expr(expr1);
                accesses.extend(Self::get_accesses_from_affine_expr(expr2));
                accesses
            }
            AffineExpr::Sub(expr1, expr2) => {
                let mut accesses = Self::get_accesses_from_affine_expr(expr1);
                accesses.extend(Self::get_accesses_from_affine_expr(expr2));
                accesses
            }
            AffineExpr::Mul(_, expr) => Self::get_accesses_from_affine_expr(expr),
            AffineExpr::Div(expr, _) => Self::get_accesses_from_affine_expr(expr),
            AffineExpr::Mod(expr, _) => Self::get_accesses_from_affine_expr(expr),
        }
    }
}

impl PassRun for FreeDimAnalysis {
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        let iter_names: Vec<String> = workspace
            .loop_nest
            .iters
            .iter()
            .map(|iter| iter.iter_name.clone())
            .collect();

        for inst in workspace.loop_nest.body.clone().iter() {
            let mut accessed_dims = Vec::new();

            match inst {
                Instruction::DataLoad(mem_access) | Instruction::DataStore(mem_access) => {
                    for expr in &mem_access.addr {
                        accessed_dims.extend(Self::get_accesses_from_affine_expr(expr));
                    }
                }
                _ => {}
            }

            // accessed_dims.dedup();
            let free_dims: Vec<String> = iter_names
                .clone()
                .into_iter()
                .filter(|dim| !accessed_dims.contains(dim))
                .collect();
            
            workspace.add_property(inst, Box::new(FreeDimProp { free_dims }));
        }
        Ok(())
    }

    fn setup(&mut self, _workspace: &mut Workspace) -> Result<(), &'static str> {
        Ok(())
    }
}

impl PassInfo for FreeDimAnalysis {
    fn name(&self) -> &str {
        "Free Dimension Analysis"
    }

    fn description(&self) -> &str {
        "Identifies dimensions in the loop nest that are not accessed in memory operations"
    }

    fn required_features(&self) -> Vec<String> {
        vec![]
    }

    fn produced_features(&self) -> Vec<String> {
        vec!["FreeDims".to_string()]
    }
}

