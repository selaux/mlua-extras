#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{proc_macro_error, abort};
use syn::spanned::Spanned;
use venial::{Item, parse_item};

mod methods;
mod userdata;
pub(crate) mod extract;

#[proc_macro_error]
#[proc_macro_derive(UserData, attributes(field))]
pub fn derive_user_data(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    userdata::derive(input).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn user_data_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemImpl);
    methods::derive(item).into()
}

#[proc_macro_error]
#[proc_macro_derive(Typed)]
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

                    fn as_param() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::named(#label)
                    }

                    fn as_return() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::named(#label)
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

            let enum_alt = format!("{label}Variant");
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

                    fn as_param() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::named(#label)
                    }

                    fn as_return() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::named(#label)
                    }
                }
            )
        },
        Err(err) => abort!(err.span(), "{}", err),
        _ => abort!(input.span(), "only `struct` and `enum` types are supported for Typed")
    }.into()
}
