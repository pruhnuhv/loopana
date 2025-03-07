use core::fmt;

use crate::representations::{affine_expr::AffineExpr, instruction::Instruction};

use super::property::Property;

use super::passes::{PassInfo, PassRun};
use super::workspace::Workspace;

pub struct MemAccessProp {
    pub accessed_dims: Vec<String>,
}

impl Property for MemAccessProp {
    fn property_id(&self) -> String {
        "MemAccessProp".to_string()
    }
}

impl fmt::Display for MemAccessProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Accessed Dims: {{{}}}", self.accessed_dims.join(", "))
    }
}
pub struct MemAccessAnalysis;

impl MemAccessAnalysis {
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

impl PassRun for MemAccessAnalysis {
    fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for inst in workspace.loop_nest.body.clone().iter() {
            let properties = match inst {
                Instruction::DataLoad(mem_access) | Instruction::DataStore(mem_access) => {
                    let accessed_dims = mem_access
                        .addr
                        .iter()
                        .flat_map(|expr| Self::get_accesses_from_affine_expr(expr))
                        .collect();
                    vec![Box::new(MemAccessProp { accessed_dims })]
                }
                _ => vec![Box::new(MemAccessProp {
                    accessed_dims: vec![],
                })],
            };
            for property in properties {
                workspace.add_property(inst, property)
            }
        }
        Ok(())
    }
    

    fn setup(&mut self, _workspace: &mut Workspace) -> Result<(), &'static str> {
        Ok(())
    }
}

impl PassInfo for MemAccessAnalysis {
    fn name(&self) -> &str {
        "Memory Access Analysis"
    }

    fn description(&self) -> &str {
        "Memory Access Analysis"
    }

    fn required_features(&self) -> Vec<String> {
        vec![]
    }

    fn produced_features(&self) -> Vec<String> {
        vec!["MemAccess".to_string()]
    }
}
