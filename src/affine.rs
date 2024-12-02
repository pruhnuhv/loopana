use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone)]
struct AffineExpression {
    // Map variable names to their coefficients
    vars: HashMap<String, i32>,
    // The constant term
    constant: i32,
}

impl AffineExpression {
    // Create a new affine expression
    fn new() -> Self {
        AffineExpression {
            vars: HashMap::new(),
            constant: 0,
        }
    }

    // Add a variable with its coefficient
    fn add_var(mut self, var: &str, coeff: i32) -> Self {
        *self.vars.entry(var.to_string()).or_insert(0) += coeff;
        self
    }

    // Set the constant term
    fn set_constant(mut self, constant: i32) -> Self {
        self.constant = constant;
        self
    }

    // Evaluate the expression given a mapping from variables to values
    fn evaluate(&self, values: &HashMap<String, i32>) -> i32 {
        let mut result = self.constant;
        for (var, coeff) in &self.vars {
            if let Some(value) = values.get(var) {
                result += coeff * value;
            } else {
                panic!("Value for variable '{}' not provided", var);
            }
        }
        result
    }
}

impl Display for AffineExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut terms = Vec::new();
        for (var, coeff) in &self.vars {
            terms.push(format!("{}*{}", coeff, var));
        }
        if self.constant != 0 {
            terms.push(self.constant.to_string());
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
        result.constant += other.constant;
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
        result.constant -= other.constant;
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
        result.constant *= scalar;
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

        let result = expr3.evaluate(&values);

        println!("The result of the expression is: {}", result);
    }
}
