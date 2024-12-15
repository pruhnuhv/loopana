use crate::representations::{
    instruction::Instruction,
    property::{InstProperty, Property},
};

pub struct MemAccessProp {
    pub mem_access: Vec<String>,
}

impl Property for MemAccessProp {
    fn name(&self) -> &str {
        "Memory Access"
    }

    fn description(&self) -> &str {
        "Memory Access"
    }
}

impl InstProperty for MemAccessProp {
    fn inline_to_str(&self, _: &Instruction) -> String {
        format!("{:?}", self.mem_access)
    }
}
