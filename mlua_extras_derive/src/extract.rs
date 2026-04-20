use darling::FromMeta;
use deluxe::{ParseAttributes, ParseMetaItem};
use proc_macro2::{Literal, TokenStream};
use quote::ToTokens;
use syn::{
    spanned::Spanned, Attribute, Expr, ExprLit, FnArg, ImplItemFn, Lit, Meta, MetaNameValue, Pat,
    PatIdent, ReturnType,
};

pub fn doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            if let Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect();

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

fn docs(attrs: Vec<Attribute>) -> darling::Result<Option<String>> {
    let docs = attrs
        .into_iter()
        .filter_map(|Attribute { meta, .. }| {
            if let Meta::NameValue(MetaNameValue { path, value, .. }) = &meta {
                if path.is_ident("doc") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit), ..
                    }) = &value
                    {
                        return Some(lit.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>();

    Ok((!docs.is_empty()).then_some(docs.join("\n")))
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(field), forward_attrs(doc))]
pub struct UserDataField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    #[allow(dead_code)]
    #[darling(with = "docs")]
    pub attrs: Option<String>,

    #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub readonly: bool,
    #[darling(default)]
    pub writeonly: bool,
    #[darling(default)]
    pub rename: Option<Index>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(field), forward_attrs(doc))]
pub struct UserDataEnumField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    
    #[darling(default, skip)]
    pub variant: TokenStream,
    #[darling(default, skip)]
    pub accessor: TokenStream,

    #[allow(dead_code)]
    #[darling(with = "docs")]
    pub attrs: Option<String>,

    #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub readonly: bool,
    #[darling(default)]
    pub writeonly: bool,
    #[darling(default)]
    pub rename: Option<Index>,
}


#[derive(Debug)]
pub enum PassBy {
    Ref,
    RefMut,
    Value,
}
impl PassBy {
    fn from_fn_arg(value: Option<&FnArg>) -> Option<Self> {
        match value {
            Some(FnArg::Receiver(recv)) => {
                if recv.reference.is_some() {
                    if recv.mutability.is_some() {
                        Some(PassBy::RefMut)
                    } else {
                        Some(PassBy::Ref)
                    }
                } else {
                    Some(PassBy::Value)
                }
            }
            Some(FnArg::Typed(typed)) => {
                if let Pat::Ident(PatIdent {
                    ident,
                    by_ref,
                    mutability,
                    ..
                }) = &*typed.pat
                {
                    if ident == "self" {
                        if by_ref.is_some() {
                            if mutability.is_some() {
                                Some(PassBy::RefMut)
                            } else {
                                Some(PassBy::Ref)
                            }
                        } else {
                            Some(PassBy::Value)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum MethodKind {
    Regular,
    Meta,
}
impl MethodKind {
    pub fn is_meta(&self) -> bool {
        match self {
            Self::Regular => false,
            Self::Meta => true,
        }
    }
}

#[derive(Debug, ParseAttributes)]
#[deluxe(attributes(metamethod))]
struct MetaMethod(IdentOrCustom);

#[derive(Default, Debug, ParseAttributes)]
#[deluxe(default, attributes(method))]
struct Method {
    rename: Option<String>,
}

pub fn is_method_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("method")
}

pub fn is_metamethod_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("metamethod")
}

#[derive(Debug)]
pub struct UserDataMethod {
    #[allow(dead_code)]
    pub doc: Option<String>,
    pub r#async: bool,
    pub name: syn::Ident,
    pub lua_name: TokenStream,
    pub instance: Option<PassBy>,
    pub lua: bool,
    pub params: Vec<(syn::Ident, syn::Type)>,
    pub fallible: bool,
    pub returnable: bool,
    pub kind: MethodKind,
}
impl UserDataMethod {
    pub fn from_imp_fn(method: &ImplItemFn) -> Option<Self> {
        let method_attr = method
            .attrs
            .iter()
            .find(|a| is_method_attr(a))
            .map(|a| deluxe::parse_attributes::<_, Method>(a).unwrap_or_default());
        let metamethod_attr = match method
            .attrs
            .iter()
            .find(|a| is_metamethod_attr(a))
        {
            Some(a) => match deluxe::parse_attributes::<_, MetaMethod>(a) {
                Ok(v) => Some(v),
                Err(err) => proc_macro_error::abort!(method.span(), "{}", err),
            },
            None => None,
        };

        if method_attr.is_some() && metamethod_attr.is_some() {
            return None;
        }

        let fn_name = method.sig.ident.clone();
        let is_async = method.sig.asyncness.is_some();
        let instance = PassBy::from_fn_arg(method.sig.inputs.first());
        let doc = doc_comment(&method.attrs);

        let (lua_name, kind) = if let Some(Method { rename }) = method_attr {
            let name = rename.unwrap_or_else(|| fn_name.to_string());
            (quote!(#name), MethodKind::Regular)
        } else if let Some(MetaMethod(target)) = metamethod_attr {
            if (target.is_ident() && target == "Index") || (target == "__index") {
                let replace = "__usr_index";
                (quote!(#replace), MethodKind::Meta)
            } else if (target.is_ident() && target == "NewIndex") || (target == "__newindex") {
                let replace = "__usr_newindex";
                (quote!(#replace), MethodKind::Meta)
            } else {
                (quote!(#target), MethodKind::Meta)
            }
        } else {
            return None;
        };

        // Check for async metamethod conflict
        if is_async {
            if kind.is_meta() {
                proc_macro_error::abort!(
                    method.sig.asyncness,
                    "async metamethods are not supported by mlua"
                );
            }
        }

        // Collect non-self parameters
        let mut params_iter = method.sig.inputs.iter().peekable();

        // Skip the instance arg if present
        match &instance {
            Some(_) => {
                params_iter.next();
            } // skip self
            None => {
                // Check if the first arg is a typed `self` pattern
                if let Some(FnArg::Typed(pat_type)) = method.sig.inputs.first() {
                    if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                        if ident == "self" {
                            params_iter.next();
                        }
                    }
                }
            }
        }

        // Detect lua parameter (first param named "lua")
        let mut has_lua: bool = false;
        if let Some(FnArg::Typed(pat_type)) = params_iter.peek() {
            if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                if ident == "lua" {
                    has_lua = true;
                    params_iter.next();
                }
            }
        }

        let params: Vec<_> = params_iter
            .filter_map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                        return Some((ident.clone(), (*pat_type.ty).clone()));
                    }
                }
                None
            })
            .collect();

        // Analyze return type
        let (is_fallible, has_return) = match &method.sig.output {
            ReturnType::Default => (false, false),
            ReturnType::Type(_, ty) => {
                let result_type = match &**ty {
                    syn::Type::Path(path) => {
                        if let Some(last) = path.path.segments.last() {
                            last.ident == "Result"
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if result_type {
                    (true, true)
                } else {
                    // Check if it's () type
                    let is_unit = matches!(&**ty, syn::Type::Tuple(t) if t.elems.is_empty());
                    (false, !is_unit)
                }
            }
        };

        Some(Self {
            doc,
            r#async: is_async,
            name: fn_name,
            instance,
            lua: has_lua,
            lua_name,
            params,
            fallible: is_fallible,
            returnable: has_return,
            kind,
        })
    }
}

#[derive(Debug, Clone)]
pub enum IdentOrCustom {
    Ident(syn::Ident),
    Custom(String),
}
impl IdentOrCustom {
    pub fn is_ident(&self) -> bool {
        match self {
            Self::Ident(_) => true,
            _ => false
        }
    }
}
impl PartialEq<&str> for IdentOrCustom {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::Custom(v) => v == *other,
            Self::Ident(v) => v == *other,
        }
    }
}
impl ParseMetaItem for IdentOrCustom {
    fn parse_meta_item(input: syn::parse::ParseStream, _mode: deluxe::ParseMode) -> deluxe::Result<Self> {
        if input.peek(syn::LitStr) {
            let lit: syn::LitStr = input.parse()?;
            Ok(IdentOrCustom::Custom(lit.value()))
        } else {
            let ident: syn::Ident = input.parse()?;
            Ok(IdentOrCustom::Ident(ident))
        }
    }
}
impl ToTokens for IdentOrCustom {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(ident) => quote!(mlua_extras::mlua::MetaMethod::#ident).to_tokens(tokens),
            Self::Custom(v) => v.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Index {
    Int(isize),
    Str(String),
}
impl Index {
    pub fn is_str(&self) -> bool {
        match self {
            Self::Str(_) => true,
            _ => false,
        }
    }
}
impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Str(s) => write!(f, "{s}"),
        }
    }
}

impl FromMeta for Index {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        if let syn::Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit { lit, .. }),
            ..
        }) = item
        {
            match lit {
                Lit::Str(s) => return Ok(Self::Str(s.value())),
                Lit::Int(i) => return Ok(Self::Int(i.base10_parse()?)),
                _ => (),
            }
        }
        Err(darling::Error::custom("Expected string or integer literal"))
    }
}
impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Int(i) => Literal::isize_unsuffixed(*i).to_tokens(tokens),
            Self::Str(s) => s.to_tokens(tokens),
        }
    }
}
