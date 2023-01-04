// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

pub(crate) fn from_redis_value(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_from_redis_value(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_from_redis_value(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let conversion = get_from_redis_value_conversion(&ast.attrs)?;

    match conversion {
        FromRedisValueConversion::Serde => impl_from_redis_value_serde(&ast),
        FromRedisValueConversion::FromStr => impl_from_redis_value_from_str(&ast),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum FromRedisValueConversion {
    Serde,
    FromStr,
}

fn get_from_redis_value_conversion(
    attrs: &[syn::Attribute],
) -> Result<FromRedisValueConversion, syn::Error> {
    let mut found_attr = None;
    for attr in attrs {
        if let Some(segment) = attr.path().segments.iter().next() {
            if segment.ident == "from_redis_value" {
                if found_attr.is_some() {
                    syn::Error::new(Span::call_site(), "Multiple #[from_redis_value(...)] found");
                } else {
                    found_attr = Some(attr);
                }
            }
        }
    }

    if let Some(attr) = found_attr {
        return parse_from_redis_value_attribute_meta(attr.meta.clone());
    }

    Err(syn::Error::new(
        Span::call_site(),
        "Attribute #[from_redis_value(...)] missing for #[derive(FromRedisValue)]",
    ))
}

fn parse_from_redis_value_attribute_meta(
    meta: syn::Meta,
) -> Result<FromRedisValueConversion, syn::Error> {
    fn create_generic_error_message() -> syn::Error {
        syn::Error::new(
            Span::call_site(),
            "Attribute #[from_redis_value(...)] requires either `FromStr` or `serde` parameter",
        )
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
                    "Attribute #[from_redis_value(...)] must have parentheses: '('",
                ));
            }

            let mut tokens = tokens.into_iter();
            let conversion = match tokens.next() {
                Some(proc_macro2::TokenTree::Ident(ident)) if ident == "FromStr" => {
                    FromRedisValueConversion::FromStr
                }
                Some(proc_macro2::TokenTree::Ident(ident)) if ident == "serde" => {
                    FromRedisValueConversion::Serde
                }
                _ => return Err(create_generic_error_message()),
            };

            if tokens.next().is_some() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Attribute #[from_redis_args(...)] does not allow additional parameters",
                ));
            }

            Ok(conversion)
        }
        syn::Meta::Path(_) => Err(create_generic_error_message()),
        syn::Meta::NameValue(_) => Err(syn::Error::new(
            Span::call_site(),
            "Attribute #[from_redis_value(...)] does not allow assignments inside the parentheses",
        )),
    }
}

fn impl_from_redis_value_serde(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::FromRedisValue for #ident #generics {
            fn from_redis_value(v: &::redis_args::__exports::redis::Value) -> ::redis_args::__exports::redis::RedisResult<Self> {
                match *v {
                    ::redis_args::__exports::redis::Value::Data(ref bytes) => ::redis_args::__exports::serde_json::from_slice(bytes).map_err(|_| {
                        ::redis_args::__exports::redis::RedisError::from(
                            (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data content")
                        )
                    }),
                    _ => ::redis_args::__exports::redis::RedisResult::Err(
                        ::redis_args::__exports::redis::RedisError::from(
                            (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data type")
                        )
                    ),
                }
            }
        }
    };
    Ok(TokenStream::from(expanded))
}

fn impl_from_redis_value_from_str(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::FromRedisValue for #ident #generics {
            fn from_redis_value(v: &::redis_args::__exports::redis::Value) -> ::redis_args::__exports::redis::RedisResult<Self> {
                match *v {
                    ::redis_args::__exports::redis::Value::Data(ref bytes) => {
                        let s = std::str::from_utf8(bytes).map_err(|_| {
                            ::redis_args::__exports::redis::RedisError::from(
                                (::redis_args::__exports::redis::ErrorKind::TypeError, "string is not utf8")
                            )
                        })?;

                        ::std::str::FromStr::from_str(s).map_err(|_| {
                            ::redis_args::__exports::redis::RedisError::from(
                                (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data type")
                            )
                        })
                    },
                    _ => ::redis_args::__exports::redis::RedisResult::Err(
                        ::redis_args::__exports::redis::RedisError::from(
                            (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data type")
                        )
                    ),
                }
            }
        }
    };
    Ok(TokenStream::from(expanded))
}
