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

/// Generates a [mlua::UserData] implementation from struct fields.
/// 
/// Each named and unnamed field is automatically exposed to Lua as a read and/or write property or index.
/// 
/// Use `#[field(...)]` attributes to controll access and naming:
/// 
/// - `readonly`: Set the field to only be readable within Lua
/// - `writeonly`: Set the field to only be writable within Lua
/// - `skip`: Ignore generating and exposing the field
/// - `rename`: Rename the field to a string for a named field and a digit for an indexed field
/// 
/// > Note: `readonly` + `writeonly` together is the same as having neither, the field will be exposed
/// > for both read and write.
/// 
/// Optionally combine with [`macro@user_data_impl`] to also register methods in a rust like manner.
/// 
/// # Example
/// 
/// ```ignore
/// #[derive(Clone, UserData)]
/// struct Player {
///     name: String,
///     health: f64,
///     #[field(skip)]
///     handle: u64,
///     #[field(readonly)]
///     score: i32,
///     #[field(rename = "pos_x")]
///     position_x: f64,
/// }
/// ```
/// 
/// ```ignore
/// #[derive(Clone, UserData)]
/// enum PlayerAction {
///     Idle,
///     Move {
///         x: i32,
///         y: i32
///     },
///     Attack(
///         #[field(rename = "name")]
///         String
///     ),
///     Quit,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(UserData, attributes(field))]
pub fn derive_user_data(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    userdata::derive(input).into()
}

/// Attribute macro that registers methods from an `impl` block for use in Lua.
/// 
/// Used on an `impl` block for a type that derives [`UserData`](macro@UserData), this
/// macro will register methods annotated with `#[method]` and `#[metamethod(...)]`.
/// 
/// # Attributes
/// 
/// - `#[method]`
///   - `#[method(rename = "name")`: register as a method with the provided name
/// - `#[metamethod(...)]`
///   - `#[metamethod(ToString)]`: register as a metamethod as a [`mlua::MetaMethod`] variant
///   - `#[metamethod("__custom")]`: register as a custom named metamethod
/// 
/// # Patterns
/// 
/// - `&self`: registered with `mlua::UserDataMethods::add_method` or `mlua::UserDataMethods::add_meta_method`
/// - `&mut self`: registered with `mlua::UserDataMethods::add_method_mut` or `mlua::UserDataMethods::add_meta_method_mut`
/// - without `self`: registered with `mlua::UserDataMethods::add_function` or `mlua::UserDataMethods::add_meta_function`
/// - `async fn`: registered with the `mlua::UserDataMethods::add_async_*` variant that matches the above arguments
/// - If the first non `self` parameter is `lua` then `&mlua::Lua` is passed to non async methods/functions
///     and `mlua::Lua` is passed into async methods/functions
/// 
/// # Return
/// 
/// - `Result<T, E>` where `E: Into<mlua::Error>`: Method is fallible and the error is automatically converted to a [`mlua::Error`].
///     This includes any error type that implements [`mlua::ExternalError`] and any return type that has the name `Result`.
/// - `T`: Method is infallible and is wrapped with `Ok(...)` when registered
/// - `()`: Method is infallible and has no return value. Registration returns `Ok(())`
/// 
/// All methods stay as is and stay as regular callable rust functions. Any methods without `#[method]` or `#[metamethod(...)]` will not be registered.
/// 
/// # Example
/// 
/// ```ignore
/// #[derive(Clone, UserData)]
/// struct Counter { value: i64 }
/// 
/// #[user_data_impl]
/// impl Counter {
///     #[method]
///     fn get(&self) -> i64 { self.value }
/// 
///     #[method]
///     fn increment(&mut self) { self.value += 1 }
/// 
///     #[method]
///     fn create_table(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
///         lua.create_table()
///     }
/// 
///     #[metamethod(ToString)]
///     fn to_string(&self) -> String { format!("Counter({})", self.value) }
/// 
///     // Requires the `async` feature
///     // Must be accessed from lua code with an entry of `mlua::Chunk::eval_async` or `mlua::Chunk::exec_async`
///     #[method]
///     async fn fetch(&self, url: String) -> mlua::Result<String> {
///         Ok(format!("fetched: {url}"))
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn user_data_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemImpl);
    methods::derive(item).into()
}

/// Generates a [`Typed`](mlua_extras::Typed) implementation from fields.
/// 
/// Only supports structs and enums.
/// 
/// This registers the target as a new Lua type that can be used
/// to generate documentation.
/// 
/// # Structs
/// 
/// Assigned as a lua `class` with it's registered fields, indexes, methods,
/// functions and their meta variants.
/// 
/// # Enums
/// 
/// Assigned as an alias to a union of `class`es where each `class` is a enum variant. This
/// is to best represent rust's use of enums as unions.
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
