// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_fields_helper::{get_fields, get_format_macro_call, Fields};
use quote::quote;

const ATTRIBUTE_NAME: &str = "to_redis_args";

pub(crate) fn to_redis_args(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_to_redis_args(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_to_redis_args(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let conversion = get_to_redis_args_conversion(&ast.attrs)?;

    match conversion {
        ToRedisArgsConversion::Serde => impl_to_redis_args_serde(&ast),
        ToRedisArgsConversion::DirectFormat => impl_to_redis_args_fmt(&ast, "{}"),
        ToRedisArgsConversion::Format(fmt) => impl_to_redis_args_fmt(&ast, fmt.as_str()),
        ToRedisArgsConversion::Display => impl_to_redis_args_display(&ast),
    }
}

fn get_to_redis_args_conversion(
    attrs: &[syn::Attribute],
) -> Result<ToRedisArgsConversion, syn::Error> {
    let mut found_attr = None;
    for attr in attrs {
        if let Some(segment) = attr.path().segments.iter().next() {
            if segment.ident == ATTRIBUTE_NAME {
                if found_attr.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Multiple #[{ATTRIBUTE_NAME}(...)] found",),
                    ));
                } else {
                    found_attr = Some(attr);
                }
            }
        }
    }

    if let Some(attr) = found_attr {
        return parse_to_redis_args_attribute_meta(attr.meta.clone());
    }

    Err(syn::Error::new(
        Span::call_site(),
        format!("Attribute #[{ATTRIBUTE_NAME}(...)] missing for #[derive(ToRedisArgs)]"),
    ))
}

#[derive(Debug, PartialEq, Eq)]
enum ToRedisArgsConversion {
    Serde,
    DirectFormat,
    Format(String),
    Display,
}

fn parse_to_redis_args_attribute_meta(
    meta: syn::Meta,
) -> Result<ToRedisArgsConversion, syn::Error> {
    fn create_generic_error_message() -> syn::Error {
        syn::Error::new(Span::call_site(), format!("Attribute #[{ATTRIBUTE_NAME}(...)] requires either `fmt`, `fmt = \"...\"`, `serde`, or `Display`  parameter"))
    }

    match meta {
        syn::Meta::List(syn::MetaList {
            path: _,
            delimiter,
            tokens,
        }) => {
            if !matches!(delimiter, syn::MacroDelimiter::Paren(_)) {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Attribute #[{ATTRIBUTE_NAME}(...)] must have parentheses: '('"),
                ));
            }

            let mut tokens = tokens.into_iter();
            let conversion = match tokens.next() {
                Some(proc_macro2::TokenTree::Ident(ident)) if ident == "fmt" => {
                    ToRedisArgsConversion::DirectFormat
                }
                Some(proc_macro2::TokenTree::Ident(ident)) if ident == "serde" => {
                    ToRedisArgsConversion::Serde
                }
                Some(proc_macro2::TokenTree::Ident(ident)) if ident == "Display" => {
                    ToRedisArgsConversion::Display
                }
                _ => return Err(create_generic_error_message()),
            };

            match tokens.next() {
                Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                    if conversion == ToRedisArgsConversion::Serde {
                        return Err(syn::Error::new(Span::call_site(),
                            format!("Attribute #[{ATTRIBUTE_NAME}(serde)] does not allow additional parameters")
                        ));
                    }

                    let tokens = proc_macro2::TokenStream::from_iter(tokens);
                    let s = syn::parse2::<syn::LitStr>(tokens)?;
                    Ok(ToRedisArgsConversion::Format(s.value()))
                }
                Some(_) => Err(syn::Error::new(Span::call_site(), "Unexpected token")),
                None => Ok(conversion),
            }
        }
        syn::Meta::Path(_) => Err(create_generic_error_message()),
        syn::Meta::NameValue(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("Attribute #[{ATTRIBUTE_NAME}(...)] does not allow assignments inside the parentheses"),
        )),
    }
}

fn impl_to_redis_args_fmt(input: &syn::DeriveInput, fmt: &str) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;
    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let fields = get_fields(fields);

            if matches!(fields, Fields::Empty) {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("The #[{ATTRIBUTE_NAME}] attribute can only be attached to structs with fields."),
                ));
            }

            let format_macro_call = get_format_macro_call(ATTRIBUTE_NAME, fmt, &fields)?;

            let expanded = quote! {
                impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
                    fn write_redis_args<W>(&self, out: &mut W)
                    where
                        W: ?Sized + ::redis_args::__exports::redis::RedisWrite,
                    {
                        out.write_arg(#format_macro_call.as_bytes())
                    }
                }
            };
            Ok(TokenStream::from(expanded))
        }
        syn::Data::Enum(_) | syn::Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(fmt)] can only be used with structs"),
        )),
    }
}

fn impl_to_redis_args_serde(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis_args::__exports::redis::RedisWrite
            {
                let json_val = ::redis_args::__exports::serde_json::to_vec(self).expect("Failed to serialize");
                out.write_arg(&json_val);
            }
        }
    };
    Ok(TokenStream::from(expanded))
}

fn impl_to_redis_args_display(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis_args::__exports::redis::RedisWrite
            {
                out.write_arg_fmt(&self);
            }
        }
    };
    Ok(TokenStream::from(expanded))
}
