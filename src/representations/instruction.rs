use super::affine_expr::AffineExpr;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Instruction {
    DataLoad(DataAccess),
    DataStore(DataAccess),
    Compute(Compute),
    Conditional(Conditional),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DataAccess {
    pub array_name: String,
    pub duration: Option<i32>,
    pub addr: AffineExpr,
    /// target or source register, depending load or store
    pub reg: String,
    /// optional condition to execute the instruction, String is the condition register
    pub cond: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Compute {
    pub op: String,
    pub src: Vec<String>,
    pub dst: String,
    pub duration: Option<i32>,
    /// optional condition to execute the instruction, String is the condition register
    pub cond: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Conditional {
    pub cond_compute: Compute,
    pub prob: f64,
}