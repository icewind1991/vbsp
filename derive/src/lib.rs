use merge::Merge;
use syn::parse::Parse;
use syn::{parse_macro_input, Attribute, DeriveInput, Result};

mod entity;

#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_wrapper(input, entity::derive_entity)
}

fn derive_wrapper<F: Fn(DeriveInput) -> Result<proc_macro2::TokenStream>>(
    input: proc_macro::TokenStream,
    derive: F,
) -> proc_macro::TokenStream {
    match derive(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn parse_attrs<T: Parse + Default + Merge>(attrs: &[Attribute]) -> Result<T> {
    let mut result = T::default();
    for attr in attrs {
        let parsed = attr.parse_args()?;
        result.merge(parsed);
    }
    Ok(result)
}
