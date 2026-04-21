use proc_macro2::TokenStream;
use syn::{ImplItem, ItemImpl, Type};
use quote::quote;

use crate::extract::{MethodKind, PassBy, UserDataField, UserDataMethod, is_field_attr, is_metamethod_attr, is_method_attr};

pub fn derive(item: ItemImpl) -> TokenStream {
    let self_ty = &item.self_ty;

    let mut user_data_methods = Vec::new();
    let mut user_data_fields = Vec::new();

    let mut cleaned_items = Vec::new();

    for impl_item in &item.items {
        match impl_item {
            ImplItem::Fn(method) => if let Some(udm) = UserDataMethod::from_impl_fn(method) {
                user_data_methods.push(udm);

                let mut cleaned = method.clone();
                cleaned.attrs.retain(|a| !MethodKind::is_attr(a));
                cleaned_items.push(ImplItem::Fn(cleaned));
            } else {
                cleaned_items.push(impl_item.clone());
            }
            ImplItem::Const(const_expr) => if let Some(udf) = UserDataField::from_impl_const(const_expr) {
                user_data_fields.push(udf);

                let mut cleaned = const_expr.clone();
                cleaned.attrs.retain(|a| !is_field_attr(a));
                cleaned_items.push(ImplItem::Const(cleaned));
            } else {
                cleaned_items.push(impl_item.clone());
            },
            _ => {
                cleaned_items.push(impl_item.clone());
            }
        }
    }

    let method_registrations: Vec<_> = user_data_methods
        .iter()
        .filter(|i| !i.kind.is_field())
        .map(|info| generate_method_registration(info, self_ty))
        .collect();

    let field_registrations: Vec<_> = user_data_methods
        .iter()
        .filter(|i| i.kind.is_field())
        .map(|info| generate_field_registration(info))
        .collect();

    let static_field_registration: Vec<_> = user_data_fields
        .iter()
        .map(|info| generate_static_field_registration(info))
        .collect();

    // Reconstruct the cleaned impl block
    let attrs = &item.attrs;
    let unsafety = &item.unsafety;
    let impl_token = &item.impl_token;
    let generics = &item.generics;

    quote! {
        #(#attrs)*
        #unsafety #impl_token #generics #self_ty {
            #(#cleaned_items)*
        }

        impl #generics #self_ty {
            #[doc(hidden)]
            fn __auto_add_fields<F: mlua_extras::mlua::UserDataFields<Self>>(fields: &mut F) {
                #(#static_field_registration)*
                #(#field_registrations)*
            }

            #[doc(hidden)]
            fn __auto_add_methods<M: mlua_extras::mlua::UserDataMethods<Self>>(methods: &mut M) {
                #(#method_registrations)*
            }
        }
    }
}

fn generate_method_registration(info: &UserDataMethod, self_ty: &Type) -> TokenStream {
    let fn_name = &info.name;
    let lua_name = &info.lua_name;

    let param_names: Vec<_> = info.params.iter().map(|(name, _)| name).collect();
    let param_types: Vec<_> = info.params.iter().map(|(_, ty)| ty).collect();

    // Build the parameter destructuring for the closure
    let params_destructure = if param_names.is_empty() {
        quote! { _: () }
    } else {
        quote! { (#(#param_names,)*): (#(#param_types,)*) }
    };

    // Build the method call arguments
    let call_args = if info.lua {
        let args = &param_names;
        quote! { lua, #(#args,)* }
    } else {
        let args = &param_names;
        quote! { #(#args,)* }
    };

    let lua_ident = if info.lua {
        quote! { lua }
    } else {
        quote! { _lua }
    };

    // Build the method call and return wrapping
    let build_call_and_return = |call: TokenStream| -> TokenStream {
        if info.fallible {
            quote! { #call.map_err(|e| e.into()) }
        } else if info.returnable {
            quote! {
                let result = #call;
                Ok(result)
            }
        } else {
            quote! {
                #call;
                Ok(())
            }
        }
    };

    let is_meta = info.kind.is_meta();

    if info.r#async {
        // Async methods
        match &info.instance {
            Some(PassBy::Ref) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args).await });
                quote! {
                    methods.add_async_method(#lua_name, |#lua_ident, this, #params_destructure| async move {
                        #body
                    });
                }
            }
            Some(PassBy::RefMut) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args).await });
                quote! {
                    methods.add_async_method_mut(#lua_name, |#lua_ident, this, #params_destructure| async move {
                        #body
                    });
                }
            }
            None => {
                let body =
                    build_call_and_return(quote! { #self_ty::#fn_name(#call_args).await });
                quote! {
                    methods.add_async_function(#lua_name, |#lua_ident, #params_destructure| async move {
                        #body
                    });
                }
            }
        }
    } else {
        // Sync methods
        match (&info.instance, is_meta) {
            (Some(PassBy::Ref), false) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
                quote! {
                    methods.add_method(#lua_name, |#lua_ident, this, #params_destructure| {
                        #body
                    });
                }
            }
            (Some(PassBy::Ref), true) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
                quote! {
                    methods.add_meta_method(#lua_name, |#lua_ident, this, #params_destructure| {
                        #body
                    });
                }
            }
            (Some(PassBy::RefMut), false) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
                quote! {
                    methods.add_method_mut(#lua_name, |#lua_ident, this, #params_destructure| {
                        #body
                    });
                }
            }
            (Some(PassBy::RefMut), true) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
                quote! {
                    methods.add_meta_method_mut(#lua_name, |#lua_ident, this, #params_destructure| {
                        #body
                    });
                }
            }
            (None, false) => {
                let body =
                    build_call_and_return(quote! { #self_ty::#fn_name(#call_args) });
                quote! {
                    methods.add_function(#lua_name, |#lua_ident, #params_destructure| {
                        #body
                    });
                }
            }
            (None, true) => {
                let body =
                    build_call_and_return(quote! { #self_ty::#fn_name(#call_args) });
                quote! {
                    methods.add_meta_function(#lua_name, |#lua_ident, #params_destructure| {
                        #body
                    });
                }
            }
        }
    }
}

fn generate_static_field_registration(info: &UserDataField) -> TokenStream {
    let name = info.ident.as_ref().unwrap();
    let lua_name = info.rename.clone().map(|v| v.to_string()).unwrap_or_else(|| name.to_string());

    quote! { fields.add_field(#lua_name, Self::#name); }
}

fn generate_field_registration(info: &UserDataMethod) -> TokenStream {
    let fn_name = &info.name;
    let lua_name = &info.lua_name;

    let param_names: Vec<_> = info.params.iter().map(|(name, _)| name).collect();
    let param_types: Vec<_> = info.params.iter().map(|(_, ty)| ty).collect();

    // Build the parameter destructuring for the closure
    let params_destructure = if param_names.is_empty() {
        quote! { _: () }
    } else if param_names.len() == 1 {
        quote! { #(#param_names)*: #(#param_types)* }
    } else {
        quote! { (#(#param_names,)*): (#(#param_types,)*) }
    };

    // Build the method call arguments
    let call_args = if info.lua {
        let args = &param_names;
        quote! { lua, #(#args,)* }
    } else {
        let args = &param_names;
        quote! { #(#args,)* }
    };

    let lua_ident = if info.lua {
        quote! { lua }
    } else {
        quote! { _lua }
    };

    // Build the method call and return wrapping
    let build_call_and_return = |call: TokenStream| -> TokenStream {
        if info.fallible {
            quote! { #call.map_err(|e| e.into()) }
        } else if info.returnable {
            quote! {
                let result = #call;
                Ok(result)
            }
        } else {
            quote! {
                #call;
                Ok(())
            }
        }
    };

    // Sync methods
    match (&info.instance, info.kind.is_getter(), param_names.len()) {
        (Some(PassBy::RefMut|PassBy::Ref), false, 1) => {
            let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
            quote! {
                fields.add_field_method_set(#lua_name, |#lua_ident, this, #params_destructure| {
                    #body
                });
            }
        }
        (Some(PassBy::Ref), true, 0) => {
            let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
            quote! {
                fields.add_field_method_get(#lua_name, |#lua_ident, this| {
                    #body
                });
            }
        }
        (passby, getter, params) => {
            if passby.is_none() {
                proc_macro_error::abort!(info.name, "missing 'self' parameter")
            }

            if getter {
                if let Some(PassBy::RefMut) = passby {
                    proc_macro_error::abort!(info.params[0].0, "'self' parameter must be a not mutable reference")
                } else if params != 0 {
                    proc_macro_error::abort!(info.params[0].0, "field getters must take 0 arguments")
                }
                proc_macro_error::abort!(info.params[0].0, "invalid getter arguments")
            } else if params != 1 {
                proc_macro_error::abort!(info.params[0].0, "field setters must take exactly 1 argument")
            }

            proc_macro_error::abort!(info.params[0].0, "invalid setter arguments")
        }
    }
}