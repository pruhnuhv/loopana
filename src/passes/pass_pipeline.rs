use log::info;

use super::passes::*;

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
            for required_property in pass.required_properties() {
                if !workspace.has_property(required_property) {
                    return Err(format!(
                        "Required property {} not found",
                        required_property.clone()
                    ));
                }
            }

            info!("Running pass: {}", pass.name());
            pass.run(workspace).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
