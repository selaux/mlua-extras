use std::collections::{BTreeMap, btree_map::Entry};

use darling::FromField;
use proc_macro::Literal;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Data, DeriveInput, Fields, LitInt, spanned::Spanned};
use quote::quote;

use crate::extract::{Index, UserDataEnumField, UserDataField};

pub fn derive(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => {
            let fields: Vec<_> = match &data.fields {
                Fields::Named(named) => named.named.iter().collect(),
                Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
                Fields::Unit => Vec::new(),
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

            derive_struct(name, user_fields)
        },
        Data::Enum(data) => {
            let mut enum_fields: BTreeMap<Index, Vec<UserDataEnumField>> = Default::default();
            let mut variants = Vec::new();

            for variant in data.variants.iter() {
                let vn = &variant.ident;
                let (variant, fields) = match &variant.fields {
                    Fields::Named(named) => {
                        let fields = named.named.iter().filter_map(|v| v.ident.as_ref().map(|v| {
                            let n = format_ident!("_{v}");
                            quote!(#v: #n)
                        }));

                        variants.push((quote!(#vn{ .. }), vn));
                        (quote!(#vn{ #(#fields,)* }), named.named.iter().collect())
                    },
                    Fields::Unnamed(unnamed) => {
                        let fields = (0..unnamed.unnamed.len()).map(|v| format_ident!("_{v}"));

                        let v = quote!(#vn( #(#fields,)* ));
                        variants.push((v.clone(), vn));
                        (v, unnamed.unnamed.iter().collect())
                    },
                    Fields::Unit => {
                        let v = quote!(#vn);
                        variants.push((v.clone(), vn));
                        (v, Vec::new())
                    }
                };

                for (i, field) in fields.iter().enumerate() {
                    match UserDataEnumField::from_field(field) {
                        Ok(mut uf) => {
                            if uf.skip {
                                continue;
                            }

                            uf.variant = variant.clone();
                            uf.accessor = match uf.ident.as_ref() {
                                Some(ident) => {
                                    let i = format_ident!("_{ident}");
                                    quote!(#i)
                                },
                                None => {
                                    let i = format_ident!("_{i}");
                                    quote!(#i)
                                }
                            };

                            let idx = match uf.rename.clone().or_else(|| uf.ident.as_ref().map(|v| Index::Str(v.to_string())))  {
                                Some(n) => n,
                                None => Index::Int(i as isize + 1)
                            };

                            match enum_fields.entry(idx) {
                                Entry::Occupied(mut entry) => entry.get_mut().push(uf),
                                Entry::Vacant(entry) => { entry.insert(vec![uf]); }
                            }
                        },
                        Err(err) => proc_macro_error::abort!(field, "{}", err)
                    }
                }
            }

            derive_enum(name, variants, enum_fields)
        }
        Data::Union(_) => {
            proc_macro_error::abort!(name, "TypedUserData does not support unions");
        }
    }
}

fn derive_struct(name: &syn::Ident, user_fields: Vec<UserDataField>) -> TokenStream2 {
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
                        |_lua, this, _value: #field_ty| { this.#field_ident = _value; Ok(()) },
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
                        |_lua, this, _value: #field_ty| { this.#field_ident = _value; Ok(()) },
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

                        Some(quote!(Some(#lua_idx) => return mlua_extras::mlua::IntoLua::into_lua(this.#idx.clone(), _lua),))
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

                        Some(quote!(Some(#lua_idx) => this.#idx = <#ty as mlua_extras::mlua::FromLua>::from_lua(_value.clone(), _lua)?,))
                    },
                }
            }).collect::<Vec<_>>();

        method_registrations.push(quote!{
            methods.add_meta_function(mlua_extras::mlua::MetaMethod::Index, |_lua, (this, _idx): (mlua_extras::mlua::AnyUserData, mlua_extras::mlua::Value)| {
                {
                    let this = this.borrow::<Self>()?;
                    match _idx.as_integer() {
                        #(#indexes)*
                        _ => ()
                    }
                }

                let metatable = this.metatable()?;
                if let Ok(usr) = metatable.get::<mlua_extras::mlua::Function>("__usr_index") {
                    return usr.call::<mlua_extras::mlua::Value>((this.clone(), _idx.clone()));
                }

                Ok(mlua_extras::mlua::Value::Nil)
            });

            methods.add_meta_function(mlua_extras::mlua::MetaMethod::NewIndex, |_lua, (this, _idx, _value): (mlua_extras::mlua::AnyUserData, mlua_extras::mlua::Value, mlua_extras::mlua::Value)| {
                {
                    let mut this = this.borrow_mut::<Self>()?;
                    match _idx.as_integer() {
                        #(#new_indexes)*
                        _ => ()
                    }
                }

                let metatable = this.metatable()?;
                if let Ok(usr) = metatable.get::<mlua_extras::mlua::Function>("__usr_newindex") {
                    return usr.call::<Option<mlua_extras::mlua::Value>>((this.clone(), _idx.clone(), _value));
                }
                
                Err(mlua_extras::mlua::Error::runtime(match _idx {
                    mlua_extras::mlua::Value::Integer(i) => format!("type does not contain index '{i}'"),
                    mlua_extras::mlua::Value::String(s) => format!("type does not contain field '{}'", s.to_string_lossy()),
                    _ => "type does not contain index".into()
                }))
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

fn derive_enum(name: &syn::Ident, enum_variants: Vec<(TokenStream2, &syn::Ident)>, user_fields: BTreeMap<Index, Vec<UserDataEnumField>>) -> TokenStream2 {
    let count = enum_variants.len();

    let field_registrations = user_fields
        .iter()
        .filter(|(idx, _fi)| idx.is_str())
        .map(|(idx, fi)| {
            let has_get = fi.iter().any(|f| f.readonly || (!f.readonly && !f.writeonly));
            let has_set = fi.iter().any(|f| f.writeonly || (!f.readonly && !f.writeonly));

            let getter = {
                let variants: Vec<_> = fi
                    .iter()
                    .filter(|f| f.readonly || (!f.readonly && !f.writeonly))
                    .map(|v| {
                        let variant = &v.variant;
                        let accessor = &v.accessor;
                        quote!(Self::#variant => #accessor.clone(),)
                    })
                    .collect();

                let err_msg = format!("type variant does not contain field '{idx}'");
                let catchall = if variants.len() < count {
                    quote!{ _ => return Err(mlua_extras::mlua::Error::runtime(#err_msg)), }
                } else {
                    quote!()
                };

                quote!(Ok(match this {
                    #(#variants)*
                    #catchall
                }))
            };

            let setter = {
                let variants: Vec<_> = fi
                    .iter()
                    .filter(|f| f.writeonly || (!f.readonly && !f.writeonly))
                    .map(|v| {
                        let variant = &v.variant;
                        let accessor = &v.accessor;
                        let ty = &v.ty;
                        quote!(Self::#variant => *#accessor = <#ty as mlua_extras::mlua::FromLua>::from_lua(_value, _lua)?,)
                    })
                    .collect();

                let err_msg = format!("type variant does not contain field '{idx}'");
                let catchall = if variants.len() < count {
                    quote!{ _ => return Err(mlua_extras::mlua::Error::runtime(#err_msg)), }
                } else {
                    quote!()
                };

                quote!({
                    match this {
                        #(#variants)*
                        #catchall
                    }
                    Ok(())
                })
            };

            match (has_get, has_set) {
                (true, true) | (false, false) => Some(quote! {
                    mlua_extras::extras::UserDataGetSet::<Self>::add_field_method_get_set(
                        fields,
                        #idx,
                        |_lua, this| #getter,
                        |_lua, this, _value: mlua_extras::mlua::Value| #setter,
                    );
                }),
                (true, false) => Some(quote! {
                    fields.add_field_method_get(
                        #idx,
                        |_lua, this| #getter,
                    );
                }),
                (false, true) => Some(quote! {
                    fields.add_field_method_set(
                        #idx,
                        |_lua, this, _value: mlua_extras::mlua::Value| #setter,
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
            .filter(|(idx, _fi)| !idx.is_str())
            .filter_map(|(idx, f)| {
                let variants: Vec<_> = f
                    .iter()
                    .filter(|f| f.readonly || (!f.readonly && !f.writeonly))
                    .map(|f| {
                        let variant = &f.variant;
                        let accessor = &f.accessor;
                        quote!{
                            Self::#variant => return mlua_extras::mlua::IntoLua::into_lua(#accessor.clone(), _lua),
                        }
                    })
                    .collect();

                if variants.is_empty() {
                    return None;
                }

                let catchall = if variants.len() < count {
                    quote!{ _ => () }
                } else {
                    quote!()
                };

                Some(quote! {
                    Some(#idx) => match &*this {
                        #(#variants)*
                        #catchall
                    }
                })
            }).collect::<Vec<_>>();

        let new_indexes = user_fields
            .iter()
            .filter(|(idx, _fi)| !idx.is_str())
            .filter_map(|(idx, f)| {
                let variants: Vec<_> = f
                    .iter()
                    .filter(|f| f.writeonly || (!f.readonly && !f.writeonly))
                    .map(|f| {
                        let variant = &f.variant;
                        let accessor = &f.accessor;
                        let ty = &f.ty;
                        quote!{
                            Self::#variant => *#accessor = <#ty as mlua_extras::mlua::FromLua>::from_lua(_value.clone(), _lua)?,
                        }
                    })
                    .collect();

                if variants.is_empty() {
                    return None;
                }

                let catchall = if variants.len() < count {
                    quote!{ _ => () }
                } else {
                    quote!()
                };

                Some(quote! {
                    Some(#idx) => match &mut *this {
                        #(#variants)*
                        #catchall
                    }
                })
            }).collect::<Vec<_>>();

        method_registrations.push(quote!{
            methods.add_meta_function(mlua_extras::mlua::MetaMethod::Index, |_lua, (this, _idx): (mlua_extras::mlua::AnyUserData, mlua_extras::mlua::Value)| {
                {
                    let this = this.borrow::<Self>()?;
                    match _idx.as_integer() {
                        #(#indexes)*
                        _ => ()
                    }
                }

                let metatable = this.metatable()?;
                if let Ok(usr) = metatable.get::<mlua_extras::mlua::Function>("__usr_index") {
                    return usr.call::<mlua_extras::mlua::Value>((this.clone(), _idx.clone()));
                }
                
                Ok(mlua_extras::mlua::Value::Nil)
            });

            methods.add_meta_function(mlua_extras::mlua::MetaMethod::NewIndex, |_lua, (this, _idx, _value): (mlua_extras::mlua::AnyUserData, mlua_extras::mlua::Value, mlua_extras::mlua::Value)| {
                {
                    let mut this = this.borrow_mut::<Self>()?;
                    match _idx.as_integer() {
                        #(#new_indexes)*
                        _ => ()
                    }
                }

                let metatable = this.metatable()?;
                if let Ok(usr) = metatable.get::<mlua_extras::mlua::Function>("__usr_newindex") {
                    return usr.call::<Option<mlua_extras::mlua::Value>>((this.clone(), _idx.clone(), _value));
                }

                Err(mlua_extras::mlua::Error::runtime(match _idx {
                    mlua_extras::mlua::Value::Integer(i) => format!("type variant does not contain index '{i}'"),
                    mlua_extras::mlua::Value::String(s) => format!("type variant does not contain field '{}'", s.to_string_lossy()),
                    _ => "type variant does not contain index".into()
                }))
            });
        });
    }

    let variants = enum_variants.iter().map(|(v, n)| {
        let name = n.to_string();
        quote!(Self::#v => #name)
    });


    quote! {
        impl #name {
            #[doc(hidden)]
            fn __auto_add_fields<F: mlua_extras::mlua::UserDataFields<Self>>(fields: &mut F) {
                fields.add_field_method_get("_variant", |_lua, this| {
                    Ok(match this {
                        #(#variants,)*
                    })
                });

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