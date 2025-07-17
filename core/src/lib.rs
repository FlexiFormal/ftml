#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

mod keys;
pub use keys::{FTMLKey, NUM_RULES, PREFIX};
pub mod utils {
    mod shared_arc;
    pub use shared_arc::SharedArc;
}
pub mod domain;
pub(crate) mod __private {
    pub trait Sealed {}
}

// ----------------------------------------------------------------------------------

macro_rules! serde_impl {
    (@i_count ) => { 0 };
    (@i_count $r:ident $($rs:tt)* ) => { 1 + crate::serde_impl!(@i_count $($rs)*) };
    (@count $($r:ident)*) => { crate::serde_impl!(@i_count $($r)*)};

    (@caseI $f:ident) => {
        Self::$f
    };
    (@caseII $ser:ident $s:ident $idx:literal $f:ident) => {
        $ser.serialize_unit_variant(stringify!($s),$idx,stringify!($f))
    };
    (@caseIII $ser:ident $s:ident $idx:literal $f:ident) => {
        {$ser.unit_variant()?;Ok(Self::$f)}
    };

    (@caseI $f:ident($nt:ident)) => {
        Self::$f($nt)
    };
    (@caseII $ser:ident $s:ident $idx:literal $f:ident($nt:ident)) => {
        $ser.serialize_newtype_variant(stringify!($s),$idx,stringify!($f),$nt)
    };
    (@caseIII $ser:ident $s:ident $idx:literal $f:ident($nt:ident)) => {
        $ser.newtype_variant().map($s::$f)
    };

    (@caseI $f:ident{ $($n:ident),* }) => {
        Self::$f{$($n),*}
    };
    (@caseII $ser:ident $s:ident $idx:literal $f:ident{ $($n:ident),* }) => {{
        let mut s = $ser.serialize_struct_variant(stringify!($s),$idx,stringify!($f),
            crate::serde_impl!(@count $($n)*)
        )?;
        $(
            s.serialize_field(stringify!($n),$n)?;
        )*
        s.end()
    }};
    (@caseIII $ser:ident $s:ident $idx:literal $f:ident{ $($n:ident),* }) => {{
        struct SVisitor;

        #[derive(serde::Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field { $($n),* }
        impl<'de> serde::de::Visitor<'de> for SVisitor {
            type Value = $s<Unchecked>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(stringify!($f))
            }
            #[allow(unused_assignments)]
            fn visit_seq<V>(self, mut seq: V) -> Result<$s<Unchecked>, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let mut count = 0;
                $(
                    let $n = seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(count, &self))?;
                    count += 1;
                )*
                Ok($s::$f{ $($n),* })
            }
            fn visit_map<V>(self, mut map: V) -> Result<$s<Unchecked>, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                $(
                    let mut $n = None;
                )*
                while let Some(key) = map.next_key()? {
                    match key {
                        $(
                            Field::$n => {
                                if $n.is_some() {
                                    return Err(serde::de::Error::duplicate_field(stringify!($n)));
                                }
                                $n = Some(map.next_value()?);
                            }
                        )*
                    }
                }
                $(
                    let $n = $n.ok_or_else(|| serde::de::Error::missing_field(stringify!($n)))?;
                )*
                Ok($s::$f { $($n),* })
            }
        }

        $ser.struct_variant(&[ $(stringify!($n)),* ],SVisitor)
    }};

    ($(mod $m:ident = )? struct $s:ident[$($f:ident),+] ) => {crate::serde_impl!{$(mod $m = )? $s : slf
        s => {
            let mut s = s.serialize_struct(
                stringify!($s),
                crate::serde_impl!(@count $($f)*)
            )?;
            $(
                s.serialize_field(stringify!($f),&slf.$f)?;
            )*
            s.end()
        }
        d => {
            #[derive(serde::Deserialize)]
            #[allow(non_camel_case_types)]
            enum Field { $($f),* }
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = $s<Unchecked>;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str(stringify!($s))
                }
                #[allow(unused_assignments)]
                fn visit_seq<V>(self, mut seq: V) -> Result<$s<Unchecked>, V::Error>
                where
                    V: serde::de::SeqAccess<'de>,
                {
                    let mut count = 0;
                    $(
                        let $f = seq.next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(count, &self))?;
                        count += 1;
                    )*
                    Ok($s{ $($f),* })
                }
                fn visit_map<V>(self, mut map: V) -> Result<$s<Unchecked>, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
                {
                    $(
                        let mut $f = None;
                    )*
                    while let Some(key) = map.next_key()? {
                        match key {
                            $(
                                Field::$f => {
                                    if $f.is_some() {
                                        return Err(serde::de::Error::duplicate_field(stringify!($f)));
                                    }
                                    $f = Some(map.next_value()?);
                                }
                            )*
                        }
                    }
                    $(
                        let $f = $f.ok_or_else(|| serde::de::Error::missing_field(stringify!($f)))?;
                    )*
                    Ok($s { $($f),* })
                }
            }
            d.deserialize_struct(stringify!($s),&[$(stringify!($f)),*],Visitor)

        }
    }};

    ($(mod $m:ident = )? enum $s:ident{ $( {$idx:literal = $f:ident $($spec:tt)*} )+ } ) => {
        crate::serde_impl!{$(mod $m = )? $s : slf
            ser => {
                match slf {
                    $(
                        crate::serde_impl!(@caseI $f $($spec)*) =>
                        crate::serde_impl!{@caseII ser $s $idx $f $($spec)* }
                    ),*
                }
            }
            de => {
                #[derive(serde::Deserialize)]
                enum Fields {
                    $(
                        $f = $idx
                    ),*
                }
                struct Visitor;
                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $s;//<Unchecked>;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(stringify!($s))
                    }
                    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                    where
                        A: EnumAccess<'de>,
                    {
                        let (v,var) = data.variant()?;
                        match v {
                            $(
                                Fields::$f => crate::serde_impl!{@caseIII var $s $idx $f $($spec)* },
                            )*
                            //s => Err(A::Error::unknown_variant(s, &[ $(stringify!($f)),* ]))
                        }

                    }

                }

                de.deserialize_enum(
                    stringify!($s),
                    &[ $( stringify!($f) ),* ],
                    Visitor
                )
            }
        }
    };

    ($s:ident : $slf:ident $ser:ident => {$($ser_impl:tt)*} $de:ident => {$($de_impl:tt)*}) => {
        crate::serde_impl!{mod serde_impl = $s : $slf $ser => {$($ser_impl)*} $de => {$($de_impl)*}}
    };

    (mod $m:ident = $s:ident : $slf:ident $ser:ident => {$($ser_impl:tt)*} $de:ident => {$($de_impl:tt)*}) => {
        #[cfg(feature="serde")]#[allow(unused_imports)]
        mod $m {
            use super::$s;
            //use crate::Unchecked;
            use ::serde::ser::{SerializeStruct,SerializeStructVariant};
            use ::serde::de::{EnumAccess,VariantAccess,Error};
            impl/*<State:$crate::CheckingState>*/ ::serde::Serialize for $s/*<State>*/ {
                fn serialize<S: ::serde::Serializer>(&self,$ser:S) -> Result<S::Ok,S::Error> {
                    let $slf = self;
                    $($ser_impl)*
                }
            }
            impl<'de> ::serde::Deserialize<'de> for $s/*<Unchecked>*/ {
                fn deserialize<D: ::serde::de::Deserializer<'de>>($de: D) -> Result<Self, D::Error> {
                    $($de_impl)*
                }
            }
        }
    };
}
pub(crate) use serde_impl;
