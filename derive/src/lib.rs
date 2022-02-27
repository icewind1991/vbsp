use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod entity;

#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_wrapper(input, entity::derive_entity)
}

fn derive_wrapper<F: Fn(DeriveInput) -> Result<proc_macro2::TokenStream, String>>(
    input: proc_macro::TokenStream,
    derive: F,
) -> proc_macro::TokenStream {
    match derive(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(e) => quote!(compile_error!(#e)).into(),
    }
}
