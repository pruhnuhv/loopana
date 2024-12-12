use loopana::passes::transform_pass::Transforming;
use loopana::representations::loops::LoopNest;
use loopana::representations::transforms::Transforms;
use std::{fs, path::Path};
#[test]
fn test_transforms() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let file_path = Path::new(manifest).join("example/transforms.trf");
    let input_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
    let transforms: Transforms =
        Transforms::from_str(&input_str).expect("Failed to deserialize YAML");
    let problem_file_path = Path::new(manifest).join("example/prob.loop");
    let yaml_str = fs::read_to_string(problem_file_path).expect("Failed to read YAML file");
    let loop_prob: LoopNest = serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
    let transformed_loop_prob = loop_prob.apply_all(&transforms);
    // Serialize the transformed loop prob
    let serialized = serde_yaml::to_string(&transformed_loop_prob).unwrap();
    // Save to file
    let transformed_file_path = Path::new(manifest).join("example/transformed_prob.loop");
    fs::write(transformed_file_path, serialized).expect("Failed to write to file");

    // try to load it again
    let transformed_file_path = Path::new(manifest).join("example/transformed_prob.loop");
    let yaml_str = fs::read_to_string(transformed_file_path).expect("Failed to read YAML file");
    let _loop_prob: LoopNest = serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");
}
