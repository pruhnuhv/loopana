use core::fmt;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{
        char, digit1, line_ending, multispace0, multispace1, not_line_ending, space0,
    },
    combinator::{cut, map, opt},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
/// A transform is a way to modify a loop nest. It can be a spatial or temporal mapping, tiling, or renaming.
/// MapSpatial: Maps a loop to a spatial dimension.
/// MapTemporal: Maps a loop to a temporal dimension.
/// Tiling: Tiles a loop with a given factor.
/// Renaming: Renames a loop iterator.
#[derive(Debug, PartialEq)]
pub enum Transform {
    MapSpatial(String),
    MapTemporal(String),
    Tiling((String, String, i32)),
    Renaming((String, String)),
    Reorder((String, String)),
}

#[derive(Debug, PartialEq)]
pub struct Transforms {
    pub transforms: Vec<Transform>,
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    delimited(
        space0,
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        space0,
    )(input)
}

fn parse_tiling_target(input: &str) -> IResult<&str, (String, String, i32)> {
    let (input, _) = delimited(space0, char('('), space0)(input)?;
    let (input, first) = parse_identifier(input)?;
    let (input, _) = delimited(space0, char(','), space0)(input)?;
    let (input, second) = parse_identifier(input)?;
    let (input, _) = delimited(space0, char(')'), space0)(input)?;
    let (input, _) = tag("by")(input)?;
    let (input, _) = space0(input)?;
    let (input, factor) = digit1(input)?;
    Ok((
        input,
        (
            first.to_string(),
            second.to_string(),
            factor.parse().unwrap(),
        ),
    ))
}

fn parse_tiling(input: &str) -> IResult<&str, Transform> {
    let prefix = delimited(space0, tag("!Tiling"), space0);
    let (input, (_, old_var, _, (old_var1, new_var, factor))) = tuple((
        opt(prefix),
        parse_identifier,
        terminated(tag("->"), space0),
        parse_tiling_target,
    ))(input)?;
    assert_eq!(old_var, old_var1, "When tiling, the old variable should be the same as the first variable in the target, you can use renanming after to change the name of the new variable");
    Ok((
        input,
        Transform::Tiling((old_var.to_string(), new_var.to_string(), factor)),
    ))
}

fn parse_renaming(input: &str) -> IResult<&str, Transform> {
    let prefix = delimited(space0, tag("!Renaming"), space0);
    let (input, (_, old_var, _, new_var)) = tuple((
        opt(prefix),
        preceded(space0, parse_identifier),
        terminated(tag("->"), space0),
        parse_identifier,
    ))(input)?;
    Ok((
        input,
        Transform::Renaming((old_var.to_string(), new_var.to_string())),
    ))
}

fn parse_mapping(input: &str) -> IResult<&str, Transform> {
    let prefix = delimited(
        space0,
        alt((tag("!MapSpatial"), tag("!MapTemporal"))),
        space0,
    );
    let (input, (_, id, _, mapping_type)) = tuple((
        opt(prefix),
        delimited(space0, parse_identifier, space0),
        delimited(space0, tag("=>"), space0),
        alt((tag("Spatial"), tag("Temporal"))),
    ))(input)?;
    match mapping_type {
        "Spatial" => Ok((input, Transform::MapSpatial(id.to_string()))),
        "Temporal" => Ok((input, Transform::MapTemporal(id.to_string()))),
        _ => unreachable!(),
    }
}

fn parse_reorder(input: &str) -> IResult<&str, Transform> {
    let prefix = delimited(space0, tag("!Reorder"), space0);
    let (input, (_, old_var, _, new_var)) = tuple((
        opt(prefix),
        preceded(space0, parse_identifier),
        terminated(tag("<->"), space0),
        parse_identifier,
    ))(input)?;
    Ok((
        input,
        Transform::Reorder((old_var.to_string(), new_var.to_string())),
    ))
}

fn parse_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tuple((multispace0, tag("//"), not_line_ending, opt(line_ending)))(input)?;
    Ok((input, ()))
}

fn ws_and_comments(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((map(parse_comment, |_| ()), map(multispace1, |_| ()))))(input)?;
    Ok((input, ()))
}

fn parse_transform(input: &str) -> IResult<&str, Transform> {
    cut(alt((
        parse_tiling,
        parse_renaming,
        parse_mapping,
        parse_reorder,
    )))(input)
}

fn parse_transforms(input: &str) -> IResult<&str, Transforms> {
    let prefix = tuple((ws_and_comments, tag("-"), multispace0));
    let prefix_1 = tuple((ws_and_comments, tag("-"), multispace0));

    let transform_with_comment =
        tuple((parse_transform, opt(preceded(multispace0, parse_comment))));
    let transform_with_comment_1 =
        tuple((parse_transform, opt(preceded(multispace0, parse_comment))));

    let (input, transforms) = tuple((
        // Parse all transforms except the last one
        many0(terminated(
            preceded(prefix, cut(transform_with_comment)),
            ws_and_comments,
        )),
        // Parse the last transform without requiring trailing whitespace
        opt(preceded(prefix_1, cut(transform_with_comment_1))),
    ))(input)?;

    // Combine all transforms
    let mut all_transforms = transforms.0;
    if let Some(last_transform) = transforms.1 {
        all_transforms.push(last_transform);
    }

    // Extract just the transforms, discarding comment info
    let transforms = all_transforms
        .into_iter()
        .map(|(transform, _comment)| transform)
        .collect();

    Ok((input, Transforms { transforms }))
}

impl Transforms {
    pub fn from_str(input: &str) -> Result<Transforms, String> {
        match parse_transforms(input) {
            Ok((_, transforms)) => Ok(transforms),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

impl Transform {
    pub fn from_str(input: &str) -> Result<Transform, String> {
        match parse_transform(input) {
            Ok((_, transform)) => Ok(transform),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Transform::MapSpatial(id) => write!(f, "!MapSpatial {} => Spatial", id),
            Transform::MapTemporal(id) => write!(f, "!MapTemporal {} => Temporal", id),
            Transform::Tiling((old_var, new_var, factor)) => {
                write!(
                    f,
                    "!Tiling {} -> ({}, {}) by {}",
                    old_var, old_var, new_var, factor
                )
            }
            Transform::Renaming((old_var, new_var)) => {
                write!(f, "!Renaming {} -> {}", old_var, new_var)
            }
            Transform::Reorder((old_var, new_var)) => {
                write!(f, "!Reorder {} <-> {}", old_var, new_var)
            }
        }
    }
}

impl fmt::Display for Transforms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for transform in &self.transforms {
            writeln!(f, " - {}", transform)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let test_str = "n -> (n, simd) by 4";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::Tiling(("n".to_string(), "simd".to_string(), 4));
        assert_eq!(transform, expected_transform);

        let test_str = "!Tiling n -> (n, x) by 8";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::Tiling(("n".to_string(), "x".to_string(), 8));
        assert_eq!(transform, expected_transform);

        let test_str = "m -> ty";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::Renaming(("m".to_string(), "ty".to_string()));
        assert_eq!(transform, expected_transform);

        let test_str = "!MapSpatial x => Spatial";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::MapSpatial("x".to_string());
        assert_eq!(transform, expected_transform);

        let test_str = "y => Spatial";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::MapSpatial("y".to_string());
        assert_eq!(transform, expected_transform);

        let test_str = "!MapSpatial simd => Spatial";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::MapSpatial("simd".to_string());
        assert_eq!(transform, expected_transform);

        let test_str = "!MapTemporal ty => Temporal";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::MapTemporal("ty".to_string());
        assert_eq!(transform, expected_transform);

        let test_str = "!Reorder y <-> tn";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::Reorder(("y".to_string(), "tn".to_string()));
        assert_eq!(transform, expected_transform);

        let test_str = "x <-> y";
        let transform: Transform = Transform::from_str(test_str).unwrap();
        let expected_transform = Transform::Reorder(("x".to_string(), "y".to_string()));
        assert_eq!(transform, expected_transform);

        let test_str = r#"
// tiling n into simd by 4
 - n -> (n, simd) by 4
 - !Tiling n -> (n, x) by 8 // tiling n into x by 8
 - !Tiling m -> (m, y) by 8
 - m -> ty
 - n -> tn
 - !MapSpatial x => Spatial
 - y => Spatial
 - !MapSpatial simd => Spatial
 - !MapTemporal ty => Temporal
 - !MapTemporal tn => Temporal
 - !Reorder y <-> tn
 - x <-> y
        "#;
        let transforms: Transforms = Transforms::from_str(test_str).unwrap();
        let expected_transforms = Transforms {
            transforms: vec![
                Transform::Tiling(("n".to_string(), "simd".to_string(), 4)),
                Transform::Tiling(("n".to_string(), "x".to_string(), 8)),
                Transform::Tiling(("m".to_string(), "y".to_string(), 8)),
                Transform::Renaming(("m".to_string(), "ty".to_string())),
                Transform::Renaming(("n".to_string(), "tn".to_string())),
                Transform::MapSpatial("x".to_string()),
                Transform::MapSpatial("y".to_string()),
                Transform::MapSpatial("simd".to_string()),
                Transform::MapTemporal("ty".to_string()),
                Transform::MapTemporal("tn".to_string()),
                Transform::Reorder(("y".to_string(), "tn".to_string())),
                Transform::Reorder(("x".to_string(), "y".to_string())),
            ],
        };
        assert_eq!(transforms, expected_transforms);
        // serialize -> deserialize
        let serialized = transforms.to_string();
        println!("{}", serialized);
        let deserialized: Transforms = Transforms::from_str(&serialized).unwrap();
        assert_eq!(
            transforms, deserialized,
            "Serialized string: {}",
            serialized
        );
    }
}
