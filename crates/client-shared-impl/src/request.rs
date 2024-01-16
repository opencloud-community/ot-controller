// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::iter::Peekable;

use opentalk_proc_macro_fields_helper::{get_fields, get_format_macro_call};
use proc_macro::TokenStream;
use proc_macro2::{token_stream::IntoIter, Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

use crate::query_field::{get_attributed_field, AttributedField};

pub const ATTRIBUTE_NAME: &str = "request";

enum ParameterType {
    Method,
    Response,
    Path,
}

enum Parameter {
    Method(String),
    Response(String),
    Path(String),
}

struct RequestParameters {
    method: Ident,
    response: Ident,
    path: String,
}

pub(crate) fn request(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_to_request(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_to_request(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let request_parameters = get_request_parameters(&ast.attrs)?;

    impl_request(&ast, request_parameters)
}

fn impl_request(
    input: &syn::DeriveInput,
    RequestParameters {
        method,
        response,
        path,
    }: RequestParameters,
) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let path = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let fields = get_fields(fields);

            get_format_macro_call(ATTRIBUTE_NAME, &path, &fields)
        }
        syn::Data::Enum(_) | syn::Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(...)] can only be used with structs"),
        )),
    }?;

    let found_crate = crate_name("opentalk-client-shared").map_err(|_| {
        syn::Error::new(
            Span::call_site(),
            "Dependency `opentalk-client-shared` not found",
        )
    })?;

    let opentalk_client_shared_crate = match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    };

    let query = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => get_attributed_field(fields, "query"),
        syn::Data::Enum(_) | syn::Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(...)] can only be used with structs"),
        )),
    }?;

    let body = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => get_attributed_field(fields, "body"),
        syn::Data::Enum(_) | syn::Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(...)] can only be used with structs"),
        )),
    }?;

    let header = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => get_attributed_field(fields, "header"),
        syn::Data::Enum(_) | syn::Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(...)] can only be used with structs"),
        )),
    }?;

    let (query_fn, query_type) = match query {
        Some(AttributedField { name, ty }) => (
            quote! {
                fn query(&self) -> Option<&Self::Query> {
                    Some(&self.#name)
                }
            },
            quote! { #ty },
        ),
        None => (
            quote! {
                fn query(&self) -> Option<&Self::Query> {
                    None
                }
            },
            quote! { () },
        ),
    };

    let (body_fn, body_type) = match body {
        Some(AttributedField { name, ty }) => (
            quote! {
                fn body(&self) -> Option<&Self::Body> {
                    Some(&self.#name)
                }
            },
            quote! { #ty },
        ),
        None => (
            quote! {
                fn body(&self) -> Option<&Self::Body> {
                    None
                }
            },
            quote! { () },
        ),
    };

    let header_fn = match header {
        Some(AttributedField { name, ty }) => {
            let syn::Type::Path(ref type_path) = ty else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Attribute #[{ATTRIBUTE_NAME}(header)] must be applied to valid type",),
                ));
            };

            if !type_path
                .path
                .segments
                .iter()
                .any(|segment| segment.ident == "HeaderMap")
            {
                return Err(syn::Error::new(Span::call_site(), format!("Attribute #[{ATTRIBUTE_NAME}(header)] must be applied to field of type http::HeaderMap",)));
            }

            quote! {
                fn apply_headers(&self, headers: &mut http::HeaderMap) {
                    headers.extend(self.#name);

                    let _ = headers
                        .entry(http::header::CONTENT_TYPE)
                        .or_insert_with(|| http::HeaderValue::from_static("application/json"));
                }
            }
        }
        None => quote! {
            fn apply_headers(&self, headers: &mut http::HeaderMap) {
                let _ = headers
                    .entry(http::header::CONTENT_TYPE)
                    .or_insert_with(|| http::HeaderValue::from_static("application/json"));
            }
        },
    };

    let expanded = quote! {
        impl #generics #opentalk_client_shared_crate::Request for #ident #generics {
            type Response = #response;
            type Query = #query_type;
            type Body = #body_type;

            const METHOD: http::Method = #opentalk_client_shared_crate::__exports::http::Method::#method;

            fn path(&self) -> std::string::String {
                #path
             }

            #query_fn

            #body_fn

            #header_fn
        }
    };

    Ok(TokenStream::from(expanded))
}

fn get_request_parameters(attrs: &[syn::Attribute]) -> Result<RequestParameters, syn::Error> {
    let mut found_attr = None;

    for attr in attrs {
        if let Some(segment) = attr.path().segments.iter().next() {
            if segment.ident == ATTRIBUTE_NAME {
                if found_attr.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Multiple #[{ATTRIBUTE_NAME}(...)] found"),
                    ));
                } else {
                    found_attr = Some(attr);
                }
            }
        }
    }

    if let Some(attr) = found_attr {
        return parse_request_attribute_meta(attr.meta.clone());
    }

    Err(syn::Error::new(
        Span::call_site(),
        format!("Attribute #[{ATTRIBUTE_NAME}(...)] missing for #[derive(Request)]"),
    ))
}

fn parse_request_attribute_meta(meta: syn::Meta) -> Result<RequestParameters, syn::Error> {
    fn create_generic_error_message() -> syn::Error {
        syn::Error::new(Span::call_site(), format!("Attribute #[{ATTRIBUTE_NAME}(...)] requires at least `method = \"...\"` and `response = \"...\"` parameters"))
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

            let mut tokens = tokens.into_iter().peekable();
            let parameters = std::iter::repeat_with(|| get_next_parameter(&mut tokens))
                .take_while(|param| !matches!(param, Ok(None)))
                .filter_map(Result::transpose)
                .collect::<Result<Vec<_>, syn::Error>>()?;

            let method = extract_parameter(&parameters, "method")?;
            let response = extract_parameter(&parameters, "response")?;
            let path = extract_parameter(&parameters, "path")?;

            Ok(RequestParameters {
                method: Ident::new(&method.to_string(), Span::call_site()),
                response: Ident::new(&response, Span::call_site()),
                path,
            })
        }
        syn::Meta::Path(_) => Err(create_generic_error_message()),
        syn::Meta::NameValue(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("Attribute #[{ATTRIBUTE_NAME}(...)] requires parentheses: `(...)`"),
        )),
    }
}

fn extract_parameter(
    parameters: &[Parameter],
    expected_parameter: &str,
) -> Result<String, syn::Error> {
    let filtered_params = parameters
        .iter()
        .filter_map(|param| match param {
            Parameter::Method(method) if expected_parameter == "method" => Some(method.clone()),
            Parameter::Response(response) if expected_parameter == "response" => {
                Some(response.clone())
            }
            Parameter::Path(path) if expected_parameter == "path" => Some(path.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    match filtered_params.split_first() {
        Some((first, rest)) if rest.is_empty() => Ok((*first).clone()),
        _ => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "Attribute #[{ATTRIBUTE_NAME}(...)] expects exactly one `{}` parameter",
                expected_parameter
            ),
        )),
    }
}

fn get_next_parameter(iter: &mut Peekable<IntoIter>) -> Result<Option<Parameter>, syn::Error> {
    match iter.next() {
        Some(proc_macro2::TokenTree::Ident(ident)) if ident == "method" => {
            handle_parameter(iter, ParameterType::Method).map(Some)
        }
        Some(proc_macro2::TokenTree::Ident(ident)) if ident == "response" => {
            handle_parameter(iter, ParameterType::Response).map(Some)
        }
        Some(proc_macro2::TokenTree::Ident(ident)) if ident == "path" => {
            handle_parameter(iter, ParameterType::Path).map(Some)
        }
        Some(t) => Err(syn::Error::new(
            Span::call_site(),
            format!("unexpected token: {}", t),
        )),
        None => Ok(None),
    }
}

fn handle_parameter(
    iter: &mut Peekable<IntoIter>,
    param_type: ParameterType,
) -> Result<Parameter, syn::Error> {
    match iter.next() {
        Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '=' => {
            let tokens = proc_macro2::TokenStream::from_iter(iter.next());
            let s = syn::parse2::<syn::LitStr>(tokens)?;

            while matches!(iter.peek(), Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == ',')
            {
                iter.next();
            }

            match param_type {
                ParameterType::Response => Ok(Parameter::Response(s.value())),
                ParameterType::Path => Ok(Parameter::Path(s.value())),
                ParameterType::Method => {
                    let method = s.value().to_uppercase();

                    if !matches!(
                        method.as_str(),
                        "GET"
                            | "POST"
                            | "PUT"
                            | "DELETE"
                            | "HEAD"
                            | "OPTIONS"
                            | "CONNECT"
                            | "PATCH"
                            | "TRACE"
                    ) {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!("unexpected method: {}", method),
                        ));
                    }

                    Ok(Parameter::Method(method))
                }
            }
        }
        Some(t) => Err(syn::Error::new(
            Span::call_site(),
            format!("unexpected token: {}", t),
        )),
        None => Err(syn::Error::new(Span::call_site(), "expected '='")),
    }
}
