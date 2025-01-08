extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// You have to implement the method `analyze_inst` for the struct that derives from `InstPass`
#[proc_macro_derive(InstPass)]
pub fn inst_pass_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl PassRun for #name {
            fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
                for inst in workspace.loop_nest.body.iter() {
                    let properties = self.analyze_inst(inst);
                    for property in properties {
                        workspace.add_inst_property_for(inst, property)?;
                    }
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IterPass)]
pub fn iter_pass_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl PassRun for #name {
            fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
                for iter in workspace.loop_nest.iters.iter() {
                    let properties = self.analyze_iter(inst);
                    for property in properties {
                        workspace.add_iter_property_for(iter, property)?;
                    }
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(LoopPass)]
pub fn loop_pass_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl PassRun for #name {
            fn run(&self, workspace: &mut Workspace) -> Result<(), &'static str> {
                let properties = self.analyze_loop(workspace.loop_nest);
                for property in properties {
                    workspace.add_loop_property(property);
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
