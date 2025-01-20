use std::{fs, path::Path};

use loopana::{
    passes::{
        mem_access_analysis::MemAccessAnalysis, pass_pipeline::PassPipeline, workspace::Workspace, free_dim_analysis::FreeDimAnalysis,
    },
    representations::loops::LoopNest,
};

#[test]
fn test_passes() {
    env_logger::init();
    // load the loop nest
    let manifest = env!("CARGO_MANIFEST_DIR");
    let file_path = Path::new(manifest).join("example/transformed_prob.loop");
    let yaml_str = fs::read_to_string(file_path).expect("Failed to read YAML file");
    let loop_nest: LoopNest = serde_yaml::from_str(&yaml_str).expect("Failed to deserialize YAML");

    let mut workspace = Workspace::new(loop_nest, None);
    let mut pass_pipeline = PassPipeline::new();
    let mem_access_pass = MemAccessAnalysis;
    let free_dim_pass = FreeDimAnalysis;
    pass_pipeline.register_pass(Box::new(mem_access_pass));
    pass_pipeline.register_pass(Box::new(free_dim_pass));
    pass_pipeline.run(&mut workspace).unwrap();

    let output_file_path = Path::new(manifest).join("example/transformed_prob.ana");
    let output_str = format!("{}", workspace);
    fs::write(output_file_path, output_str).expect("Failed to write to output file");
    return;
}
