use core::fmt;
use pass_derive::InstPass;

use crate::representations::{
    affine_expr::AffineExpr,
    instruction::Instruction,
    property::{InstProperty, Property},
};

use super::passes::{InstAnalysis, PassInfo, PassRun, Workspace};

pub struct MemAccessProp {
    pub accessed_dims: Vec<String>,
}

impl Property for MemAccessProp {
    fn name(&self) -> &str {
        "MemAccess"
    }

    fn description(&self) -> &str {
        "Determining the dimensions of memeory accesses"
    }
}

impl InstProperty for MemAccessProp {
    fn inline_to_str(&self, _: &Instruction) -> String {
        format!("Used dimensions: {:?}", self.accessed_dims)
    }
}

impl fmt::Display for MemAccessProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.accessed_dims.join(", "))
    }
}
#[derive(InstPass)]
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

impl InstAnalysis for MemAccessAnalysis {
    fn analyze_inst(&self, inst: &Instruction) -> Vec<Box<dyn InstProperty>> {
        match inst {
            Instruction::DataLoad(mem_access) => {
                let accessed_dims = mem_access
                    .addr
                    .iter()
                    .flat_map(|expr| Self::get_accesses_from_affine_expr(expr))
                    .collect();
                vec![Box::new(MemAccessProp { accessed_dims })]
            }
            Instruction::DataStore(mem_access) => {
                let accessed_dims = mem_access
                    .addr
                    .iter()
                    .flat_map(|expr| Self::get_accesses_from_affine_expr(expr))
                    .collect();
                vec![Box::new(MemAccessProp { accessed_dims })]
            }
            _ => {
                vec![]
            }
        }
        // let mut mem_access = Vec::new();
        // for port in inst.pe.arch.data_ports.iter() {
        //     match port {
        //         DataPort::MemoryReadPort(mem_port) => {
        //             mem_access.push(mem_port.mem_name.clone());
        //         }
        //         DataPort::MemoryWritePort(mem_port) => {
        //             mem_access.push(mem_port.mem_name.clone());
        //         }
        //         _ => {}
        //     }
        // }
        // vec![Box::new(MemAccessProp { mem_access })]
    }
}

impl PassInfo for MemAccessAnalysis {
    fn name(&self) -> &str {
        "Memory Access Analysis"
    }

    fn description(&self) -> &str {
        "Memory Access Analysis"
    }

    fn required_properties(&self) -> Vec<String> {
        vec!["Architecture".to_string()]
    }

    fn produced_properties(&self) -> Vec<String> {
        vec!["MemAccess".to_string()]
    }
}
