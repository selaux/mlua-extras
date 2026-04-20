use darling::FromField;
use proc_macro::Literal;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Data, DeriveInput, Fields, LitInt, spanned::Spanned};
use quote::quote;

use crate::extract::{Index, UserDataField};

pub fn derive(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let fields: Vec<_> = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().collect(),
            Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
            Fields::Unit => Vec::new(),
        },
        Data::Enum(_) => {
            proc_macro_error::abort!(name, "TypedUserData does not support enums");
        }
        Data::Union(_) => {
            proc_macro_error::abort!(name, "TypedUserData does not support unions");
        }
    };

    let user_fields = fields
        .iter()
        .map(|field| {
            match UserDataField::from_field(field) {
                Ok(uf) => uf,
                Err(err) => proc_macro_error::abort!(field, "{}", err)
            }
        })
        .collect::<Vec<UserDataField>>();

    let field_registrations = user_fields
        .iter()
        .enumerate()
        .filter_map(|(i, fi)| {
            let (index, field_ident) = match (&fi.rename, &fi.ident) {
                (None, None) | (Some(Index::Int(_)), _) => return None,
                (Some(Index::Str(v)), _) => (Index::Str(v.clone()), match &fi.ident {
                    Some(i) => quote!(#i),
                    None => {
                        let i = LitInt::new(&Literal::usize_unsuffixed(i).to_string(), fi.ty.span());
                        quote!(#i)
                    }
                }),
                (None, Some(ident)) => (Index::Str(ident.to_string()), quote!(#ident)),
            };

            let field_ty = &fi.ty;

            match (fi.skip, fi.readonly, fi.writeonly) {
                (true, _, _) => None,
                (_, true, true) | (_, false, false) => Some(quote! {
                    mlua_extras::extras::UserDataGetSet::<Self>::add_field_method_get_set(
                        fields,
                        #index,
                        |_lua, this| Ok(this.#field_ident.clone()),
                        |_lua, this, val: #field_ty| { this.#field_ident = val; Ok(()) },
                    );
                }),
                (_, true, false) => Some(quote! {
                    fields.add_field_method_get(
                        #index,
                        |_lua, this| Ok(this.#field_ident.clone()),
                    );
                }),
                (_, false, true) => Some(quote! {
                    fields.add_field_method_set(
                        #index,
                        |_lua, this, val: #field_ty| { this.#field_ident = val; Ok(()) },
                    );
                }),
            }
        });

    let mut method_registrations = Vec::<TokenStream2>::new();

    // Add a custom __index and __newindex for the tuple struct/enum fields
    // this will always attempt to fallback to the user definend #[metamethod(Index)] or #[metamethod(NewIndex)]
    {
        let indexes = user_fields
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                match f {
                    UserDataField { skip: true, .. }
                    | UserDataField { ident: Some(_), rename: None|Some(Index::Str(_)), .. }
                    | UserDataField { rename: Some(Index::Str(_)), .. }
                    | UserDataField { readonly: false, writeonly: true, .. } => None,
                    UserDataField { readonly: true, writeonly: true, .. }
                    | UserDataField { readonly: false, writeonly: false, .. }
                    | UserDataField { readonly: true, writeonly: false, .. } => {
                        let idx = match &f.ident {
                            Some(i) => quote!(#i),
                            None => {
                                let i = LitInt::new(&Literal::isize_unsuffixed(i as isize).to_string(), Span::call_site());
                                quote!(#i)
                            }
                        };
                        let lua_idx = LitInt::new(&Literal::isize_unsuffixed(match f.rename {
                            Some(Index::Int(v)) => v,
                            _ => i as isize + 1
                        }).to_string(), Span::call_site());

                        Some(quote!(Some(#lua_idx) => return mlua_extras::mlua::IntoLua::into_lua(this.#idx.clone(), lua),))
                    },
                }
            }).collect::<Vec<_>>();

        let new_indexes = user_fields
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                match f {
                    UserDataField { skip: true, .. }
                    | UserDataField { ident: Some(_), rename: None|Some(Index::Str(_)), .. }
                    | UserDataField { rename: Some(Index::Str(_)), .. }
                    | UserDataField { readonly: true, writeonly: false, .. } => None,
                    UserDataField { readonly: true, writeonly: true, .. }
                    | UserDataField { readonly: false, writeonly: false, .. }
                    | UserDataField { readonly: false, writeonly: true, .. } => {
                        let idx = match &f.ident {
                            Some(i) => quote!(#i),
                            None => {
                                let i = LitInt::new(&Literal::isize_unsuffixed(i as isize).to_string(), Span::call_site());
                                quote!(#i)
                            }
                        };
                        let lua_idx = Some(LitInt::new(&Literal::isize_unsuffixed(match f.rename {
                            Some(Index::Int(v)) => v,
                            _ => i as isize + 1
                        }).to_string(), Span::call_site()));
                        let ty = &f.ty;

                        Some(quote!(Some(#lua_idx) => this.#idx = <#ty as mlua_extras::mlua::FromLua>::from_lua(value, lua)?,))
                    },
                }
            }).collect::<Vec<_>>();

        method_registrations.push(quote!{
            methods.add_meta_function(mlua_extras::mlua::MetaMethod::Index, |lua, (this, idx): (mlua_extras::mlua::AnyUserData, mlua_extras::mlua::Value)| {
                let metatable = this.metatable()?;
                
                if let Ok(usr) = metatable.get::<mlua_extras::mlua::Function>("__usr_index") {
                    match usr.call::<mlua_extras::mlua::Value>((this.clone(), idx.clone()))? {
                        mlua_extras::mlua::Value::Nil => (),
                        other => return Ok(other)
                    };
                }
                
                let this = this.borrow::<Self>()?;
                match idx.as_integer() {
                    #(#indexes)*
                    _ => Ok(mlua_extras::mlua::Value::Nil)
                }
            });

            methods.add_meta_function(mlua_extras::mlua::MetaMethod::NewIndex, |lua, (this, idx, value): (mlua_extras::mlua::AnyUserData, mlua_extras::mlua::Value, mlua_extras::mlua::Value)| {
                let metatable = this.metatable()?;

                let mut error = None;
                if let Ok(usr) = metatable.get::<mlua_extras::mlua::Function>("__usr_newindex") {
                    match usr.call::<Option<mlua_extras::mlua::Value>>((this.clone(), idx.clone(), value.clone())) {
                        Ok(v) => return Ok(()),
                        Err(err) => error = Some(err),
                    }
                }

                let mut this = this.borrow_mut::<Self>()?;
                match idx.as_integer() {
                    #(#new_indexes)*
                    _ => return Err(error.unwrap_or(mlua_extras::mlua::Error::runtime(match idx {
                        mlua_extras::mlua::Value::Integer(i) => format!("invalid index '{i}'"),
                        mlua_extras::mlua::Value::String(s) => format!("invalid index '{}'", s.to_string_lossy()),
                        _ => "invalid index".into()
                    })))
                }
                Ok(())
            });
        });
    }

    quote! {
        impl #name {
            #[doc(hidden)]
            fn __auto_add_fields<F: mlua_extras::mlua::UserDataFields<Self>>(fields: &mut F) {
                #(#field_registrations)*
            }
            #[doc(hidden)]
            fn __implicit_methods<M: mlua_extras::mlua::UserDataMethods<Self>>(methods: &mut M) {
                #(#method_registrations)*
            }
        }

        impl mlua_extras::mlua::UserData for #name {
            fn add_fields<F: mlua_extras::mlua::UserDataFields<Self>>(fields: &mut F) {
                Self::__auto_add_fields(fields);
            }

            fn add_methods<M: mlua_extras::mlua::UserDataMethods<Self>>(methods: &mut M) {
                Self::__implicit_methods(methods);

                use mlua_extras::__DefaultAutoMethods as _;
                Self::__auto_add_methods(methods);
            }
        }
    }
}
