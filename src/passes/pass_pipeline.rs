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

    pub fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
        for pass in self.passes.iter() {
            info!("Running pass: {}", pass.name());
            pass.run(workspace)?;
        }
        Ok(())
    }
}
