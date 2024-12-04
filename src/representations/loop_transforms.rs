use super::loops::*;
pub trait AffineTransform {
    fn transform(&self, expr: &AffineExpression);
}

impl AffineTransform for AffineExpression {
    fn transform(&self, expr: &AffineExpression) {
        let mut result = self.clone();
        for (var, coeff) in expr.vars {
            *result.vars.entry(var).or_insert(0) += coeff;
        }
        let self_const = self.constant.unwrap_or(0);
        let other_const = expr.constant.unwrap_or(0);
        result.constant = Some(self_const + other_const);
        result
    }
}