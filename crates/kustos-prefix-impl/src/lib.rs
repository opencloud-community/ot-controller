// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

#[proc_macro_derive(KustosPrefix, attributes(kustos_prefix))]
pub fn derive_kustos_prefix(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_derive_kustos_prefix(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_derive_kustos_prefix(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let kustos_prefix = crate_name("opentalk-kustos-prefix").map_err(|_| {
        syn::Error::new(
            Span::call_site(),
            "Couldn't find opentalk-kustos-prefix crate",
        )
    })?;

    let reexports = match kustos_prefix {
        FoundCrate::Itself => quote!(crate::__exports),
        FoundCrate::Name(name) => {
            let ident = proc_macro2::Ident::new(&name, Span::call_site());
            quote!(#ident::__exports)
        }
    };

    let msg = "#[derive(KustosPrefix)] can only be used on anonymous structs with a single field.";

    let syn::Data::Struct(data_struct) = ast.data else {
        return Err(syn::Error::new(Span::call_site(), msg));
    };

    let syn::Fields::Unnamed(fields) = data_struct.fields else {
        return Err(syn::Error::new(Span::call_site(), msg));
    };

    if fields.unnamed.len() != 1 {
        return Err(syn::Error::new(Span::call_site(), msg));
    }

    let ident = ast.ident;
    let kustos_prefix = get_prefix_from_attributes(&ast.attrs)?;

    let expanded = quote! {
        impl #reexports::kustos_shared::resource::Resource for #ident {
            const PREFIX: &'static str = #kustos_prefix;
        }
    };

    Ok(TokenStream::from(expanded))
}

fn get_prefix_from_attributes(attrs: &[syn::Attribute]) -> Result<syn::LitStr, syn::Error> {
    let mut found_attr = None;
    for attr in attrs {
        if let Some(segment) = attr.path.segments.iter().next() {
            if segment.ident == "kustos_prefix" {
                if found_attr.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Multiple #[kustos_prefix(...)] found",
                    ));
                } else {
                    found_attr = Some(attr);
                }
            }
        }
    }

    if let Some(attr) = found_attr {
        return parse_attribute(attr.tokens.clone());
    }

    Err(syn::Error::new(
        Span::call_site(),
        "Attribute #[kustos_prefix(...)] missing for #[derive(KustosPrefix)]",
    ))
}

fn parse_attribute(parameters: proc_macro2::TokenStream) -> Result<syn::LitStr, syn::Error> {
    let msg = "Attribute #[kustos_prefix(...)] must have braces: '('";

    match parameters.into_iter().next() {
        Some(proc_macro2::TokenTree::Group(group)) => {
            if group.delimiter() != proc_macro2::Delimiter::Parenthesis {
                return Err(syn::Error::new(Span::call_site(), msg));
            }

            let tokens = group.stream();

            syn::parse2::<syn::LitStr>(tokens)
        }
        _ => Err(syn::Error::new(
            Span::call_site(),
            "Attribute #[kustos_prefix(...)] must have a string parameter",
        )),
    }
}
