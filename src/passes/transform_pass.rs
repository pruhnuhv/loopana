use crate::representations::affine_expr::AffineExpr;
use crate::representations::instruction::*;
use crate::representations::loops::*;
use crate::representations::mapping::MappingType;
use crate::representations::transforms::{Transform, Transforms};

pub trait Transforming {
    fn apply(&self, transforms: &Transform) -> Self;
    fn apply_all(&self, transforms: &Transforms) -> Self where Self: Clone {
        let mut new_self= self.clone();
        for transform in &transforms.transforms {
            new_self = new_self.apply(&transform);
        }
        new_self
    }
}

impl Transforming for AffineExpr {
    fn apply(&self, transform: &Transform) -> Self {
        // TODO
        self.clone()
    }
}

impl Transforming for Compute {
    fn apply(&self, _transform: &Transform) -> Self {
        // Current Transformations have no effect on Compute instructions
        self.clone()
    }
}

impl Transforming for Conditional {
    fn apply(&self, _transform: &Transform) -> Self {
        // Current Transformations have no effect on Conditional instructions
        self.clone()
    }
}

impl Transforming for DataAccess {
    fn apply(&self, transform: &Transform) -> Self {
        let new_addr = self.addr.apply(transform);
        DataAccess {
            array_name: self.array_name.clone(),
            duration: self.duration,
            addr: new_addr,
            reg: self.reg.clone(),
            cond: self.cond.clone(),
        }
    }
}

impl Transforming for Instruction {
    fn apply(&self, transform: &Transform) -> Self {
        match self {
            Instruction::DataLoad(data_access) => {
                let new_data_access = data_access.apply(transform);
                Instruction::DataLoad(new_data_access)
            }
            Instruction::DataStore(data_access) => {
                let new_data_access = data_access.apply(transform);
                Instruction::DataStore(new_data_access)
            }
            Instruction::Compute(compute) => {
                let new_compute = compute.apply(transform);
                Instruction::Compute(new_compute)
            }
            Instruction::Conditional(conditional) => {
                let new_conditional = conditional.apply(transform);
                Instruction::Conditional(new_conditional)
            }
        }
    }
}

impl Transforming for LoopProperties {
    fn apply(&self, transform: &Transform) -> Self {
        match transform {
            Transform::MapSpatial(iter_to_map) => {
                let new_mapping = self.mapping.iter().map(|(iter_name, mapping_type)| {
                    if iter_name == iter_to_map {
                        (iter_name.clone(), MappingType::Spatial)
                    } else {
                        (iter_name.clone(), mapping_type.clone())
                    }
                }).collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
            Transform::MapTemporal(iter_to_map) => {
                let new_mapping = self.mapping.iter().map(|(iter_name, mapping_type)| {
                    if iter_name == iter_to_map {
                        (iter_name.clone(), MappingType::Temporal)
                    } else {
                        (iter_name.clone(), mapping_type.clone())
                    }
                }).collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
            Transform::Renaming((old_iter, new_iter)) => {
                let new_mapping = self.mapping.iter().map(|(iter_name, mapping_type)| {
                    if iter_name == old_iter {
                        (new_iter.clone(), mapping_type.clone())
                    } else {
                        (iter_name.clone(), mapping_type.clone())
                    }
                }).collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
            Transform::Tiling((iter_to_tile, factor)) => {
                // TODO, implement tiling
                let new_mapping = self.mapping.iter().map(|(iter_name, mapping_type)| {
                    if iter_name == iter_to_tile {
                        (iter_name.clone(), MappingType::Spatial)
                    } else {
                        (iter_name.clone(), mapping_type.clone())
                    }
                }).collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
        }
    }
}
