#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

use crate::utils::SourceRange;

pub mod domain;
pub mod narrative;
pub mod terms;
pub mod utils;
pub(crate) mod __private {
    pub trait Sealed {}
}

pub trait Ftml: __private::Sealed {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple>;
    fn source_range(&self) -> SourceRange;
}
