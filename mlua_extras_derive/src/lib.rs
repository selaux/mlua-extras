#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

use crate::builder::Builder;

mod methods;
mod userdata;
pub(crate) mod extract;
pub(crate) mod builder;

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
    Builder::regular().derive_fields(input).into()
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
    Builder::regular().derive_methods(item).into()
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
    let item = syn::parse_macro_input!(input as syn::DeriveInput);
    Builder::derive_typed(&item, false).into()
}

/// Derive macro that generates a `TypedUserData` implementation from struct fields.
///
/// Each named field is automatically exposed to Lua as a read/write property.
/// Use `#[mlua_extras(...)]` attributes on fields to control access:
///
/// - `#[mlua_extras(skip)]` — field is not exposed to Lua
/// - `#[mlua_extras(readonly)]` — getter only
/// - `#[mlua_extras(writeonly)]` — setter only
/// - `#[mlua_extras(rename = "lua_name")]` — use a different name in Lua
///
/// Doc comments on fields are forwarded to the type metadata system.
///
/// This also generates the `mlua::UserData` impl, so you do not need to
/// separately derive `UserData`.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, TypedUserData)]
/// struct Player {
///     /// The player's display name
///     name: String,
///     health: f64,
///     #[mlua_extras(skip)]
///     internal_id: u64,
///     #[mlua_extras(readonly)]
///     score: i32,
///     #[mlua_extras(rename = "pos_x")]
///     position_x: f64,
/// }
/// ```
///
/// Optionally combine with [`macro@typed_user_data_impl`] to also register methods.
/// See `tests/lua_user_data.rs` for more exhaustive examples.
#[proc_macro_error]
#[proc_macro_derive(TypedUserData, attributes(field))]
pub fn derive_typed_user_data(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    Builder::typed().derive_fields(input).into()
}

// /// Attribute macro that registers methods from an `impl` block for use in Lua.
// ///
// /// Place on an `impl` block for a type that derives [`TypedUserData`](macro@TypedUserData).
// /// Annotate individual methods with `#[method]` or `#[metamethod(...)]`.
// ///
// /// # Method attributes
// ///
// /// - `#[method]` — register as a regular Lua method/function
// /// - `#[method(rename = "lua_name")]` — register under a different Lua name
// /// - `#[metamethod(ToString)]` — register as a metamethod (any `mlua::MetaMethod` variant)
// /// - `#[metamethod("__custom")]` — register a custom-named metamethod
// ///
// /// # Receiver handling
// ///
// /// - `&self` → `add_method`
// /// - `&mut self` → `add_method_mut`
// /// - no `self` → `add_function`
// /// - `async fn` with `&self` → `add_async_method`
// /// - `async fn` no `self` → `add_async_function`
// ///
// /// # Optional `lua` parameter
// ///
// /// If the first non-self parameter is named `lua`, it receives the Lua context
// /// from the closure and is not part of the Lua-side argument list.
// ///
// /// # Return types
// ///
// /// - `-> Result<T, E>` where `E: Into<mlua::Error>` — fallible; the error is
// ///   converted via `.into()`. This includes `mlua::Result<T>`, `mlua::Error`,
// ///   `anyhow::Error` (with the mlua `anyhow` feature), `std::io::Error`, and
// ///   any type implementing mlua's `ExternalError` trait.
// /// - `-> T` — infallible, wrapped in `Ok(...)`
// /// - no return / `-> ()` — returns `Ok(())`
// ///
// /// # Example
// ///
// /// ```ignore
// /// #[derive(Clone, TypedUserData)]
// /// struct Counter { value: i64 }
// ///
// /// #[mlua_extras::typed_user_data_impl]
// /// impl Counter {
// ///     #[method]
// ///     fn get(&self) -> i64 { self.value }
// ///
// ///     #[method]
// ///     fn increment(&mut self) { self.value += 1; }
// ///
// ///     #[method]
// ///     fn create_table(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
// ///         lua.create_table()
// ///     }
// ///
// ///     #[metamethod(ToString)]
// ///     fn to_string(&self) -> String { format!("Counter({})", self.value) }
// ///
// ///     #[method]
// ///     async fn fetch(&self, url: String) -> mlua::Result<String> {
// ///         Ok(format!("fetched: {url}"))
// ///     }
// /// }
// /// ```
// ///
// /// Methods without `#[method]` or `#[metamethod(...)]` are left as normal Rust
// /// methods, callable from Rust but not registered with Lua.
// ///
// /// See `tests/lua_user_data.rs` for more exhaustive examples.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn typed_user_data_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemImpl);
    Builder::typed().derive_methods(item).into()
}