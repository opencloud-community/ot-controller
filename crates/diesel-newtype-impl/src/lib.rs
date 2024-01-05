// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::Ident;

#[proc_macro_derive(DieselNewtype, attributes(diesel))]
pub fn derive_diesel_newtype(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_derive_diesel_newtype(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_derive_diesel_newtype(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let err_msg =
        "#[derive(DieselNewtype)] can only be used on anonymous structs with a single field.";

    let reexports = match crate_name("opentalk-diesel-newtype")
        .map_err(|_| syn::Error::new(Span::call_site(), err_msg))?
    {
        FoundCrate::Itself => {
            quote!(crate::__exports)
        }
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident::__exports)
        }
    };

    let syn::Data::Struct(data_struct) = ast.data else {
        return Err(syn::Error::new(Span::call_site(), err_msg));
    };

    let syn::Fields::Unnamed(fields) = data_struct.fields else {
        return Err(syn::Error::new(Span::call_site(), err_msg));
    };

    if fields.unnamed.len() != 1 {
        return Err(syn::Error::new(Span::call_site(), err_msg));
    }

    let field = fields.unnamed.iter().next().expect("unnamed fields");

    let ident = ast.ident;
    let inner_type = field.ty.clone();

    let sql_type = get_sql_type_from_attributes(&ast.attrs)?;

    let expanded = quote! {
        impl DieselNewtype<#sql_type> for #ident {}

        impl<DB> #reexports::diesel::serialize::ToSql<#sql_type, DB> for #ident
        where
            DB: #reexports::diesel::backend::Backend,
            #inner_type: #reexports::diesel::serialize::ToSql<#sql_type, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut #reexports::diesel::serialize::Output<'b, '_, DB>,
            )
                -> #reexports::diesel::serialize::Result {
                <#inner_type as #reexports::diesel::serialize::ToSql<#sql_type, DB>>::to_sql(&self.0, out)
            }
        }

        impl<DB> #reexports::diesel::deserialize::FromSql<#sql_type, DB> for #ident
        where
            DB: #reexports::diesel::backend::Backend,
            #inner_type: #reexports::diesel::deserialize::FromSql<#sql_type, DB>,
        {
            fn from_sql(raw: #reexports::diesel::backend::RawValue<DB>)
                -> #reexports::diesel::deserialize::Result<Self>
            {
                <#inner_type as #reexports::diesel::deserialize::FromSql<#sql_type, DB>>::from_sql(raw).map(Self)
            }

            fn from_nullable_sql(
                raw: ::std::option::Option<::diesel::backend::RawValue<DB>>)
                -> #reexports::diesel::deserialize::Result<Self>
            {
                 <#inner_type as #reexports::diesel::deserialize::FromSql<#sql_type, DB>>::from_nullable_sql(raw).map(Self)
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

fn get_sql_type_from_attributes(attrs: &[syn::Attribute]) -> Result<syn::Type, syn::Error> {
    let mut found_attr = None;
    for attr in attrs {
        if let Some(segment) = attr.path.segments.iter().next() {
            if segment.ident == "diesel" {
                if found_attr.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Multiple #[diesel(...)] found",
                    ));
                }

                found_attr = Some(attr);
            }
        }
    }

    if let Some(attr) = found_attr {
        return parse_attribute_parameters(attr.tokens.clone());
    }

    Err(syn::Error::new(
        Span::call_site(),
        "Attribute #[diesel(...)] missing for #[derive(DieselNewtype)]",
    ))
}

fn parse_attribute_parameters(
    parameters: proc_macro2::TokenStream,
) -> Result<syn::Type, syn::Error> {
    match parameters.into_iter().next() {
        Some(proc_macro2::TokenTree::Group(group)) => {
            if group.delimiter() != proc_macro2::Delimiter::Parenthesis {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Attribute #[diesel(...)] must have braces: '('",
                ));
            }
            let mut tokens = group.stream().into_iter();

            parse_sql_type(&mut tokens)
        }
        _ => Err(syn::Error::new(Span::call_site(), "Unexpected token")),
    }
}

fn parse_sql_type<T>(tokens: &mut T) -> Result<syn::Type, syn::Error>
where
    T: Iterator<Item = proc_macro2::TokenTree>,
{
    match tokens.next() {
        Some(proc_macro2::TokenTree::Ident(ident)) if ident == "sql_type" => {}
        None => {
            return Err(syn::Error::new(
                Span::call_site(),
                "No `sql_type = ...` parameter found for #[diesel(...)]",
            ))
        }
        _ => {
            return Err(syn::Error::new(Span::call_site(), "Unexpected token"));
        }
    };

    match tokens.next() {
        Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '=' => {}
        Some(_) => {
            return Err(syn::Error::new(Span::call_site(), "Unexpected token"));
        }
        None => {
            return Err(syn::Error::new(
                Span::call_site(),
                "`sql_type` parameter requires `= ...` with value",
            ));
        }
    }

    let tokens = proc_macro2::TokenStream::from_iter(tokens);
    syn::parse2::<syn::Type>(tokens)
}
