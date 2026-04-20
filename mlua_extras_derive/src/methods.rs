use proc_macro2::TokenStream;
use syn::{ImplItem, ItemImpl, Type};
use quote::quote;

use crate::extract::{PassBy, UserDataMethod};

pub fn derive(item: ItemImpl) -> TokenStream {
    let self_ty = &item.self_ty;

    let mut user_data = Vec::new();
    let mut cleaned_items = Vec::new();

    for impl_item in &item.items {
        match impl_item {
            ImplItem::Fn(method) => if let Some(udm) = UserDataMethod::from_imp_fn(method) {
                user_data.push(udm);

                let mut cleaned = method.clone();
                cleaned.attrs.retain(|a| !is_method_attr(a) && !is_metamethod_attr(a));
                cleaned_items.push(ImplItem::Fn(cleaned));
            } else {
                cleaned_items.push(impl_item.clone());
            }
            _ => {
                cleaned_items.push(impl_item.clone());
            }
        }
    }

    let registrations: Vec<_> = user_data
        .iter()
        .map(|info| generate_registration(info, self_ty))
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
            fn __auto_add_methods<M: mlua_extras::mlua::UserDataMethods<Self>>(methods: &mut M) {
                #(#registrations)*
            }
        }
    }
}

fn is_method_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("method")
}

fn is_metamethod_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("metamethod")
}

fn generate_registration(info: &UserDataMethod, self_ty: &Type) -> TokenStream {
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
            Some(PassBy::Ref|PassBy::Value) => {
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
            (Some(PassBy::Ref|PassBy::Value), false) => {
                let body = build_call_and_return(quote! { this.#fn_name(#call_args) });
                quote! {
                    methods.add_method(#lua_name, |#lua_ident, this, #params_destructure| {
                        #body
                    });
                }
            }
            (Some(PassBy::Ref|PassBy::Value), true) => {
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