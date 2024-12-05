use super::affine_expr::AffineExpr;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Instruction {
    DataLoad(DataAccess),
    DataStore(DataAccess),
    Compute(Compute),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DataAccess {
    pub array_name: String,
    pub duration: Option<i32>,
    pub addr: AffineExpr,
    /// target or source register, depending load or store
    pub reg: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Compute {
    pub op: String,
    pub src: Vec<String>,
    pub dst: String,
    pub duration: Option<i32>,
}
