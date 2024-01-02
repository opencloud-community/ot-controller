// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use proc_macro2::Span;
use quote::quote;

pub enum Fields {
    Named(Vec<syn::Ident>),
    Unnamed(usize),
    Empty,
}

pub fn get_fields(fields: &syn::Fields) -> Fields {
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

pub fn get_format_macro_call(
    attribute_name: &str,
    fmt: &str,
    fields: &Fields,
) -> Result<proc_macro2::TokenStream, syn::Error> {
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

            Ok(quote! {
                format!(#fmt, #(#field_args),*)
            })
        }
        Fields::Unnamed(count) => {
            // A very naive and probably fragile way to get the number of arguments in the format string.
            // Should work for most cases, but could be improved someday.
            let num_arguments = fmt.replace("{{", "").replace("}}", "").matches('{').count();

            if &num_arguments > count {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Too many arguments in #[{attribute_name}] format string."),
                ));
            }

            let field_args = (0..num_arguments).map(|i| {
                let index = syn::Index::from(i);
                quote! {
                    self.#index
                }
            });

            Ok(quote! {
                format!(#fmt, #(#field_args),*)
            })
        }
        Fields::Empty => Ok(quote! {format!(#fmt)}),
    }
}
