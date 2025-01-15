use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(PropertyHook)]
pub fn derive_property_hook(input: TokenStream) -> TokenStream {
    // Parse the macro input as a DeriveInput syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // The name of the struct/enum we're deriving on
    let name = &input.ident;

    // Generate an impl block that implements `PropertyHook` for `name`
    // using the type's Display (`to_string`) and the type’s name.
    let expanded = quote! {
        impl PropertyHook for #name {
            fn property_hook_id(&self) -> String {
                format!("{}::{}", stringify!(#name), self.to_string())
            }
        }
    };

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}
