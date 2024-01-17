// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::request::ATTRIBUTE_NAME;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Iter, Expr, ExprPath, Field};

#[derive(Debug)]
pub struct AttributedField {
    pub name: TokenStream,
    pub ty: syn::Type,
}

pub fn get_attributed_field(
    fields: &syn::Fields,
    field_attribute_name: &str,
) -> Result<Option<AttributedField>, syn::Error> {
    let field = match &fields {
        syn::Fields::Named(fields) => {
            find_field_attribute(field_attribute_name, fields.named.iter())?
        }
        syn::Fields::Unnamed(fields) => {
            find_field_attribute(field_attribute_name, fields.unnamed.iter())?
        }
        syn::Fields::Unit => None,
    };

    Ok(field)
}

fn find_field_attribute(
    field_attribute_name: &str,
    fields: Iter<Field>,
) -> Result<Option<AttributedField>, syn::Error> {
    let query_fields = fields
        .enumerate()
        .filter_map(|(index, field)| {
            let is_query = field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident(ATTRIBUTE_NAME))
                .filter(|attr| {
                    matches!(attr.parse_args::<Expr>(), Ok(Expr::Path(ExprPath { path, .. })) if path.is_ident(field_attribute_name))
                }).count() > 0;

                if is_query {
                    Some(AttributedField {
                        name: field.ident.as_ref().map(|ident| quote!( #ident ) ).unwrap_or_else(|| {
                            let i = syn::Index::from(index);
                            quote!( #i )
                        }),
                        ty: field.ty.clone(),
                    })
                } else {
                    None
                }
        })
        .collect::<Vec<_>>();

    if query_fields.len() > 1 {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("Only one field can be marked as #[{ATTRIBUTE_NAME}(query)]"),
        ));
    }

    Ok(query_fields.into_iter().next())
}
