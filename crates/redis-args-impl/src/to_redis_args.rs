// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

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
            if segment.ident == "to_redis_args" {
                if found_attr.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Multiple #[to_redis_args(...)] found",
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
        "Attribute #[to_redis_args(...)] missing for #[derive(ToRedisArgs)]",
    ))
}

#[derive(Debug, PartialEq, Eq)]
enum ToRedisArgsConversion {
    Serde,
    DirectFormat,
    Format(String),
    Display,
}

enum Fields {
    Named(Vec<syn::Ident>),
    Unnamed(usize),
    Empty,
}

fn parse_to_redis_args_attribute_meta(
    meta: syn::Meta,
) -> Result<ToRedisArgsConversion, syn::Error> {
    fn create_generic_error_message() -> syn::Error {
        syn::Error::new(Span::call_site(), "Attribute #[to_redis_args(...)] requires either `fmt`, `fmt = \"...\"`, `serde`, or `Display`  parameter")
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
                    "Attribute #[to_redis_args(...)] must have parentheses: '('",
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
                            "Attribute #[to_redis_args(serde)] does not allow additional parameters"
                        ));
                    }

                    let tokens = proc_macro2::TokenStream::from_iter(tokens);
                    let s = syn::parse2::<syn::LitStr>(tokens).unwrap();
                    Ok(ToRedisArgsConversion::Format(s.value()))
                }
                Some(_) => Err(syn::Error::new(Span::call_site(), "Unexpected token")),
                None => Ok(conversion),
            }
        }
        syn::Meta::Path(_) => Err(create_generic_error_message()),
        syn::Meta::NameValue(_) => Err(syn::Error::new(
            Span::call_site(),
            "Attribute #[from_redis_value(...)] does not allow assignments inside the parentheses",
        )),
    }
}

fn get_fields(fields: &syn::Fields) -> Fields {
    match &fields {
        syn::Fields::Named(fields) => Fields::Named(
            fields
                .named
                .iter()
                .filter_map(|field| field.ident.as_ref().cloned())
                .collect(),
        ),
        syn::Fields::Unnamed(fields) => Fields::Unnamed(fields.unnamed.len()),
        syn::Fields::Unit => Fields::Empty,
    }
}

fn impl_to_redis_args_fmt(input: &syn::DeriveInput, fmt: &str) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;
    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let fields = get_fields(fields);

            match fields {
                Fields::Named(fields) => {
                    let field_args = fields.iter().filter_map(|field| {
                        if fmt.contains(&format!("{{{field}}}")) {
                            Some(quote! {
                                #field=self.#field
                            })
                        } else {
                            None
                        }
                    });
                    let expanded = quote! {
                        impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
                            fn write_redis_args<W>(&self, out: &mut W)
                            where
                                W: ?Sized + ::redis_args::__exports::redis::RedisWrite,
                            {
                                out.write_arg(format!(#fmt, #(#field_args),*).as_bytes())
                            }
                        }
                    };
                    Ok(TokenStream::from(expanded))
                }
                Fields::Unnamed(count) => {
                    // A very naive and probably fragile way to get the number of arguments in the format string.
                    // Should work for most cases, but could be improved someday.
                    let num_arguments =
                        fmt.replace("{{", "").replace("}}", "").matches('{').count();

                    if num_arguments > count {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "Too many arguments in #[redis_args] format string.",
                        ));
                    }

                    let field_args = (0..num_arguments).map(|i| {
                        let index = syn::Index::from(i);
                        quote! {
                            self.#index
                        }
                    });

                    let expanded = quote! {
                        impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
                            fn write_redis_args<W>(&self, out: &mut W)
                            where
                                W: ?Sized + ::redis_args::__exports::redis::RedisWrite,
                            {
                                out.write_arg(format!(#fmt, #(#field_args),*).as_bytes())
                            }
                        }
                    };
                    Ok(TokenStream::from(expanded))
                }
                Fields::Empty => Err(syn::Error::new(
                    Span::call_site(),
                    "The #[redis_args] attribute can only be attached to structs with fields.",
                )),
            }
        }
        syn::Data::Enum(_) | syn::Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "#[to_redis_args(fmt)] can only be used with structs",
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
