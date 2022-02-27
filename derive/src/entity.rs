use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Field, GenericParam, Path, PathArguments, PathSegment,
    Type, TypePath, Variant,
};
use syn_util::{contains_attribute, get_attribute_value};

type Result<T, E = &'static str> = std::result::Result<T, E>;

pub fn derive_entity(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    if input.generics.lifetimes().count() > 1 {
        return Err("Can't derive Entity on structs or entities with more than 1 lifetime");
    }
    let source_lifetime = input
        .generics
        .params
        .iter()
        .find(|param| matches!(param, GenericParam::Lifetime(_)));

    match &input.data {
        Data::Struct(data) => derive_entity_struct(&input, data, source_lifetime),
        Data::Enum(data) => derive_entity_enum(&input, data, source_lifetime),
        _ => Err("Can only derive Entity for structs and enums"),
    }
}

fn derive_entity_enum(
    input: &DeriveInput,
    data: &DataEnum,
    source_lifetime: Option<&GenericParam>,
) -> Result<proc_macro2::TokenStream> {
    let variants = data
        .variants
        .iter()
        .filter(|variant| !contains_attribute(&variant.attrs, &["entity", "default"]))
        .map(EntityVariant::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let default = &data
        .variants
        .iter()
        .find(|variant| contains_attribute(&variant.attrs, &["entity", "default"]))
        .ok_or("Enum must have one variant with `#[entity(default)]` set")?
        .ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    let lifetime_or_default = LifetimeOrAnonymous(source_lifetime);

    Ok(quote! {
        impl #impl_generics TryFrom<crate::RawEntity<#lifetime_or_default>> for #name #ty_generics #where_clause {
            type Error = crate::error::EntityParseError;

            fn try_from(raw: crate::RawEntity<#source_lifetime>) -> Result<Self, Self::Error> {
                let class = raw.prop("classname")?;
                Ok(match class {
                    #(#variants)*
                    _ => Self::#default(raw),
                })
            }
        }
    })
}

fn derive_entity_struct(
    input: &DeriveInput,
    data: &DataStruct,
    source_lifetime: Option<&GenericParam>,
) -> Result<proc_macro2::TokenStream> {
    let fields = data
        .fields
        .iter()
        .map(EntityField::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    let lifetime_or_default = LifetimeOrAnonymous(source_lifetime);

    Ok(quote! {
        impl #impl_generics TryFrom<crate::RawEntity<#lifetime_or_default>> for #name #ty_generics #where_clause {
            type Error = crate::error::EntityParseError;

            fn try_from(raw: crate::RawEntity<#source_lifetime>) -> Result<Self, Self::Error> {
                Ok(#name {
                    #(#fields)*
                })
            }
        }
    })
}

struct LifetimeOrAnonymous<'a>(Option<&'a GenericParam>);

impl ToTokens for LifetimeOrAnonymous<'_> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        match &self.0 {
            Some(params) => params.to_tokens(stream),
            None => stream.append_all(quote! {'_}),
        }
    }
}

struct EntityField<'a> {
    field: &'a Ident,
    name: String,
    default: bool,
}

impl<'a> TryFrom<&'a Field> for EntityField<'a> {
    type Error = &'static str;

    fn try_from(field: &'a Field) -> std::result::Result<Self, Self::Error> {
        let ident = &field
            .ident
            .as_ref()
            .ok_or("Can't derive Entity on structs with unnamed fields")?;
        let name = get_attribute_value(&field.attrs, &["entity", "name"])
            .unwrap_or_else(|| ident.to_string());
        let default = contains_attribute(&field.attrs, &["entity", "default"]);
        Ok(EntityField {
            field: ident,
            name,
            default,
        })
    }
}

impl ToTokens for EntityField<'_> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let EntityField { field, name, .. } = &self;
        let tokens = if self.default {
            quote! {#field: raw.prop_parse(#name).unwrap_or_default(),}
        } else {
            quote! {#field: raw.prop_parse(#name)?,}
        };
        stream.append_all(tokens);
    }
}

struct EntityVariant<'a> {
    name: String,
    variant: &'a Ident,
    ty: &'a Path,
}

impl<'a> TryFrom<&'a Variant> for EntityVariant<'a> {
    type Error = &'static str;

    fn try_from(value: &'a Variant) -> std::result::Result<Self, Self::Error> {
        let name = get_attribute_value(&value.attrs, &["entity", "name"]).ok_or(
            "All variants must have the `#[entity(name)]` or `#[entity(default)]` attribute set",
        )?;
        if value.fields.len() != 1 {
            return Err("All enum variants must have exactly one field");
        }
        let field = value.fields.iter().next().unwrap();
        let path = match &field.ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => return Err("Variants can only contain plain types"),
        };
        Ok(EntityVariant {
            name,
            variant: &value.ident,
            ty: path,
        })
    }
}

impl ToTokens for EntityVariant<'_> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let EntityVariant { name, variant, ty } = &self;

        // strip generic/lifetime params
        let ty = Path {
            leading_colon: ty.leading_colon,
            segments: ty
                .segments
                .iter()
                .map(|segment| PathSegment {
                    ident: segment.ident.clone(),
                    arguments: PathArguments::None,
                })
                .collect(),
        };
        stream.append_all(quote! {#name => Self::#variant(#ty::try_from(raw)?),});
    }
}
