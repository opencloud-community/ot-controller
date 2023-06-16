// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Allows to create one or more typed ids
///
/// Defines the type and implements a variety of traits for it to be usable with diesel.
#[macro_export]
macro_rules! diesel_newtype {
    // This macro is called with feature_gated: in front
    // ```rust
    // diesel_newtype! {
    //     feature_gated: // <-- This token makes the macro enter this branch
    //     MyStringType(String) => diesel::sql_types::Text
    // }
    // ```
    //
    // Turns into `diesel_newtype!(@impl__ true  MyStringType(String) => diesel::sql_types::Text;);`
    (
        feature_gated:
        $(
            $(#[$meta:meta])*
            $name:ident($to_wrap:ty) => $sql_type:ty
            $(, $kustos_prefix:literal)?),+
    ) => {
        $crate::diesel_newtype!(
            @impl__
            $(
                true
                $(#[$meta])*
                $name($to_wrap) => $sql_type
                $(, $kustos_prefix)?;
            )*
        );
    };


    // This macro is called when there's no `feature_gated:` in front
    //
    // Example input:
    // ```rust
    // diesel_newtype! {
    //     MyStringType(String) => diesel::sql_types::Text
    // }
    // ```
    //
    // Turns into `diesel_newtype!(@impl__ false  MyStringType(String) => diesel::sql_types::Text;);`
    (
        $(
            $(#[$meta:meta])*
            $name:ident($to_wrap:ty) => $sql_type:ty
            $(, $kustos_prefix:literal)?),+
    ) => {
        $crate::diesel_newtype!(
            @impl__
            $(
                false
                $(#[$meta])*
                $name($to_wrap) => $sql_type
                $(, $kustos_prefix)?;
            )*
        );
    };

    // This macro generate the actual code, its called from one of the arms above
    // Its marked as internal branch impl with the @impl__ prefix
    // Each iteration takes an ident 'do_feature_gate' which is either `true` or `false`, it is used to decide wether
    // or not to apply a #[cfg(feature = "...")] in front of some items
    (
        @impl__
        $(
            $do_feature_gate:ident
            $(#[$meta:meta])*
            $name:ident($to_wrap:ty) => $sql_type:ty
            $(, $kustos_prefix:literal)?;
        )+
    ) => {
        $(
            pub use __newtype_impl::$name;
        )+

        mod __newtype_impl {
            use std::fmt;

            $(

            $crate::maybe_put_meta_behind_feature! {
                feature_gate_it = $do_feature_gate;

                feature = "diesel";
                meta =
                    #[derive(::diesel::AsExpression, ::diesel::FromSqlRow)],
                    #[diesel(sql_type = $sql_type)];

                feature = "serde";
                meta =
                    #[derive(::serde::Serialize, ::serde::Deserialize)];

                item:

                #[derive(
                    Debug,
                    Clone,
                    PartialEq,
                    Eq,
                    PartialOrd,
                    Ord,
                    Hash,
                )]
                $(#[$meta])*
                #[allow(missing_docs)]
                pub struct $name($to_wrap);
            }

            impl $name {
                /// Wrap a value into this type.
                pub const fn from(inner: $to_wrap) -> Self {
                    Self (inner)
                }

                /// Get a reference to the inner type.
                pub fn inner(&self) -> &$to_wrap {
                    &self.0
                }

                /// Destructure this type and extract the inner value.
                pub fn into_inner(self) -> $to_wrap {
                    self.0
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            $crate::maybe_put_behind_feature!{
                feature_gate_it = $do_feature_gate;
                feature = "diesel";

                const _: () = {
                    use diesel::backend::{Backend, RawValue};
                    use diesel::deserialize::{self, FromSql};
                    use diesel::serialize::{self, Output, ToSql};

                    impl<DB> ToSql<$sql_type, DB> for $name
                    where
                        DB: Backend,
                        $to_wrap: ToSql<$sql_type, DB>,
                    {
                        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
                            <$to_wrap as ToSql<$sql_type, DB>>::to_sql(&self.0, out)
                        }
                    }

                    impl<DB> FromSql<$sql_type, DB> for $name
                    where
                        DB: Backend,
                        $to_wrap: FromSql<$sql_type, DB>,
                    {
                        fn from_sql(bytes: RawValue<DB>) -> deserialize::Result<Self> {
                            <$to_wrap as FromSql<$sql_type, DB>>::from_sql(bytes).map(Self)
                        }

                        fn from_nullable_sql(bytes: Option<RawValue<DB>>) -> deserialize::Result<Self> {
                            <$to_wrap as FromSql<$sql_type, DB>>::from_nullable_sql(bytes).map(Self)
                        }
                    }
                };
            }

            $(
            $crate::maybe_put_behind_feature!{
                feature_gate_it = $do_feature_gate;
                feature = "kustos";

                impl ::std::str::FromStr for $name {
                    type Err = kustos::ResourceParseError;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        s.parse().map(Self).map_err(From::from)
                    }
                }


                impl ::kustos::Resource for $name {
                    const PREFIX: &'static str = $kustos_prefix;
                }
            }
            )?

            )+
        }
    };
}
