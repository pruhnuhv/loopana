use serde_derive::{Deserialize, Serialize};
/// A transform is a way to modify a loop nest. It can be a spatial or temporal mapping, tiling, or renaming.
/// MapSpatial: Maps a loop to a spatial dimension.
/// MapTemporal: Maps a loop to a temporal dimension.
/// Tiling: Tiles a loop with a given factor.
/// Renaming: Renames a loop iterator.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Transform {
    MapSpatial(String),
    MapTemporal(String),
    Tiling((String, String, i32)),
    Renaming((String, String)),
    Reorder((String, String)),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Transforms {
    pub transforms: Vec<Transform>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn test_deserialize() {
        let yaml_str = r#"
transforms:
    - !MapSpatial
        "m"
    - !MapSpatial
        "k"
    - !MapTemporal
        "n"
    - !Renaming
        - "m"
        - "x"
    - !Renaming
        - "k"
        - "SIMD"
    - !Renaming
        - "n"
        - "y"
        "#;
        let transforms: Transforms =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
        let expected_transforms = Transforms {
            transforms: vec![
                Transform::MapSpatial("m".to_string()),
                Transform::MapSpatial("k".to_string()),
                Transform::MapTemporal("n".to_string()),
                Transform::Renaming(("m".to_string(), "x".to_string())),
                Transform::Renaming(("k".to_string(), "SIMD".to_string())),
                Transform::Renaming(("n".to_string(), "y".to_string())),
            ],
        };
        assert_eq!(transforms, expected_transforms);
    }

    #[test]
    fn test_serde() {
        let test_str = r#"transforms:
    - !MapSpatial
        "m"
    - !MapTemporal
        "n"
    - !Tiling
        - "k"
        - "tiled_k"
        - 2
    - !Renaming
        - "m"
        - "m1""#;
        let transforms: Transforms = serde_yaml::from_str(test_str).unwrap();
        let expected_transforms = Transforms {
            transforms: vec![
                Transform::MapSpatial("m".to_string()),
                Transform::MapTemporal("n".to_string()),
                Transform::Tiling(("k".to_string(), "tiled_k".to_string(), 2)),
                Transform::Renaming(("m".to_string(), "m1".to_string())),
            ],
        };
        assert_eq!(transforms, expected_transforms);
    }
}
