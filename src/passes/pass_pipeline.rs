use log::{debug, info};

use super::{passes::*, workspace::Workspace};

pub struct PassPipeline {
    passes: Vec<Box<dyn Pass>>,
}

impl PassPipeline {
    pub fn new() -> Self {
        PassPipeline { passes: Vec::new() }
    }

    pub fn register_pass(&mut self, pass: Box<dyn Pass>) {
        self.passes.push(pass);
    }

    pub fn run(&self, workspace: &mut Workspace) -> Result<(), String> {
        for pass in self.passes.iter() {
            // checking if the required properties are present
            for required_feature in pass.required_features() {
                if !workspace.feature_available_str(&required_feature) {
                    return Err(format!(
                        "Required property {} for pass {} not found",
                        required_feature.clone(),
                        pass.name()
                    ));
                }
            }

            info!("Running pass: {}", pass.name());
            debug!("Required features: {:?}", pass.required_features());
            pass.run(workspace).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
