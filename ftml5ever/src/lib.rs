#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

use ftml_core::extraction::{
    FtmlExtractionError, FtmlStateExtractor, OpenFtmlElement, state::ExtractorState,
};
use ftml_ontology::utils::Css;

mod ever;
mod parser;

pub struct HtmlExtractor {
    errors: String,
    css: Vec<Css>,
    refs: Vec<u8>,
    title: Option<Box<str>>,
    //document:UncheckedDocument,
    //backend: &'a AnyBackend,
    state: ExtractorState<ever::NodeRef>,
}

static RULES: ftml_core::extraction::FtmlRuleSet<HtmlExtractor> =
    ftml_core::extraction::FtmlRuleSet::new();
impl FtmlStateExtractor for HtmlExtractor {
    type Attributes<'a> = ever::Attributes;
    type Node = ever::NodeRef;
    type Return = ();
    const RULES: &'static ftml_core::extraction::FtmlRuleSet<Self> = &RULES;
    #[cfg(feature = "rdf")]
    const DO_RDF: bool = true;
    #[cfg(not(feature = "rdf"))]
    const DO_RDF: bool = false;

    #[inline]
    fn state_mut(&mut self) -> &mut ExtractorState<ever::NodeRef> {
        &mut self.state
    }
    #[inline]
    fn state(&self) -> &ExtractorState<ever::NodeRef> {
        &self.state
    }
    /// ### Errors
    #[inline]
    fn on_add(&mut self, _: &OpenFtmlElement) -> Result<Self::Return, FtmlExtractionError> {
        Ok(())
    }
}
