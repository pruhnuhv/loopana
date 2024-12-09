use super::mapping::MappingType;
use core::fmt;
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0, multispace1},
    combinator::{map, opt},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

use super::instruction::Instruction;
use serde::{Deserialize, Deserializer, Serialize};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoopNest {
    pub iters: Vec<LoopIter>,
    pub body: Vec<Instruction>,
    pub properties: Option<LoopProperties>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopIter {
    pub iter_name: String,
    pub bounds: (i32, i32),
    pub step: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoopProperties {
    pub mapping: HashMap<String, MappingType>,
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(alpha1, String::from)(input)
}

fn parse_number(input: &str) -> IResult<&str, i32> {
    map(digit1, |s: &str| s.parse().unwrap())(input)
}

fn parse_range(input: &str) -> IResult<&str, (i32, i32)> {
    delimited(
        tag("("),
        tuple((parse_number, preceded(tag(".."), parse_number))),
        tag(")"),
    )(input)
}

fn parse_step(input: &str) -> IResult<&str, i32> {
    preceded(
        tuple((tag("."), tag("step"), tag("("))),
        terminated(parse_number, tag(")")),
    )(input)
}

fn parse_loop_iter(input: &str) -> IResult<&str, LoopIter> {
    let (input, _) = tuple((tag("for"), multispace1))(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = tuple((multispace0, tag("in"), multispace0))(input)?;
    let (input, (start, end)) = parse_range(input)?;
    let (input, step) = opt(preceded(multispace0, parse_step))(input)?;

    Ok((
        input,
        LoopIter {
            iter_name: name,
            bounds: (start, end),
            step: if let Some(step) = step { step } else { 1 },
        },
    ))
}

impl<'de> Deserialize<'de> for LoopIter {
    fn deserialize<D>(deserializer: D) -> Result<LoopIter, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input as a string
        let s = String::deserialize(deserializer)?;
        // Parse the string into an AffineExpr
        parse_loop_iter(&s)
            .map(|(_, expr)| expr)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}

impl Serialize for LoopIter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl fmt::Display for LoopIter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.step != 1 {
            write!(
                f,
                "for {} in ({}..{}).step({})",
                self.iter_name, self.bounds.0, self.bounds.1, self.step
            )
        } else {
            write!(
                f,
                "for {} in ({}..{})",
                self.iter_name, self.bounds.0, self.bounds.1
            )
        }
    }
}

// TODO move this to integration tests
#[cfg(test)]
mod tests {
    use serde_yaml;
    use std::{fs, path::Path};

    use super::*;

    #[test]
    fn test_deserialize() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // Construct the file path to loopprob.yaml
        let file_path = Path::new(manifest_dir).join("example/prob.loop");
        let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
        let loop_prob: LoopNest =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
    }

    #[test]
    fn test_serde() {
        let loop_prob_str = r#"
iters:
  - for m in (0..100).step(1)
  - for k in (0..300)
  - for n in (0..200)
body:
  - Ra <= A[m][k]
  - cmp Rcmp Ra, $0
  - Rb <= B[k][n] (LE Rcmp)
  - Rc <= C[m][n] (LE Rcmp)
  - mac Rc1 Ra, Rb, Rc (LE Rcmp)
  - Rc1 => C[m][n] (LE Rcmp)
    "#;
        let loop_prob: LoopNest = serde_yaml::from_str(loop_prob_str).unwrap();
        let serialized = serde_yaml::to_string(&loop_prob).unwrap().clone();
        println!("{}", serialized);
        let deserialized: LoopNest = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(loop_prob, deserialized);
    }
}
