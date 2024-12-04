use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, Mul, Sub};
use std::result;

use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AffineExpression {
    // Map variable names to their coefficients
    vars: HashMap<String, i32>,
    // The constant term
    constant: Option<i32>,
}

impl AffineExpression {
    // Create a new affine expression
    fn new() -> Self {
        AffineExpression {
            vars: HashMap::new(),
            constant: None,
        }
    }

    // Add a variable with its coefficient
    fn add_var(mut self, var: &str, coeff: i32) -> Self {
        *self.vars.entry(var.to_string()).or_insert(0) += coeff;
        self
    }

    // Set the constant term
    fn set_constant(mut self, constant: i32) -> Self {
        self.constant = Some(constant);
        self
    }
}

impl Display for AffineExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut terms = Vec::new();
        for (var, coeff) in &self.vars {
            terms.push(format!("{}*{}", coeff, var));
        }
        if self.constant == None || self.constant.unwrap() != 0 {
            if let Some(constant) = self.constant {
                terms.push(constant.to_string());
            }
        }
        write!(f, "{}", terms.join(" + "))
    }
}

// Implement addition for AffineExpression
impl Add for AffineExpression {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = self.clone();
        for (var, coeff) in other.vars {
            *result.vars.entry(var).or_insert(0) += coeff;
        }
        let self_const = self.constant.unwrap_or(0);
        let other_const = other.constant.unwrap_or(0);
        result.constant = Some(self_const + other_const);
        result
    }
}

// Implement subtraction for AffineExpression
impl Sub for AffineExpression {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut result = self.clone();
        for (var, coeff) in other.vars {
            *result.vars.entry(var).or_insert(0) -= coeff;
        }
        let self_const = self.constant.unwrap_or(0);
        let other_const = other.constant.unwrap_or(0);
        result.constant = Some(self_const - other_const);
        result
    }
}

// Implement scalar multiplication for AffineExpression
impl Mul<i32> for AffineExpression {
    type Output = Self;

    fn mul(self, scalar: i32) -> Self {
        let mut result = self.clone();
        for coeff in result.vars.values_mut() {
            *coeff *= scalar;
        }
        let self_const = result.constant.unwrap_or(0);
        result.constant = Some(self_const * scalar);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affine() {
        // Create an affine expression: 2x + 3y + 5
        let expr1 = AffineExpression::new()
            .add_var("x", 2)
            .add_var("y", 3)
            .set_constant(5);

        // Create another affine expression: -x + 4z - 2
        let expr2 = AffineExpression::new()
            .add_var("x", -1)
            .add_var("z", 4)
            .set_constant(-2);

        // Add the two expressions
        let expr3 = expr1 + expr2;

        // Evaluate the result with x=1, y=2, z=3
        let mut values = HashMap::new();
        values.insert("x".to_string(), 1);
        values.insert("y".to_string(), 2);
        values.insert("z".to_string(), 3);
    }
}
