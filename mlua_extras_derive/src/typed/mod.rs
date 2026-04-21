use proc_macro2::TokenStream;
use syn::Data;

pub mod methods;
pub mod user_data;

pub fn derive(input: syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    match input.data {
        Data::Struct(_) => {
            let label = name.to_string();
            quote!(
                impl mlua_extras::typed::Typed for #name {
                    fn ty() -> mlua_extras::typed::Type {
                        mlua_extras::typed::Type::class(mlua_extras::typed::TypedClassBuilder::new::<#name>().build())
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
        Data::Enum(enum_type) => {
            let label = name.to_string();
            let underscore_name = format!("_{name}");

            let named = enum_type.variants.iter().map(|variant| format!("{label}{}", variant.ident)).collect::<Vec<_>>();
            let variants = enum_type.variants
                .iter()
                .map(|variant| {
                    let name = format!("{label}{}", variant.ident);
                    quote!{
                        (
                            #name,
                            mlua_extras::typed::Type::class(
                                mlua_extras::typed::TypedClassBuilder::default()
                                    .derive(#underscore_name)
                                    .build()
                            )
                        )
                    }
                })
                .collect::<Vec<_>>();

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
                                #underscore_name,
                                mlua_extras::typed::Type::class(mlua_extras::typed::TypedClassBuilder::new::<Self>().build())
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
        _ => proc_macro_error::abort!(input, "only `struct` and `enum` types are supported for Typed")
    }.into()
}