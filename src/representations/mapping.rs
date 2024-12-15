use super::loops::{LoopIter, LoopNest};
use core::fmt;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, multispace0, space0},
    combinator::{cut, map},
    multi::{self, many0, separated_list0},
    sequence::{delimited, terminated, tuple},
    IResult, Parser,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub loop_nest: Option<LoopNest>,
    pub types: HashMap<String, MappingType>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum MappingType {
    Spatial(String),
    TemporalTODO,
    InterTile,
    IntraTile,
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(alpha1, String::from)(input)
}

fn parse_spatial_mapping(input: &str) -> IResult<&str, MappingType> {
    let (input, _) = tag("$")(input)?;
    map(alpha1, |s: &str| MappingType::Spatial(s.to_string()))(input)
}

fn parse_temporal_todo(input: &str) -> IResult<&str, MappingType> {
    let (input, _) = alt((tag("Temporal"), tag("TODO"), tag("?")))(input)?;
    Ok((input, MappingType::TemporalTODO))
}

fn parse_inter_tile(input: &str) -> IResult<&str, MappingType> {
    let (input, _) = tag("InterTile")(input)?;
    Ok((input, MappingType::InterTile))
}

fn parse_intra_tile(input: &str) -> IResult<&str, MappingType> {
    let (input, _) = tag("IntraTile")(input)?;
    Ok((input, MappingType::IntraTile))
}

fn parse_mapping_entry(input: &str) -> IResult<&str, (String, MappingType)> {
    let (input, _) = terminated(tag("-"), space0)(input)?;
    let (input, iter) = parse_identifier(input)?;
    let (input, _) = delimited(space0, tag("->"), space0)(input)?;
    let (input, mapping_type) = cut(alt((
        parse_temporal_todo,
        parse_inter_tile,
        parse_intra_tile,
        parse_spatial_mapping,
    )))(input)?;
    Ok((input, (iter, mapping_type)))
}

fn parse_type_mapping(input: &str) -> IResult<&str, HashMap<String, MappingType>> {
    let (input, entries) = separated_list0(multispace0, parse_mapping_entry)(input)?;
    Ok((input, entries.into_iter().collect()))
}

fn parse_mapping(input: &str) -> IResult<&str, Mapping> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tuple((tag("Mapping"), space0, tag(":")))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, types) = parse_type_mapping(input)?;
    Ok((
        input,
        Mapping {
            loop_nest: None,
            types: types,
        },
    ))
}

impl Mapping {
    pub fn from_str(input: &str) -> Mapping {
        parse_mapping(input).unwrap().1
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mapping:\n")?;
        for (iter, mapping_type) in &self.types {
            write!(f, "\t- {} -> {}\n", iter, mapping_type)?;
        }
        Ok(())
    }
}

impl fmt::Display for MappingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MappingType::Spatial(s) => write!(f, "${}", s),
            MappingType::TemporalTODO => write!(f, "Temporal"),
            MappingType::InterTile => write!(f, "InterTile"),
            MappingType::IntraTile => write!(f, "IntraTile"),
        }
    }
}

impl<'de> Deserialize<'de> for Mapping {
    fn deserialize<D>(deserializer: D) -> Result<Mapping, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Mapping::from_str(&s))
    }
}

impl Serialize for Mapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_serde() {
        let input = r#"
Mapping:
    - m -> InterTile
    - tk -> IntraTile
    - tn -> IntraTile
    - x -> $x
    - y -> $y
    - simd -> $simd
        "#;
        let expected_mapping = Mapping {
            loop_nest: None,
            types: vec![
                ("m".to_string(), MappingType::InterTile),
                ("tk".to_string(), MappingType::IntraTile),
                ("tn".to_string(), MappingType::IntraTile),
                ("x".to_string(), MappingType::Spatial("x".to_string())),
                ("y".to_string(), MappingType::Spatial("y".to_string())),
                ("simd".to_string(), MappingType::Spatial("simd".to_string())),
            ]
            .into_iter()
            .collect(),
        };
        let mapping: Mapping = Mapping::from_str(input);

        //try serialize -> deserialize
        let serialized = serde_yaml::to_string(&expected_mapping).unwrap();
        let deserialized: Mapping = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected_mapping);
    }
}
