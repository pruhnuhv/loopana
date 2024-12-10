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
use std::{clone, fmt, ops::Mul};

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
    Mul(Box<Coeff>, Box<Coeff>),
}

impl Coeff {
    /// make the expression canonical by always putting the constant on the left
    pub fn normalize(&self) -> Coeff {
        let mut e = self.simplify();
        match e {
            Coeff::Const(_) => self.clone(),
            Coeff::ConstVar(_) => self.clone(),
            Coeff::Mul(e1, e2) => {
                let e1 = e1.normalize();
                let e2 = e2.normalize();
                match (e1.clone(), e2.clone()) {
                    (Coeff::Const(_), Coeff::Const(_)) => Coeff::Mul(Box::new(e2), Box::new(e1)),
                    (Coeff::Const(_), _) => Coeff::Mul(Box::new(e2), Box::new(e1)),
                    (_, Coeff::Const(_)) => Coeff::Mul(Box::new(e2), Box::new(e1)),
                    (_, _) => Coeff::Mul(Box::new(e1), Box::new(e2)),
                }
            }
        }
    }

    /// Simplify the const expression in the AST,
    /// e.g., 0 * x = 0, 1 * x = x, x * 1 = x, x * 0 = 0
    /// and 3 * (3 * x) = 9 * x, etc.
    fn simplify(&self) -> Coeff {
        match self {
            Coeff::Const(_) => self.clone(),
            Coeff::ConstVar(_) => self.clone(),
            Coeff::Mul(e1, e2) => {
                let e1 = e1.simplify();
                let e2 = e2.simplify();
                match (e1.clone(), e2.clone()) {
                    (Coeff::Const(0), _) => Coeff::Const(0),
                    (_, Coeff::Const(0)) => Coeff::Const(0),
                    (Coeff::Const(1), e) => e,
                    (e, Coeff::Const(1)) => e,
                    (Coeff::Const(c1), Coeff::Const(c2)) => Coeff::Const(c1 * c2),
                    (Coeff::Const(c1), Coeff::Mul(e1, e2)) => match (*e1, *e2) {
                        (Coeff::Const(c2), e2) => {
                            Coeff::Mul(Box::new(Coeff::Const(c1 * c2)), Box::new(e2))
                        }
                        (e1, Coeff::Const(c2)) => {
                            Coeff::Mul(Box::new(Coeff::Const(c1 * c2)), Box::new(e1))
                        }
                        (e1, e2) => Coeff::Mul(
                            Box::new(Coeff::Const(c1)),
                            Box::new(Coeff::Mul(Box::new(e1), Box::new(e2))),
                        ),
                    },
                    (Coeff::Mul(e1, e2), Coeff::Const(c1)) => match (*e1, *e2) {
                        (Coeff::Const(c2), e2) => {
                            Coeff::Mul(Box::new(Coeff::Const(c1 * c2)), Box::new(e2))
                        }
                        (e1, Coeff::Const(c2)) => {
                            Coeff::Mul(Box::new(Coeff::Const(c1 * c2)), Box::new(e1))
                        }
                        (e1, e2) => Coeff::Mul(
                            Box::new(Coeff::Const(c1)),
                            Box::new(Coeff::Mul(Box::new(e1), Box::new(e2))),
                        ),
                    },
                    (Coeff::Mul(_, _), Coeff::Mul(_, _))
                    | (Coeff::ConstVar(_), _)
                    | (_, Coeff::ConstVar(_)) => Coeff::Mul(Box::new(e1), Box::new(e2)),
                }
            }
        }
    }
}

impl AffineExpr {
    /// Simplify the expression by grouping constatants:
    /// 1 + x + 2 = 3 + x; x + 1 + 2 = x + 3; 1 + x + 2 + y = 3 + x + y
    fn simplify(&self) -> AffineExpr {
        match self {
            AffineExpr::Const(_) => self.clone(),
            AffineExpr::Var(_) => self.clone(),
            AffineExpr::Add(e1, e2) => {
                let e1 = e1.simplify();
                let e2 = e2.simplify();
                match (e1.clone(), e2.clone()) {
                    // Const + Const = Const
                    (AffineExpr::Const(c1), AffineExpr::Const(c2)) => AffineExpr::Const(c1 + c2),
                    // 0 + Const = Const
                    (AffineExpr::Const(0), e) => e,
                    // Const + 0 = Const
                    (e, AffineExpr::Const(0)) => e,
                    // Add(Const, Add(Const, e)) = Add(Const, e)
                    // Add(Const, Add(e, Const)) = Add(Const, e)
                    (AffineExpr::Const(c1), AffineExpr::Add(e1, e2)) => {
                        match (*e1.clone(), *e2.clone()) {
                            (AffineExpr::Const(c2), e2) => {
                                AffineExpr::Add(Box::new(AffineExpr::Const(c1 + c2)), Box::new(e2))
                            }
                            (e1, AffineExpr::Const(c2)) => {
                                AffineExpr::Add(Box::new(AffineExpr::Const(c1 + c2)), Box::new(e1))
                            }
                            (e1, e2) => AffineExpr::Add(
                                Box::new(AffineExpr::Const(c1)),
                                Box::new(AffineExpr::Add(Box::new(e1), Box::new(e2))),
                            ),
                        }
                    }
                    // Add(Add(Const, e), Const) = Add(Const, e)
                    // Add(Add(e, Const), Const) = Add(Const, e)
                    (AffineExpr::Add(e1, e2), AffineExpr::Const(c1)) => {
                        match (*e1.clone(), *e2.clone()) {
                            (AffineExpr::Const(c2), e2) => {
                                AffineExpr::Add(Box::new(AffineExpr::Const(c1 + c2)), Box::new(e2))
                            }
                            (e1, AffineExpr::Const(c2)) => {
                                AffineExpr::Add(Box::new(AffineExpr::Const(c1 + c2)), Box::new(e1))
                            }
                            (e1, e2) => AffineExpr::Add(
                                Box::new(AffineExpr::Const(c1)),
                                Box::new(AffineExpr::Add(Box::new(e1), Box::new(e2))),
                            ),
                        }
                    }

                    // Add(c1, Sub(e, c2)) = Add(c1 - c2, e)
                    // Add(c1, Sub(c2, e)) = Sub(c1 + c2, e)
                    (AffineExpr::Const(c1), AffineExpr::Sub(e1, e2)) => {
                        match (*e1.clone(), *e2.clone()) {
                            (AffineExpr::Const(c2), e2) => {
                                AffineExpr::Add(Box::new(AffineExpr::Const(c1 - c2)), Box::new(e2))
                            }
                            (e1, AffineExpr::Const(c2)) => {
                                AffineExpr::Sub(Box::new(AffineExpr::Const(c1 + c2)), Box::new(e1))
                            }
                            (e1, e2) => AffineExpr::Add(
                                Box::new(AffineExpr::Const(c1)),
                                Box::new(AffineExpr::Sub(Box::new(e1), Box::new(e2))),
                            ),
                        }
                    }
                    // Add(Sub(e, c1), c2) = Add(c2 - c1, e)
                    // Add(Sub(c1, e), c2) = Sub(c1 + c2, e)
                    (AffineExpr::Sub(e1, e2), AffineExpr::Const(c)) => {
                        match (*e1.clone(), *e2.clone()) {
                            (AffineExpr::Const(c1), e2) => {
                                AffineExpr::Add(Box::new(AffineExpr::Const(c - c1)), Box::new(e2))
                            }
                            (e1, AffineExpr::Const(c1)) => {
                                AffineExpr::Sub(Box::new(AffineExpr::Const(c1 + c)), Box::new(e1))
                            }
                            (e1, e2) => AffineExpr::Add(
                                Box::new(AffineExpr::Const(c)),
                                Box::new(AffineExpr::Sub(Box::new(e1), Box::new(e2))),
                            ),
                        }
                    }

                    // Add(e, Const) = Add(Const, e)
                    (e, AffineExpr::Const(c)) => {
                        AffineExpr::Add(Box::new(AffineExpr::Const(c)), Box::new(e))
                    }

                    // Default, do nothing
                    (e1, e2) => AffineExpr::Add(Box::new(e1), Box::new(e2)),
                }
            } // End of Add
            AffineExpr::Sub(_, _) => {
                // TODO
                self.clone()
            }
            AffineExpr::Mul(coeff, e) => {
                let coeff = coeff.normalize();
                let e = e.simplify();
                // TODO, the possible optimizations are not done
                AffineExpr::Mul(coeff, Box::new(e))
            }
            AffineExpr::Div(e, coeff) => {
                let e = e.simplify();
                let coeff = coeff.normalize();
                // TODO, the possible optimizations are not done
                AffineExpr::Div(Box::new(e), coeff)
            }
            AffineExpr::Mod(e, coeff) => {
                let e = e.simplify();
                let coeff = coeff.normalize();
                // TODO, the possible optimizations are not done
                AffineExpr::Mod(Box::new(e), coeff)
            }
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

impl Serialize for AffineExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
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

// A coefficient const variable is annoted as an identifier followed by underscore and a normal identifier
fn parse_const_var(input: &str) -> IResult<&str, Coeff> {
    map(
        pair(recognize(pair(alpha1, tag("_"))), parse_identifier),
        |(s1, s2)| Coeff::ConstVar(format!("{}{}", s1, s2)),
    )(input)
}

// parse multiplication expressions for Coeff
fn parse_coeff(input: &str) -> IResult<&str, Coeff> {
    let (input, first) = parse_factor_coeff(input)?;
    let (input, res) = many0(preceded(
        multispace0,
        preceded(char('*'), preceded(multispace0, parse_factor_coeff)),
    ))(input)?;
    let expr = res
        .into_iter()
        .fold(first, |acc, item| Coeff::Mul(Box::new(acc), Box::new(item)));
    Ok((input, expr))
}

// parse individual factors for Coeff (constants, variables, or parenthesized expressions)
fn parse_factor_coeff(input: &str) -> IResult<&str, Coeff> {
    alt((
        parse_const_var,
        map(parse_integer, Coeff::Const),
        delimited(
            preceded(multispace0, char('(')),
            parse_coeff,
            preceded(multispace0, char(')')),
        ),
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
pub fn parse_expr(input: &str) -> IResult<&str, AffineExpr> {
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
                AffineExpr::Var(_) => write!(f, "{} * {}", coeff, expr),
                _ => write!(f, "{} * ({})", coeff, expr),
            },
            AffineExpr::Div(expr, divisor) => write!(f, "{} / {}", expr, divisor),
            AffineExpr::Mod(expr, modulus) => write!(f, "{} % {}", expr, modulus),
        }
    }
}

impl fmt::Display for Coeff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coeff::Const(c) => write!(f, "{}", c),
            Coeff::ConstVar(name) => write!(f, "{}", name),
            Coeff::Mul(lhs, rhs) => write!(f, "{} * {}", lhs, rhs),
        }
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
            let yaml_str = "(1*x + M_a*y)/ 3 - 3*z%5\n";
            let expr: AffineExpr = serde_yaml::from_str(yaml_str).unwrap();
            let expected_expr = AffineExpr::Sub(
                Box::new(AffineExpr::Div(
                    Box::new(AffineExpr::Add(
                        Box::new(AffineExpr::Var("x".to_string())),
                        Box::new(AffineExpr::Mul(
                            Coeff::ConstVar("M_a".to_string()),
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

        // test another case for ser->deser->ser
        {
            let test_str = "(MAX_a * MAX_b) * x + y";
            let expr: AffineExpr = serde_yaml::from_str(test_str).unwrap();
            let serialized = serde_yaml::to_string(&expr).unwrap();
            let deserialized: AffineExpr = serde_yaml::from_str(&serialized).unwrap();
            assert_eq!(expr, deserialized);
        }
    }

    #[test]
    fn test_normalization() {}
}
