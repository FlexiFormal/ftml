#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod utils {
    mod shared_arc;
    pub use shared_arc::SharedArc;
    mod tree;
    pub use tree::*;
    #[cfg(feature = "serde")]
    mod hexable;
    #[cfg(feature = "serde")]
    pub use hexable::*;
    //mod css;
    //pub use css::*;
    pub mod regex;
}
pub mod domain;
pub mod expressions;
pub mod narrative;
pub(crate) mod __private {
    pub trait Sealed {}
}

pub trait Ftml: __private::Sealed {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple>;
}
