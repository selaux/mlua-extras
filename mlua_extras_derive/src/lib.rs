#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{proc_macro_error, abort};
use syn::spanned::Spanned;
use venial::{Item, parse_item};

#[proc_macro_error]
#[proc_macro_derive(UserData)]
pub fn derive_user_data(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    let name = match parse_item(input.clone()) {
        Ok(Item::Struct(struct_type)) => {
            struct_type.name.clone()
        },
        Ok(Item::Enum(enum_type)) => {
            enum_type.name.clone()
        },
        Err(err) => abort!(err.span(), "{}", err),
        _ => abort!(input.span(), "only `struct` and `enum` types are supported for TypedUserData")
    };

    quote!(
        impl mlua_extras::mlua::UserData for #name {
            fn add_fields<F: mlua_extras::mlua::UserDataFields<Self>>(fields: &mut F) {
                let mut wrapper = mlua_extras::typed::WrappedBuilder::new(fields);
                <#name as mlua_extras::typed::TypedUserData>::add_fields(&mut wrapper);
            }

            fn add_methods<M: mlua_extras::mlua::UserDataMethods<Self>>(methods: &mut M) {
                let mut wrapper = mlua_extras::typed::WrappedBuilder::new(methods);
                <#name as mlua_extras::typed::TypedUserData>::add_methods(&mut wrapper);
            }
        }
    ).into()
}

#[proc_macro_error]
#[proc_macro_derive(Typed, attributes(typed))]
pub fn derive_typed(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    match parse_item(input.clone()) {
        Ok(Item::Struct(struct_type)) => {
            let name = struct_type.name.clone();
            let label = name.to_string();
            quote!(
                impl mlua_extras::typed::Typed for #name {
                    fn ty() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::class(mlua_extras::typed::TypedClassBuilder::new::<#name>())
                    }

                    fn as_param() -> mlua_extras::typed::Param {
                        mlua_extras::typed::Param {
                            doc: None,
                            name: None,
                            ty: mlua_extras::typed::Type::named(#label),
                        }
                    }
                }
            )
        },
        Ok(Item::Enum(enum_type)) => {
            let name = enum_type.name.clone();
            let label = name.to_string();
            let underscore_name = format!("_{name}");

            let names = enum_type.variants.iter().map(|(variant, _)| variant.name.to_string()).collect::<Vec<_>>();
            let named = enum_type.variants.iter().map(|(variant, _)| format!("{label}{}", variant.name)).collect::<Vec<_>>();
            let variants = enum_type.variants
                .iter()
                .map(|(variant, _punc)| {
                    let name = format!("{label}{}", variant.name);
                    quote!{
                        (
                            #name,
                            mlua_extras::typed::Type::class(
                                mlua_extras::typed::TypedClassBuilder::default()
                                    .derive(#underscore_name)
                            )
                        )
                    }
                })
                .collect::<Vec<_>>();

            let enum_alt = format!("{label}Enum");
            // TODO: This should be a union alias
            quote!(
                impl mlua_extras::typed::Typed for #name {
                    fn ty() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::r#union([
                            #(mlua_extras::typed::Type::named(#named),)*
                        ])
                    }

                    fn implicit() -> impl IntoIterator<Item=(&'static str, mlua_extras::typed::Type)> {
                        [
                            (
                                #enum_alt,
                                mlua_extras::typed::Type::r#enum(vec![#(mlua_extras::typed::Type::literal(#names),)*])
                            ),
                            (
                                #underscore_name,
                                mlua_extras::typed::Type::class(mlua_extras::typed::TypedClassBuilder::new::<Self>())
                            ),
                            #(#variants,)*
                        ]
                    }

                    fn as_param() -> mlua_extras::typed::Param {
                        mlua_extras::typed::Param {
                            doc: None,
                            name: None,
                            ty: mlua_extras::typed::Type::named(#label),
                        }
                    }
                }
            )
        },
        Err(err) => abort!(err.span(), "{}", err),
        _ => abort!(input.span(), "only `struct` and `enum` types are supported for Typed")
    }.into()
}
