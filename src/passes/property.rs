use core::fmt;

pub trait Property: fmt::Display {}

pub trait PropertyHook {
    fn property_hook_id(&self) -> String;
}

impl<T: PropertyHook> PropertyHook for &T {
    fn property_hook_id(&self) -> String {
        (*self).property_hook_id()
    }
}
