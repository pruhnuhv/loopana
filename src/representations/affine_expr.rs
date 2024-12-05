use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric0, char, digit1, multispace0, space0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents an affine expression.
/// It can be a constant, a variable, or an affine combination of variables.
#[derive(Clone, Debug, PartialEq)]
pub enum AffineExpr {
    Var(String),
    Const(i32),
    Add(Box<AffineExpr>, Box<AffineExpr>),
    Sub(Box<AffineExpr>, Box<AffineExpr>),
    Mul(Coeff, Box<AffineExpr>),
    Div(Box<AffineExpr>, Coeff),
    Mod(Box<AffineExpr>, Coeff),
}

/// Represents a coefficient (constant or a variable as metaparameters)
#[derive(Clone, Debug, PartialEq)]
pub enum Coeff {
    Const(i32),
    ConstVar(String),
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

// Helper function to parse identifiers (variables)
fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, alphanumeric0))(input)
}

// Helper function to parse integers
fn parse_integer(input: &str) -> IResult<&str, i32, nom::error::Error<&str>> {
    map_res(
        recognize::<_, _, nom::error::Error<_>, _>(pair(alt((tag("-"), tag("+"))), digit1)),
        str::parse::<i32>,
    )(input)
    .or_else(|_| map_res(digit1, str::parse::<i32>)(input))
}

// Parse constants as AffineExpr::Const
fn parse_const(input: &str) -> IResult<&str, AffineExpr> {
    map(parse_integer, AffineExpr::Const)(input)
}

// Parse variables as AffineExpr::Var
fn parse_var(input: &str) -> IResult<&str, AffineExpr> {
    map(parse_identifier, |s: &str| AffineExpr::Var(s.to_string()))(input)
}

// parse coefficient (constant or variable)
fn parse_coeff(input: &str) -> IResult<&str, Coeff> {
    alt((
        map(parse_identifier, |s: &str| Coeff::ConstVar(s.to_string())),
        map(parse_integer, Coeff::Const),
    ))(input)
}

// Multiplication can only be between a coefficient and an expression
fn parse_mul(input: &str) -> IResult<&str, AffineExpr> {
    let (input, (coeff, expr)) = tuple((
        parse_coeff,
        alt((
            preceded(space0, parse_factor), // handles "3x"
            preceded(space0, preceded(char('*'), preceded(space0, parse_factor))), // handles "3 * x"
        )),
    ))(input)?;
    if coeff == Coeff::Const(1) {
        Ok((input, expr))
    } else {
        Ok((input, AffineExpr::Mul(coeff, Box::new(expr))))
    }
}

// Parse parenthesized expressions
fn parse_parens(input: &str) -> IResult<&str, AffineExpr> {
    delimited(
        delimited(multispace0, tag("("), multispace0),
        parse_expr,
        delimited(multispace0, tag(")"), multispace0),
    )(input)
}

// Parse primary expressions: constants, variables, variables with coefficient, or parenthesized expressions
fn parse_factor(input: &str) -> IResult<&str, AffineExpr> {
    preceded(
        multispace0,
        alt((parse_mul, parse_const, parse_var, parse_parens)),
    )(input)
}

// Parse term (including optional division and modulo)
fn parse_term(input: &str) -> IResult<&str, AffineExpr> {
    let (input, (expr, op_div)) = tuple((
        parse_factor,
        preceded(
            multispace0,
            opt(tuple((
                alt((tag("/"), tag("%"))),
                preceded(multispace0, parse_coeff),
            ))),
        ),
    ))(input)?;

    Ok((
        input,
        match op_div {
            Some((op, divisor)) => match op {
                "/" => AffineExpr::Div(Box::new(expr), divisor),
                "%" => AffineExpr::Mod(Box::new(expr), divisor),
                _ => unreachable!(),
            },
            None => expr,
        },
    ))
}

// Parse addition and subtraction
fn parse_expr(input: &str) -> IResult<&str, AffineExpr> {
    let (input, init) = parse_term(input)?;
    let (input, res) = many0(pair(
        delimited(multispace0, alt((tag("+"), tag("-"))), multispace0),
        parse_term,
    ))(input)?;
    let expr = res.into_iter().fold(init, |acc, (op, val)| match op {
        "+" => AffineExpr::Add(Box::new(acc), Box::new(val)),
        "-" => AffineExpr::Sub(Box::new(acc), Box::new(val)),
        _ => unreachable!(),
    });
    Ok((input, expr))
}

impl fmt::Display for AffineExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AffineExpr::Const(c) => write!(f, "{}", c),
            AffineExpr::Var(name) => write!(f, "{}", name),
            AffineExpr::Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs),
            AffineExpr::Sub(lhs, rhs) => write!(f, "{} - {}", lhs, rhs),
            AffineExpr::Mul(coeff, expr) => match **expr {
                AffineExpr::Var(_) => write!(f, "{}{}", coeff, expr),
                _ => write!(f, "{}*({})", coeff, expr),
            },
            AffineExpr::Div(expr, divisor) => write!(f, "{}/{}", expr, divisor),
            AffineExpr::Mod(expr, modulus) => write!(f, "{}%{}", expr, modulus),
        }
    }
}

impl fmt::Display for Coeff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coeff::Const(c) => write!(f, "{}", c),
            Coeff::ConstVar(name) => write!(f, "{}", name),
        }
    }
}

impl Serialize for AffineExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AffineExpr;
    use super::Coeff;

    #[test]
    fn test_serde() {
        // test deserialization
        {
            let yaml_str = "(1*x + 2*y)/ 3 - 3*z%5\n";
            let expr: AffineExpr = serde_yaml::from_str(yaml_str).unwrap();
            let expected_expr = AffineExpr::Sub(
                Box::new(AffineExpr::Div(
                    Box::new(AffineExpr::Add(
                        Box::new(AffineExpr::Var("x".to_string())),
                        Box::new(AffineExpr::Mul(
                            Coeff::Const(2),
                            Box::new(AffineExpr::Var("y".to_string())),
                        )),
                    )),
                    Coeff::Const(3),
                )),
                Box::new(AffineExpr::Mod(
                    Box::new(AffineExpr::Mul(
                        Coeff::Const(3),
                        Box::new(AffineExpr::Var("z".to_string())),
                    )),
                    Coeff::Const(5),
                )),
            );
            assert_eq!(expr, expected_expr);
        }

        // test ser->deser->ser consistency
        {
            let expr = AffineExpr::Sub(
                Box::new(AffineExpr::Add(
                    Box::new(AffineExpr::Var("x".to_string())),
                    Box::new(AffineExpr::Div(
                        Box::new(AffineExpr::Mul(
                            Coeff::Const(2),
                            Box::new(AffineExpr::Var("y".to_string())),
                        )),
                        Coeff::Const(3),
                    )),
                )),
                Box::new(AffineExpr::Mod(
                    Box::new(AffineExpr::Mul(
                        Coeff::Const(3),
                        Box::new(AffineExpr::Var("z".to_string())),
                    )),
                    Coeff::Const(5),
                )),
            );
            let serialized = serde_yaml::to_string(&expr).unwrap();
            let deserialized: AffineExpr = serde_yaml::from_str(&serialized).unwrap();
            assert_eq!(expr, deserialized);
        }
    }
}
