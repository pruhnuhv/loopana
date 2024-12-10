use super::affine_expr::{self, AffineExpr};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric0, alphanumeric1, char, digit1, multispace0, multispace1,
    },
    combinator::{opt, recognize},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    DataLoad(DataAccess),
    DataStore(DataAccess),
    Compute(Compute),
}

type Register = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(Register),
    Imm(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataAccess {
    pub array_name: String,
    pub addr: Vec<AffineExpr>,
    /// target or source register, depending load or store
    pub reg: Register,
    /// optional condition to execute the instruction, String is the condition register
    pub cond_suffix: Option<ConditionSuffix>,
    pub cond: Option<Register>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compute {
    pub op: String,
    pub src: Vec<Operand>,
    pub dst: Register,
    /// optional condition to execute the instruction, String is the condition register
    pub cond_suffix: Option<ConditionSuffix>,
    pub cond: Option<Register>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionSuffix {
    EQ,
    NE,
    LT,
    LE,
    GT,
    GE,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub cond_compute: Compute,
    pub prob: f64,
}

fn parse_var_id(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, alphanumeric0))(input)
}

fn parse_reg_id(input: &str) -> IResult<&str, &str> {
    recognize(pair(char('R'), alphanumeric0))(input)
}

fn parse_immediate(input: &str) -> IResult<&str, &str> {
    recognize(pair(char('$'), digit1))(input)
}

fn parse_src(input: &str) -> IResult<&str, &str> {
    alt((parse_reg_id, parse_immediate))(input)
}

fn parse_indices(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(tag("["), separated_list0(tag("]["), parse_var_id), tag("]"))(input)
}

fn parse_cond_code(input: &str) -> IResult<&str, &str> {
    alt((
        tag("EQ"),
        tag("NE"),
        tag("LT"),
        tag("LE"),
        tag("GT"),
        tag("GE"),
    ))(input)
}
fn parse_condition(input: &str) -> IResult<&str, (ConditionSuffix, Register)> {
    let (input, (cond_code, _, reg)) = delimited(
        char('('),
        tuple((parse_cond_code, multispace1, parse_reg_id)),
        char(')'),
    )(input)?;
    Ok((
        input,
        (ConditionSuffix::from_str(cond_code), Register::from(reg)),
    ))
}

fn parse_data_load(input: &str) -> IResult<&str, Instruction> {
    let (input, (reg, _, array, idxs, cond)) = tuple((
        parse_reg_id,
        preceded(multispace0, tag("<=")),
        preceded(multispace0, parse_var_id),
        parse_indices,
        opt(preceded(multispace1, parse_condition)),
    ))(input)?;

    // Parse the indices into AffineExpr
    let idxs: Vec<AffineExpr> = idxs
        .iter()
        .map(|expr| affine_expr::parse_expr(expr).unwrap().1)
        .collect();
    if let Some((cond_suffix, cond)) = cond {
        Ok((
            input,
            Instruction::DataLoad(DataAccess {
                array_name: array.to_string(),
                addr: idxs,
                reg: Register::from(reg),
                cond_suffix: Some(cond_suffix),
                cond: Some(cond),
            }),
        ))
    } else {
        Ok((
            input,
            Instruction::DataLoad(DataAccess {
                array_name: array.to_string(),
                addr: idxs,
                reg: Register::from(reg),
                cond_suffix: None,
                cond: None,
            }),
        ))
    }
}

fn parse_data_store(input: &str) -> IResult<&str, Instruction> {
    let (input, (reg, _, array, idxs, cond)) = tuple((
        parse_reg_id,
        preceded(multispace0, tag("=>")),
        preceded(multispace0, parse_var_id),
        parse_indices,
        opt(preceded(multispace1, parse_condition)),
    ))(input)?;

    // Parse the indices into AffineExpr
    let idxs: Vec<AffineExpr> = idxs
        .iter()
        .map(|expr| affine_expr::parse_expr(expr).unwrap().1)
        .collect();

    if let Some((cond_suffix, cond)) = cond {
        Ok((
            input,
            Instruction::DataStore(DataAccess {
                array_name: array.to_string(),
                addr: idxs,
                reg: Register::from(reg),
                cond_suffix: Some(cond_suffix),
                cond: Some(cond),
            }),
        ))
    } else {
        Ok((
            input,
            Instruction::DataStore(DataAccess {
                array_name: array.to_string(),
                addr: idxs,
                reg: Register::from(reg),
                cond_suffix: None,
                cond: None,
            }),
        ))
    }
}

fn parse_compute(input: &str) -> IResult<&str, Instruction> {
    let (input, (op, dst, srcs, cond)) = tuple((
        preceded(multispace0, alphanumeric1),
        terminated(preceded(multispace0, parse_reg_id), multispace0),
        separated_list1(tuple((multispace0, char(','), multispace0)), parse_src),
        opt(preceded(multispace0, parse_condition)),
    ))(input)?;

    // check dst operand cannot be immediat
    assert!(dst.chars().next().unwrap() != '$');
    if let Some((cond_suffix, cond)) = cond {
        Ok((
            input,
            Instruction::Compute(Compute {
                op: op.to_string(),
                src: srcs.iter().map(|s| Operand::from_str(s)).collect(),
                dst: dst.to_string(),
                cond_suffix: Some(cond_suffix),
                cond: Some(cond.to_string()),
            }),
        ))
    } else {
        Ok((
            input,
            Instruction::Compute(Compute {
                op: op.to_string(),
                src: srcs.iter().map(|s| Operand::from_str(s)).collect(),
                dst: dst.to_string(),
                cond_suffix: None,
                cond: None,
            }),
        ))
    }
}

fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    terminated(
        alt((parse_data_load, parse_data_store, parse_compute)),
        multispace0,
    )(input)
}

// Function to multiply coefficients TODO:Not used?
// fn parse_multiply_coeff(c1: Coeff, c2: Coeff) -> Coeff {
//     match (c1, c2) {
//         (Coeff::Const(a), Coeff::Const(b)) => Coeff::Const(a * b),
//         (Coeff::Const(1), other) => other,
//         (other, Coeff::Const(1)) => other,
//         (c1, c2) => Coeff::Mul(Box::new(c1), Box::new(c2)),
//     }
// }

// Function to compute the affine expression
// fn compute_affine_expr(indices: &[&str]) -> AffineExpr {
//     let n = indices.len();
//     if n == 0 {
//         return AffineExpr::Const(0);
//     }
//     let mut expr = AffineExpr::Var(indices[n - 1].to_string());
//     let mut coeff = Coeff::Const(1);
//     for k in (0..n - 1).rev() {
//         let max_coeff = Coeff::ConstVar(format!("MAX_{}", indices[k + 1]));
//         coeff = parse_multiply_coeff(coeff, max_coeff);
//         let term = AffineExpr::Mul(
//             coeff.clone(),
//             Box::new(AffineExpr::Var(indices[k].to_string())),
//         );
//         expr = AffineExpr::Add(Box::new(term), Box::new(expr));
//     }
//     expr
// }

// Displays

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::DataLoad(data_access) => {
                write!(f, "{} <= ", data_access.reg)?;
                write!(f, "{}", data_access.array_name)?;
                for idx in &data_access.addr {
                    write!(f, "[{}]", idx)?;
                }
                if data_access.cond.is_some() {
                    write!(
                        f,
                        " ({} {})",
                        data_access.cond_suffix.as_ref().unwrap(),
                        data_access.cond.as_ref().unwrap()
                    )
                } else {
                    Ok(())
                }
            }

            Instruction::DataStore(data_access) => {
                write!(f, "{} => ", data_access.reg)?;
                write!(f, "{}", data_access.array_name)?;
                for idx in &data_access.addr {
                    write!(f, "[{}]", idx)?;
                }
                if data_access.cond.is_some() {
                    write!(
                        f,
                        " ({} {})",
                        data_access.cond_suffix.as_ref().unwrap(),
                        data_access.cond.as_ref().unwrap()
                    )
                } else {
                    Ok(())
                }
            }
            Instruction::Compute(compute) => write!(f, "{}", compute),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Reg(reg) => write!(f, "{}", reg),
            Operand::Imm(imm) => write!(f, "${}", imm),
        }
    }
}

impl fmt::Display for Compute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.op, self.dst)?;
        let mut src_iter = self.src.iter();
        if let Some(first_src) = src_iter.next() {
            write!(f, " {}", first_src)?;
            for src in src_iter {
                write!(f, ", {}", src)?;
            }
        }
        if self.cond.is_some() {
            write!(
                f,
                " ({} {})",
                self.cond_suffix.as_ref().unwrap(),
                self.cond.as_ref().unwrap()
            )
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for ConditionSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionSuffix::EQ => write!(f, "EQ"),
            ConditionSuffix::NE => write!(f, "NE"),
            ConditionSuffix::LT => write!(f, "LT"),
            ConditionSuffix::LE => write!(f, "LE"),
            ConditionSuffix::GT => write!(f, "GT"),
            ConditionSuffix::GE => write!(f, "GE"),
        }
    }
}

// Serializers/Deserializers
impl<'de> Deserialize<'de> for Instruction {
    fn deserialize<D>(deserializer: D) -> Result<Instruction, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input as a string
        let s = String::deserialize(deserializer)?;
        // Parse the string into an Instruction
        parse_instruction(&s)
            .map(|(_, instr)| instr)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}

impl Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// from_str of the simple structs
impl ConditionSuffix {
    fn from_str(s: &str) -> ConditionSuffix {
        match s {
            "EQ" => ConditionSuffix::EQ,
            "NE" => ConditionSuffix::NE,
            "LT" => ConditionSuffix::LT,
            "LE" => ConditionSuffix::LE,
            "GT" => ConditionSuffix::GT,
            "GE" => ConditionSuffix::GE,
            _ => panic!("Invalid condition suffix"),
        }
    }
}

impl Operand {
    fn from_str(s: &str) -> Operand {
        if s.chars().next().unwrap() == '$' {
            Operand::Imm(s[1..].parse().unwrap())
        } else {
            Operand::Reg(s.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_parse_instruction() {
        let instr = "R1 <= A[x][y] (EQ Rcmp)";
        let (_, instr) = parse_instruction(instr).unwrap();
        assert_eq!(
            instr,
            Instruction::DataLoad(DataAccess {
                array_name: "A".to_string(),
                addr: vec![
                    AffineExpr::Var("x".to_string()),
                    AffineExpr::Var("y".to_string())
                ],
                reg: "R1".to_string(),
                cond_suffix: Some(ConditionSuffix::EQ),
                cond: Some("Rcmp".to_string())
            })
        );

        let instr = "R1 => A[x][y]";
        let (_, instr) = parse_instruction(instr).unwrap();
        assert_eq!(
            instr,
            Instruction::DataStore(DataAccess {
                array_name: "A".to_string(),
                addr: vec![
                    AffineExpr::Var("x".to_string()),
                    AffineExpr::Var("y".to_string())
                ],
                reg: "R1".to_string(),
                cond_suffix: None,
                cond: None
            })
        );

        let instr = "cmp Rcmp Ra, $0";
        let (_, instr) = parse_compute(instr).unwrap();
        assert_eq!(
            instr,
            Instruction::Compute(Compute {
                op: "cmp".to_string(),
                src: vec![Operand::Reg("Ra".to_string()), Operand::Imm(0)],
                dst: "Rcmp".to_string(),
                cond_suffix: None,
                cond: None
            })
        );
    }
}
