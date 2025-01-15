use core::fmt;
use std::collections::HashMap;

pub trait Property: fmt::Display {}

pub trait PropertyHook {
    fn property_hook_id(&self) -> String;
}

impl<T: PropertyHook> PropertyHook for &T {
    fn property_hook_id(&self) -> String {
        (*self).property_hook_id()
    }
}

pub struct PropertyManager {
    properties: HashMap<String, Vec<Box<dyn Property>>>,
}

impl PropertyManager {
    pub fn new() -> Self {
        PropertyManager {
            properties: HashMap::new(),
        }
    }

    /// add entries
    pub fn from_entries(entries: Vec<String>) -> Self {
        PropertyManager {
            properties: entries
                .into_iter()
                .map(|entry| (entry, Vec::new()))
                .collect(),
        }
    }

    /// add entry
    pub fn add_entry(&mut self, entry: String) {
        self.properties.entry(entry).or_insert(Vec::new());
    }

    pub fn add_property_to_hook(
        &mut self,
        property_hook: impl PropertyHook,
        property: Box<dyn Property>,
    ) {
        let property_hook_id = property_hook.property_hook_id();
        if !self.properties.contains_key(&property_hook_id) {
            panic!("Property hook {} not found", property_hook_id);
        }
        self.properties
            .entry(property_hook_id)
            .or_insert(Vec::new())
            .push(property);
    }

    pub fn get_properties_by_hook(
        &self,
        property_hook: impl PropertyHook,
    ) -> Option<&Vec<Box<dyn Property>>> {
        let property_hook_id = property_hook.property_hook_id();
        self.properties.get(&property_hook_id)
    }

    pub fn add_property_by_id(&mut self, property_hook_id: String, property: Box<dyn Property>) {
        self.properties
            .entry(property_hook_id)
            .or_insert(Vec::new())
            .push(property);
    }

    pub fn get_properties_by_id(
        &self,
        property_hook_id: String,
    ) -> Option<&Vec<Box<dyn Property>>> {
        self.properties.get(&property_hook_id)
    }
}
