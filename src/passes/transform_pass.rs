use crate::representations::affine_expr::AffineExpr;
use crate::representations::affine_expr::Coeff;
use crate::representations::instruction::*;
use crate::representations::loops::*;
use crate::representations::mapping::MappingType;
use crate::representations::transforms::{Transform, Transforms};

pub trait Transforming {
    fn apply(&self, transforms: &Transform) -> Self;
    fn apply_all(&self, transforms: &Transforms) -> Self
    where
        Self: Clone,
    {
        let mut new_self = self.clone();
        for transform in &transforms.transforms {
            new_self = new_self.apply(&transform);
        }
        new_self
    }
}

impl Transforming for Coeff {
    fn apply(&self, transform: &Transform) -> Self {
        match (self, transform) {
            (Coeff::ConstVar(var), Transform::Renaming((old_iter, new_iter))) => {
                if var == old_iter {
                    Coeff::ConstVar(new_iter.clone())
                } else {
                    self.clone()
                }
            }
            (Coeff::ConstVar(_), Transform::Tiling(_)) => {
                // TODO
                self.clone()
            }
            _ => self.clone(),
        }
    }
}

impl Transforming for AffineExpr {
    fn apply(&self, transform: &Transform) -> Self {
        match (self, transform) {
            (AffineExpr::Var(var_name), Transform::Renaming((old_iter, new_iter))) => {
                if var_name == old_iter {
                    AffineExpr::Var(new_iter.clone())
                } else {
                    self.clone()
                }
            }
            (AffineExpr::Var(_), Transform::Tiling(_)) => {
                // TODO
                self.clone()
            }
            (AffineExpr::Var(_), _) => self.clone(),
            (AffineExpr::Const(_), _) => self.clone(),
            (AffineExpr::Add(lhs, rhs), _) => {
                let new_lhs = lhs.apply(transform);
                let new_rhs = rhs.apply(transform);
                AffineExpr::Add(Box::new(new_lhs), Box::new(new_rhs))
            }
            (AffineExpr::Sub(lhs, rhs), _) => {
                let new_lhs = lhs.apply(transform);
                let new_rhs = rhs.apply(transform);
                AffineExpr::Sub(Box::new(new_lhs), Box::new(new_rhs))
            }
            (AffineExpr::Mul(coeff, expr), _) => {
                let new_coeff = coeff.apply(transform);
                let new_expr = expr.apply(transform);
                AffineExpr::Mul(new_coeff, Box::new(new_expr))
            }
            (AffineExpr::Div(expr, divisor), _) => {
                let new_expr = expr.apply(transform);
                let new_divisor = divisor.apply(transform);
                AffineExpr::Div(Box::new(new_expr), new_divisor)
            }
            (AffineExpr::Mod(expr, modulus), _) => {
                let new_expr = expr.apply(transform);
                let new_modulus = modulus.apply(transform);
                AffineExpr::Mod(Box::new(new_expr), new_modulus)
            }
        }
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
        let new_addr = self.addr.iter().map(|idx| idx.apply(transform)).collect();
        DataAccess {
            array_name: self.array_name.clone(),
            addr: new_addr,
            reg: self.reg.clone(),
            cond: self.cond.clone(),
            cond_suffix: self.cond_suffix.clone(),
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
        }
    }
}

impl Transforming for LoopProperties {
    fn apply(&self, transform: &Transform) -> Self {
        match transform {
            Transform::MapSpatial(iter_to_map) => {
                let new_mapping = self
                    .mapping
                    .iter()
                    .map(|(iter_name, mapping_type)| {
                        if iter_name == iter_to_map {
                            (iter_name.clone(), MappingType::Spatial)
                        } else {
                            (iter_name.clone(), mapping_type.clone())
                        }
                    })
                    .collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
            Transform::MapTemporal(iter_to_map) => {
                let new_mapping = self
                    .mapping
                    .iter()
                    .map(|(iter_name, mapping_type)| {
                        if iter_name == iter_to_map {
                            (iter_name.clone(), MappingType::Temporal)
                        } else {
                            (iter_name.clone(), mapping_type.clone())
                        }
                    })
                    .collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
            Transform::Renaming((old_iter, new_iter)) => {
                let new_mapping = self
                    .mapping
                    .iter()
                    .map(|(iter_name, mapping_type)| {
                        if iter_name == old_iter {
                            (new_iter.clone(), mapping_type.clone())
                        } else {
                            (iter_name.clone(), mapping_type.clone())
                        }
                    })
                    .collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
            Transform::Tiling((iter_to_tile, factor)) => {
                // TODO, implement tiling
                let new_mapping = self
                    .mapping
                    .iter()
                    .map(|(iter_name, mapping_type)| {
                        if iter_name == iter_to_tile {
                            (iter_name.clone(), MappingType::Spatial)
                        } else {
                            (iter_name.clone(), mapping_type.clone())
                        }
                    })
                    .collect();
                LoopProperties {
                    mapping: new_mapping,
                }
            }
        }
    }
}

impl Transforming for LoopIter {
    fn apply(&self, transform: &Transform) -> Self {
        match transform {
            Transform::Renaming((old_iter, new_iter)) => {
                if self.iter_name == *old_iter {
                    LoopIter {
                        iter_name: new_iter.clone(),
                        bounds: self.bounds,
                        step: self.step,
                    }
                } else {
                    self.clone()
                }
            }
            Transform::Tiling((iter_to_tile, factor)) => {
                // TODO
                self.clone()
            }
            _ => self.clone(),
        }
    }
}

impl Transforming for LoopNest {
    fn apply(&self, transform: &Transform) -> Self {
        let new_iters = self
            .iters
            .iter()
            .map(|iter| iter.apply(transform))
            .collect();
        let new_body = self
            .body
            .iter()
            .map(|instr| instr.apply(transform))
            .collect();
        let new_properties = match &self.properties {
            Some(properties) => Some(properties.apply(transform)),
            None => None,
        };
        LoopNest {
            iters: new_iters,
            body: new_body,
            properties: new_properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn test_transforms() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let file_path = Path::new(manifest).join("example/transforms.yaml");
        let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
        let transforms: Transforms =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
        let problem_file_path = Path::new(manifest).join("example/prob.yaml");
        let yaml_str = fs::read_to_string(problem_file_path).expect("Failed to read YAML file");
        let loop_prob: LoopNest =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
        let transformed_loop_prob = loop_prob.apply_all(&transforms);
        // Serialize the transformed loop prob
        let serialized = serde_yaml::to_string(&transformed_loop_prob).unwrap();
        // Save to file
        let transformed_file_path = Path::new(manifest).join("example/transformed_prob.yaml");
        fs::write(transformed_file_path, serialized).expect("Failed to write to file");
    }
}
