use crate::extraction::attributes::Attributes;
use ftml_uris::DocumentElementUri;

macro_rules! ftml {
    () => {
        "data-ftml-"
    };
    ($l:literal) => {
        concat!(ftml!(), $l)
    };
}
pub const PREFIX: &str = "data-ftml-";
pub const NUM_KEYS: u8 = 2; //119;

pub struct FtmlRuleSet<E: crate::extraction::FtmlExtractor>(
    pub(crate)  [fn(
        &mut E,
        &mut E::Attributes<'_>,
        &mut crate::extraction::KeyList,
        &E::Node,
    ) -> Result<
        (E::Return, Option<crate::extraction::CloseFtmlElement>),
        crate::extraction::FtmlExtractionError,
    >; NUM_KEYS as usize],
);

#[allow(clippy::unnecessary_wraps)]
fn todo<E: crate::extraction::FtmlExtractor>(
    key: FtmlKey,
    ext: &mut E,
    _: &mut E::Attributes<'_>,
    _: &mut crate::extraction::KeyList,
    node: &E::Node,
) -> Result<
    (E::Return, Option<crate::extraction::CloseFtmlElement>),
    crate::extraction::FtmlExtractionError,
> {
    tracing::warn!("Not yet implemented: {key}");
    Ok((
        ext.add_element(crate::extraction::OpenFtmlElement::None, node)?,
        None,
    ))
}

macro_rules! ret {
    ($ext:ident,$node:ident) => {Ok(($ext.add_element(OpenFtmlElement::None,$node)?,None))};
    (@I $ext:ident,$node:ident <- $id:ident{$($b:tt)*} + $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id{$($b)*},$node)?,$r))
    };
    (@I $ext:ident,$node:ident <- $id:ident($($a:expr),*) + $r:expr) => {
        Ok(($ext.add_element(crate::extraction::OpenFtmlElement::$id( $($a),* ),$node)?,$r))
    };
    (@I $ext:ident,$node:ident <- $id:ident + $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id,$node)?,$r))
    };
    ($ext:ident,$node:ident <- $id:ident{$($b:tt)*} + $r:ident) => {
        ret!(@I $ext,$node <- $id{$($b)*} + Some(CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident( $($a:expr),* ) + $r:ident) => {
        ret!(@I $ext,$node <- $id( $($a),*) + Some(crate::extraction::CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident + $r:ident) => {
        ret!(@I $ext,$node <- $id + Some(CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident{$($b:tt)*}) => {
        ret!(@I $ext,$node <- $id{$($b)*} + None)
    };
    ($ext:ident,$node:ident <- $id:ident( $($a:expr),* )) => {
        ret!(@I $ext,$node <- $id( $($a),*) + None)
    };
    ($ext:ident,$node:ident <- $id:ident) => {
        ret!(@I $ext,$node <- $id + None)
    };
}
macro_rules! del {
    ($keys:ident - $($k:ident),* $(,)?) => {
        $keys.0.retain(|e| !matches!(e,$(FtmlKey::$k)|*))
    }
}

macro_rules! do_keys {
    (@LDOC) => {""};
    (@RDOC) => {""};
    (@ADOC) => {""};

    (@LDOC -!( $not:literal ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC -!( $not:literal ) $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC -!( $not:literal ) $($rest:tt)*) => {
        concat!(
            "\n\n<div class=\"warning\">\n\n*Not allowed ",$not,"*\n\n</div>\n\n",
            do_keys!(@ADOC $($rest)*)
        )
    };

    (@LDOC -( $($req:ident),+ ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC -( $($req:ident),+ ) $($rest:tt)*) => {
        concat!(
            "Attribute of: " $(,
                "[" ,stringify!($req),"](FtmlKey::",stringify!($req), "), "
            )*,
            do_keys!(@ADOC $($rest)*)
        )
    };
    (@ADOC -( $($req:ident),+ ) $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};

    (@LDOC +( $($other:ident),* ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC +( $($other:ident),* ) $($rest:tt)*) => {
        concat!(
            "Additional attributes: "
            $(, "[",stringify!($other),"](FtmlKey::",stringify!($other), "), " )*,
            do_keys!(@RDOC $($rest)*)
        )
    };
    (@ADOC +( $($other:ident),* ) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC -?($tp:ty) $($rest:tt)*) => {
        concat!(
            "`[`=\"`<[",stringify!($tp),"]>`\"`]`",
            do_keys!(@LDOC $($rest)*)
        )
    };
    (@RDOC -?($tp:ty) $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC -?($tp:ty) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC @($tp:ty) $($rest:tt)*) => {
        concat!(
            "=\"`<[",stringify!($tp),"]>`\"",
            do_keys!(@LDOC $($rest)*)
        )
    };
    (@RDOC @($tp:ty) $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC @($tp:ty) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@DOC $tag:ident = $key:literal $($rest:tt)*) => {
        concat!(
            "<div class=\"ftml-syntax\">\n\n","`",ftml!($key),"`",
            do_keys!(@LDOC $($rest)*),
            "\n\n",
            do_keys!(@RDOC $($rest)*),
            "\n\n</div>",
            do_keys!(@ADOC $($rest)*)
        )
    };
    (@ENUM $(
        $(#[$meta:meta])*
        $tag:ident = $key:literal
        {$($rest:tt)*} :=
            $($todo:ident)?
            $(
                ($ext:ident,$attrs:ident,$keys:ident,$node:ident) => {$($impl:tt)+}
                $(=> $open:ident $({$($f:ident:$ft:ty),*$(,)?})? $( ($($tn:ident:$t:ty),*) )? )?
                $(+ $close:ident => $closeb:block   )?
            )?
    ),* $(,)? ) => {
        #[derive(Copy,Clone,PartialEq, Eq,Hash,serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
        #[repr(u8)]
        pub enum FtmlKey {
            $(
                #[doc = do_keys!(@DOC $tag = $key $($rest)* )]
                $(#[$meta])* $tag
            ),*
        }

        impl FtmlKey {
            #[must_use]#[inline]
            pub const fn as_str(self) -> &'static str {
                static AS_STRS: [&str;NUM_KEYS as usize] = [$($key),*];
                AS_STRS[(self as u8) as usize]
            }
            #[inline]#[must_use]
            pub const fn as_u8(self) -> u8 {
                self as _
            }
            #[must_use]
            pub const fn from_u8(b:u8) -> Option<Self> {
                $(
                    if b == Self::$tag as u8 { return Some(Self::$tag);}
                )*
                None
            }
            #[must_use]#[inline]
            pub const fn attr_name(self) -> &'static str {
                static ATTR_STRS: [&str;NUM_KEYS as usize] = [$(ftml!($key)),*];
                ATTR_STRS[(self as u8) as usize]
            }
            #[must_use]
            pub fn from_attr(s:&str) -> Option<Self> {
                match s {
                    $( ftml!($key) => Some(Self::$tag) ),*,
                    _ => None
                }
            }

            #[must_use]
            pub const fn all_rules<E:crate::extraction::FtmlExtractor>() -> FtmlRuleSet<E> {
                FtmlRuleSet([$(
                    do_keys!(@fun $tag $($todo)? $(
                        ($ext,$attrs,$keys,$node) => {$($impl)*}
                        //$(=> $open $($b)? $( ($($t),*) )?     )?
                    )? )
                ),*])
            }
        }
        pub enum OpenFtmlElement {
            $($($(
                $open $({$($f:$ft),*})? $( ( $($t),*) )? ,
            )?)?)*
        }
        impl<N: crate::extraction::nodes::FtmlNode + std::fmt::Debug> crate::extraction::state::ExtractorState<N> {
            pub fn add2(&mut self,e:OpenFtmlElement,node:&N) {
                match e {
                    $($($(
                        OpenFtmlElement::$open $({$($f),*})? $( ( $($tn),*) )? => todo!() ,
                    )?)?)*
                }
            }
        }
    };

    (@fun $self:ident todo) => { |e,a,k,n| todo(Self::$self,e,a,k,n) };
    (@fun $self:ident ($ext:ident,$attrs:ident,$keys:ident,$node:ident) => {$($impl:tt)+} ) => {
        |$ext,$attrs,$keys,$node| { $($impl)*}
    };

    ( $(
        $(#[$meta:meta])*
        $tag:ident = $key:literal
        { $($rest:tt)* }
        :=
            $($todo:ident)?
            $(
                ($ext:ident,$attrs:ident,$keys:ident,$node:ident) => {$($impl:tt)+}
                $(=> $open:ident $({$($f:ident:$ft:ty),*$(,)?})? $( ($($tn:ident:$t:ty),*) )? )?
                $(+ $close:ident => $closeb:block   )?
            )?
    ),* $(,)? ) => {
        do_keys!{@ENUM $( $(#[$meta])* $tag = $key { $($rest)*} :=
            $($todo)?
            $(
                ($ext,$attrs,$keys,$node) => {$($impl)*}
                $(=> $open $({$($f:$ft),*})? $( ( $($tn:$t),*) )?     )?
                $(+ $close => $closeb   )?
            )?
        ),*}
    };
}

impl std::fmt::Display for FtmlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for FtmlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attr_name())
    }
}

do_keys! {
    /// Denotes a new [Section]. The given [SectionLevel] is only a sanity check;
    /// the actual level is determined by the occurrence within a [Document].
    Section = "section"
        { @(SectionLevel) + (Id) -!("in [LogicalParagraph]s, [Problem]s or [Slide]s") }
        := (ext,attrs,keys,node) => {
            let uri = attrs.get_elem_uri_from_id(ext, "section")?;
            del!(keys - Id);
            ret!(ext,node <- Section(uri) + Section)
        } => Section(uri:DocumentElementUri), // := section


    // ------------------------------

    Id = "id"
    {-(Section,Definition, Paragraph, Assertion, Example, Proof, SubProof, Problem, SubProblem, Slide)} := todo
}
