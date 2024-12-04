use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{char, digit1, space0},
    combinator::{map_res, opt},
    sequence::terminated,
    IResult,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::str::FromStr;

/// Represents an affine expression.
/// It can be a constant, a variable, or an affine combination of variables.
#[derive(Clone, Debug)]
pub enum AffineExpr {
    Const(i32),
    Var(String),
    Add(Box<AffineExpr>, Box<AffineExpr>),
    Sub(Box<AffineExpr>, Box<AffineExpr>),
    Mul(i32, Box<AffineExpr>),
    Div(Box<AffineExpr>, i32),
    Mod(Box<AffineExpr>, i32),
}

impl PartialEq for AffineExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AffineExpr::Const(a), AffineExpr::Const(b)) => a == b,
            (AffineExpr::Var(a), AffineExpr::Var(b)) => a == b,
            (AffineExpr::Add(a1, a2), AffineExpr::Add(b1, b2))
            | (AffineExpr::Sub(a1, a2), AffineExpr::Sub(b1, b2)) => a1 == b1 && a2 == b2,
            (AffineExpr::Mul(c1, e1), AffineExpr::Mul(c2, e2)) => c1 == c2 && e1 == e2,
            (AffineExpr::Div(e1, c1), AffineExpr::Div(e2, c2)) => e1 == e2 && c1 == c2,
            (AffineExpr::Mod(e1, c1), AffineExpr::Mod(e2, c2)) => e1 == e2 && c1 == c2,
            _ => false,
        }
    }
}

impl Eq for AffineExpr {}

impl AffineExpr {
    /// Evaluates the affine expression given a mapping of variable values.
    fn evaluate(&self, vars: &HashMap<String, i32>) -> i32 {
        match self {
            AffineExpr::Const(c) => *c,
            AffineExpr::Var(name) => *vars.get(name).expect("Variable not found"),
            AffineExpr::Add(lhs, rhs) => lhs.evaluate(vars) + rhs.evaluate(vars),
            AffineExpr::Sub(lhs, rhs) => lhs.evaluate(vars) - rhs.evaluate(vars),
            AffineExpr::Mul(coeff, expr) => coeff * expr.evaluate(vars),
            AffineExpr::Div(expr, divisor) => expr.evaluate(vars) / divisor,
            AffineExpr::Mod(expr, modulus) => expr.evaluate(vars) % modulus,
        }
    }
}

impl<'de> Deserialize<'de> for AffineExpr {
    fn deserialize<D>(deserializer: D) -> Result<AffineExpr, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input as a string
        let s = String::deserialize(deserializer)?;
        // Parse the string into an AffineExpr
        parse_expr(&s)
            .map(|(_, expr)| expr)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}

fn parse_expr(input: &str) -> IResult<&str, AffineExpr> {
    let (input, initial_term) = parse_term(input)?;
    parse_expr_tail(input, initial_term)
}

fn parse_expr_tail(input: &str, left: AffineExpr) -> IResult<&str, AffineExpr> {
    let (input, _) = space0(input)?;
    if let Ok((input, (op, term))) = parse_add_sub(input) {
        let expr = match op {
            '+' => AffineExpr::Add(Box::new(left), Box::new(term)),
            '-' => AffineExpr::Sub(Box::new(left), Box::new(term)),
            _ => unreachable!(),
        };
        parse_expr_tail(input, expr)
    } else {
        Ok((input, left))
    }
}

fn parse_add_sub(input: &str) -> IResult<&str, (char, AffineExpr)> {
    let (input, op) = alt((char('+'), char('-')))(input)?;
    let (input, _) = space0(input)?;
    let (input, term) = parse_term(input)?;
    Ok((input, (op, term)))
}

fn parse_term(input: &str) -> IResult<&str, AffineExpr> {
    let (input, _) = space0(input)?;
    alt((parse_full_term, parse_const))(input)
}

fn parse_full_term(input: &str) -> IResult<&str, AffineExpr> {
    let (input, coeff_opt) = opt(terminated(map_res(digit1, i32::from_str), space0))(input)?;
    let coeff = coeff_opt.unwrap_or(1);

    let (input, var_name) = take_while1(|c: char| c.is_alphabetic())(input)?;
    let var_expr = if coeff == 1 {
        AffineExpr::Var(var_name.to_string())
    } else {
        AffineExpr::Mul(coeff, Box::new(AffineExpr::Var(var_name.to_string())))
    };

    let (input, _) = space0(input)?;
    let (input, div_mod_opt) = opt(parse_div_mod)(input)?;
    if let Some((op_char, constant)) = div_mod_opt {
        let expr = match op_char {
            '/' => AffineExpr::Div(Box::new(var_expr), constant),
            '%' => AffineExpr::Mod(Box::new(var_expr), constant),
            _ => unreachable!(),
        };
        Ok((input, expr))
    } else {
        Ok((input, var_expr))
    }
}

fn parse_div_mod(input: &str) -> IResult<&str, (char, i32)> {
    let (input, op_char) = alt((char('/'), char('%')))(input)?;
    let (input, _) = space0(input)?;
    let (input, constant) = map_res(digit1, i32::from_str)(input)?;
    Ok((input, (op_char, constant)))
}

fn parse_const(input: &str) -> IResult<&str, AffineExpr> {
    let (input, number) = map_res(digit1, i32::from_str)(input)?;
    Ok((input, AffineExpr::Const(number)))
}

#[cfg(test)]
mod tests {
    use super::AffineExpr;

    #[test]
    fn test_deserialize() {
        // Read the YAML file (assuming it's in the same directory)
        let yaml_str = "1x + 2y/3 - 3z%5";
        let expr: AffineExpr = serde_yaml::from_str(yaml_str).unwrap();
        let expected_expr = AffineExpr::Sub(
            Box::new(AffineExpr::Add(
                Box::new(AffineExpr::Var("x".to_string())),
                Box::new(AffineExpr::Div(
                    Box::new(AffineExpr::Mul(
                        2,
                        Box::new(AffineExpr::Var("y".to_string())),
                    )),
                    3,
                )),
            )),
            Box::new(AffineExpr::Mod(
                Box::new(AffineExpr::Mul(
                    3,
                    Box::new(AffineExpr::Var("z".to_string())),
                )),
                5,
            )),
        );

        assert_eq!(expr, expected_expr);
    }
}
