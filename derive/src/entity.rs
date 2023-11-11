use crate::parse_attrs;
use merge::Merge;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use structmeta::StructMeta;
use syn::spanned::Spanned;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Field, GenericParam, Ident, LitStr, Path,
    PathArguments, PathSegment, Result, Type, TypePath, Variant,
};

pub fn derive_entity(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    if input.generics.lifetimes().count() > 1 {
        return Err(Error::new(
            input.generics.span(),
            "Can't derive Entity on structs or entities with more than 1 lifetime",
        ));
    }
    let source_lifetime = input
        .generics
        .params
        .iter()
        .find(|param| matches!(param, GenericParam::Lifetime(_)));

    #[cfg(feature = "__vbsp_as_self")]
    let crate_name = "crate";
    #[cfg(not(feature = "__vbsp_as_self"))]
    let crate_name = "vbsp";
    let crate_ident = Ident::new(crate_name, input.span());

    match &input.data {
        Data::Struct(data) => derive_entity_struct(&input, data, source_lifetime, crate_ident),
        Data::Enum(data) => derive_entity_enum(&input, data, source_lifetime, crate_ident),
        _ => Err(Error::new(
            input.span(),
            "Can only derive Entity for structs and enums",
        )),
    }
}

fn derive_entity_enum(
    input: &DeriveInput,
    data: &DataEnum,
    source_lifetime: Option<&GenericParam>,
    crate_ident: Ident,
) -> Result<proc_macro2::TokenStream> {
    let variants = data
        .variants
        .iter()
        .map(EntityVariant::try_from)
        .collect::<Result<Vec<EntityVariant>>>()?;

    if variants
        .iter()
        .filter(|variant| variant.is_default())
        .count()
        != 1
    {
        return Err(Error::new(
            data.variants.span(),
            "Enums must have exactly one variant with `#[entity(default)]` set",
        ));
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    let lifetime_or_default = LifetimeOrAnonymous(source_lifetime);

    let span = input.span();

    Ok(quote_spanned! {span =>
        impl #impl_generics TryFrom<#crate_ident::RawEntity<#lifetime_or_default>> for #name #ty_generics #where_clause {
            type Error = #crate_ident::error::EntityParseError;

            fn try_from(raw: #crate_ident::RawEntity<#source_lifetime>) -> Result<Self, Self::Error> {
                let class = raw.prop("classname")?;
                Ok(match class {
                    #(#variants)*
                })
            }
        }
    })
}

fn derive_entity_struct(
    input: &DeriveInput,
    data: &DataStruct,
    source_lifetime: Option<&GenericParam>,
    crate_ident: Ident,
) -> Result<proc_macro2::TokenStream> {
    let fields = data
        .fields
        .iter()
        .map(EntityField::try_from)
        .collect::<Result<Vec<_>>>()?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    let lifetime_or_default = LifetimeOrAnonymous(source_lifetime);

    Ok(quote! {
        impl #impl_generics TryFrom<#crate_ident::RawEntity<#lifetime_or_default>> for #name #ty_generics #where_clause {
            type Error = #crate_ident::error::EntityParseError;

            fn try_from(raw: #crate_ident::RawEntity<#source_lifetime>) -> Result<Self, Self::Error> {
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

#[derive(Default, StructMeta, Merge)]
struct EntityFieldAttrs {
    name: Option<LitStr>,
    #[merge(strategy = merge::bool::overwrite_false)]
    default: bool,
}

impl EntityFieldAttrs {
    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(LitStr::value)
    }
}

struct EntityField<'a> {
    field: &'a Ident,
    name: String,
    default: bool,
}

impl<'a> TryFrom<&'a Field> for EntityField<'a> {
    type Error = syn::Error;

    fn try_from(field: &'a Field) -> Result<Self> {
        let attrs: EntityFieldAttrs = parse_attrs(&field.attrs)?;
        let ident = &field.ident.as_ref().ok_or(Error::new(
            field.span(),
            "Can't derive Entity on structs with unnamed fields",
        ))?;
        let name = attrs.name().unwrap_or_else(|| ident.to_string());
        Ok(EntityField {
            field: ident,
            name,
            default: attrs.default,
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
    name: NameOrDefault,
    variant: &'a Ident,
    ty: &'a Path,
}

impl EntityVariant<'_> {
    fn is_default(&self) -> bool {
        matches!(self.name, NameOrDefault::Default)
    }
}

enum NameOrDefault {
    Name(String),
    Default,
}

impl<'a> TryFrom<&'a Variant> for EntityVariant<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a Variant) -> Result<Self> {
        let attrs: EntityFieldAttrs = parse_attrs(&value.attrs)?;
        let name = match attrs.default {
            true => NameOrDefault::Default,
            false => NameOrDefault::Name(attrs.name().ok_or_else(||Error::new(value.span(), "All variants must have the `#[entity(name)]` or `#[entity(default)]` attribute set"),) ?)
        };
        if value.fields.len() != 1 {
            return Err(Error::new(
                value.span(),
                "All enum variants must have exactly one field",
            ));
        }
        let field = value.fields.iter().next().unwrap();
        let path = match &field.ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => {
                return Err(Error::new(
                    field.span(),
                    "Variants can only contain plain types",
                ))
            }
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
        match name {
            NameOrDefault::Default => {
                stream.append_all(quote! {_ => Self::#variant(#ty::from(raw)),})
            }
            NameOrDefault::Name(name) => {
                stream.append_all(quote! {#name => Self::#variant(#ty::try_from(raw)?),})
            }
        }
    }
}
