use crate::representations::affine_expr::AffineExpr;
use crate::representations::affine_expr::Coeff;
use crate::representations::instruction::*;
use crate::representations::loops::*;
use crate::representations::transforms::{Transform, Transforms};

pub trait Transforming {
    fn apply(&self, transform: &Transform) -> Self;
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
            // Coeff::Const
            (Coeff::Const(_), _) => self.clone(),

            // Coeff::ConstVar
            (Coeff::ConstVar(var), Transform::Tiling((old, _, _))) => {
                if var == old {
                    panic!("You are trying to tile the constant variable {} that is used as a coefficient. This is not allowed.", var);
                } else {
                    self.clone()
                }
            }
            (Coeff::ConstVar(var), Transform::Renaming((old_iter, new_iter))) => {
                if var == old_iter {
                    Coeff::ConstVar(new_iter.clone())
                } else {
                    self.clone()
                }
            }
            (Coeff::ConstVar(_), Transform::Reorder(_)) => self.clone(),

            // Coeff::Mul
            (Coeff::Mul(lhs, rhs), _) => {
                let new_lhs = lhs.apply(transform);
                let new_rhs = rhs.apply(transform);
                Coeff::Mul(Box::new(new_lhs), Box::new(new_rhs))
            }
        }
    }
}

impl Transforming for AffineExpr {
    fn apply(&self, transform: &Transform) -> Self {
        match (self, transform) {
            // AffineExpr::Var
            (AffineExpr::Var(var), Transform::Tiling((old, new, factor))) => {
                if var == old {
                    AffineExpr::Add(
                        Box::new(AffineExpr::Mul(
                            Coeff::Const(*factor),
                            Box::new(AffineExpr::Var(new.clone())),
                        )),
                        Box::new(AffineExpr::Var(var.clone())),
                    )
                } else {
                    self.clone()
                }
            }
            (AffineExpr::Var(var_name), Transform::Renaming((old_iter, new_iter))) => {
                if var_name == old_iter {
                    AffineExpr::Var(new_iter.clone())
                } else {
                    self.clone()
                }
            }
            (AffineExpr::Var(_), Transform::Reorder(_)) => self.clone(),

            // AffineExpr::Const
            (AffineExpr::Const(_), _) => self.clone(),

            // AffineExpr::Add
            (AffineExpr::Add(lhs, rhs), _) => {
                let new_lhs = lhs.apply(transform);
                let new_rhs = rhs.apply(transform);
                AffineExpr::Add(Box::new(new_lhs), Box::new(new_rhs))
            }

            // AffineExpr::Sub
            (AffineExpr::Sub(lhs, rhs), _) => {
                let new_lhs = lhs.apply(transform);
                let new_rhs = rhs.apply(transform);
                AffineExpr::Sub(Box::new(new_lhs), Box::new(new_rhs))
            }

            // AffineExpr::Mul
            (AffineExpr::Mul(coeff, expr), _) => {
                let new_coeff = coeff.apply(transform);
                let new_expr = expr.apply(transform);
                AffineExpr::Mul(new_coeff, Box::new(new_expr))
            }

            // AffineExpr::Div
            (AffineExpr::Div(expr, divisor), _) => {
                let new_expr = expr.apply(transform);
                let new_divisor = divisor.apply(transform);
                AffineExpr::Div(Box::new(new_expr), new_divisor)
            }

            // AffineExpr::Mod
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
        let mut new_addr = Vec::new();
        for idx in &self.addr {
            match (transform, idx) {
                // If tiling and the index is exactly the variable to be tiled, add the new index just after the old one
                // E.g. A[x][y][z] -> A[x][y][y'][z] if tiling y to y' by some factor
                // The factor is only useful for the upper bound of the iterator, so here can ignore it
                (Transform::Tiling((old, new, _)), AffineExpr::Var(var)) => {
                    if var == old {
                        let new_idx = AffineExpr::Var(new.clone());
                        new_addr.push(idx.clone());
                        new_addr.push(new_idx);
                    } else {
                        new_addr.push(idx.apply(transform));
                    }
                }
                _ => new_addr.push(idx.apply(transform)),
            }
        }
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

impl Transforming for LoopIter {
    fn apply(&self, transform: &Transform) -> Self {
        match transform {
            // Tiling transform applied to the iterator it self is only changing the bound
            // The extra loop (with the new iterator) is created by LoopNest
            Transform::Tiling((old, _, factor)) => {
                if self.iter_name == *old {
                    if self.bounds.1 % factor != 0 {
                        panic!(
                        "The upper bound: {} of the iterator {} is not divisible by the factor: {}",
                        self.bounds.1, self.iter_name, factor
                    );
                    }
                    LoopIter {
                        iter_name: old.clone(),
                        bounds: (self.bounds.0, self.bounds.1 / factor),
                        step: self.step,
                    }
                } else {
                    self.clone()
                }
            }
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
            Transform::Reorder(_) => self.clone(),
        }
    }
}

impl Transforming for LoopProperties {
    fn apply(&self, _: &Transform) -> Self {
        // Current Transformations have no effect on LoopProperties
        return self.clone();
        // match transform {
        //     Transform::Renaming((old_iter, new_iter)) => {
        //         let new_mapping = self
        //             .cond_prob
        //             .iter()
        //             .map(|(iter_name, mapping_type)| {
        //                 if iter_name == old_iter {
        //                     (new_iter.clone(), mapping_type.clone())
        //                 } else {
        //                     (iter_name.clone(), mapping_type.clone())
        //                 }
        //             })
        //             .collect();
        //         LoopProperties {
        //             cond_prob: new_mapping,
        //         }
        //     }
        //     Transform::Tiling((old, new, factor)) => {
        //         // The old mapping is kept, add a new entry for the new iterator if the old iterator was in the map
        //         let found = self.cond_prob.get(old);
        //         let mut new_mapping = self.cond_prob.clone();
        //         if found.is_some() {
        //             new_mapping.insert(new.clone(), MappingType::Spatial);
        //         }
        //         LoopProperties {
        //             cond_prob: new_mapping,
        //         }
        //     }
        //     Transform::Reorder((iter1, iter2)) => self.clone(),
        // }
    }
}

impl Transforming for LoopNest {
    fn apply(&self, transform: &Transform) -> Self {
        match transform {
            Transform::Tiling((old, new, factor)) => {
                let mut new_iters: Vec<LoopIter> = self
                    .iters
                    .iter()
                    .map(|iter| iter.apply(transform))
                    .collect();
                let new_body = self
                    .body
                    .iter()
                    .map(|instr| instr.apply(transform))
                    .collect();
                let new_properties = self.properties.apply(transform);
                // Add a new loop with the new iterator
                // The step is the same as the old iterator
                // The upper bound is the factor
                let new_iter = LoopIter {
                    iter_name: new.clone(),
                    bounds: (0, *factor),
                    step: self
                        .iters
                        .iter()
                        .find(|iter| iter.iter_name == *old)
                        .unwrap()
                        .step,
                };
                // insert the new iterator just after the old iterator
                let idx = new_iters.iter().position(|iter| iter.iter_name == *old);
                assert!(
                    idx.is_some(),
                    "The iterator {} to tile was not found in the loop nest, current iterators: {:?}",
                    old,
                    new_iters
                );
                let idx = idx.unwrap();
                new_iters.insert(idx + 1, new_iter);

                LoopNest {
                    iters: new_iters,
                    body: new_body,
                    properties: new_properties,
                }
            }

            Transform::Reorder((iter1, iter2)) => {
                let mut new_iters: Vec<LoopIter> = self
                    .iters
                    .iter()
                    .map(|iter| iter.apply(transform))
                    .collect();
                let new_body = self
                    .body
                    .iter()
                    .map(|instr| instr.apply(transform))
                    .collect();
                let new_properties = self.properties.apply(transform);
                // Reorder the iterators
                let idx1 = new_iters
                    .iter()
                    .position(|iter| iter.iter_name == *iter1)
                    .unwrap_or_else(|| {
                        panic!(
                            "The first iterator {} to reorder was not found in the loop nest",
                            iter1
                        )
                    });
                let idx2 = new_iters
                    .iter()
                    .position(|iter| iter.iter_name == *iter2)
                    .unwrap_or_else(|| {
                        panic!(
                            "The second iterator {} to reorder was not found in the loop nest",
                            iter2
                        )
                    });
                new_iters.swap(idx1, idx2);

                LoopNest {
                    iters: new_iters,
                    body: new_body,
                    properties: new_properties,
                }
            }
            // Renaming
            Transform::Renaming(_) => {
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
                let new_properties = self.properties.apply(transform);
                LoopNest {
                    iters: new_iters,
                    body: new_body,
                    properties: new_properties,
                }
            }
        }
    }
}
