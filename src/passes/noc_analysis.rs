use crate::representations::{arch::DataPort, loops::LoopNest};

pub struct NoCAnalysis {
    pub loop_nest: LoopNest,
    pub concretized_noc: Vec<DataPort>,
}

impl NoCAnalysis {
    pub fn from_loop_nest(loop_nest: LoopNest) -> NoCAnalysis {
        NoCAnalysis {
            loop_nest,
            concretized_noc: Vec::new(),
        }
    }
}
