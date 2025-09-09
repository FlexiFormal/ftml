#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

use ftml_ontology::{narrative::DocumentRange, utils::Css};
use ftml_parser::extraction::{
    FtmlExtractionError, FtmlStateExtractor, OpenFtmlElement,
    state::{ExtractionResult, ExtractorState},
};
use ftml_uris::DocumentUri;

mod ever;
mod parser;

pub struct FtmlResult {
    pub ftml: Box<str>,
    pub css: Box<[Css]>,
    pub errors: Box<[FtmlExtractionError]>,
    pub doc: ExtractionResult,
    pub body: DocumentRange,
    pub inner_offset: u32,
}

/// # Errors
pub fn run(
    ftml: &str,
    img: impl Fn(&str) -> Option<String>,
    css: impl Fn(&str) -> Option<Box<str>>,
    uri: DocumentUri,
    rdf: bool,
) -> Result<FtmlResult, String> {
    use html5ever::tendril::{SliceExt, TendrilSink};
    let parser = parser::HtmlParser {
        document_node: ever::NodeRef::new_document(),
        body: std::cell::Cell::new((DocumentRange::default(), 0)),
        errors: std::cell::RefCell::new(Vec::new()),
        img,
        css,
        extractor: std::cell::RefCell::new(HtmlExtractor {
            parse_errors: String::new(),
            css: Vec::new(),
            state: ExtractorState::new(uri, rdf),
        }),
    };
    html5ever::parse_document(parser, html5ever::ParseOpts::default())
        .from_utf8()
        .one(ftml.as_bytes().to_tendril())
}

pub struct HtmlExtractor {
    parse_errors: String,
    css: Vec<Css>,
    //document:UncheckedDocument,
    //backend: &'a AnyBackend,
    state: ExtractorState<ever::NodeRef>,
}

static RULES: ftml_parser::extraction::FtmlRuleSet<HtmlExtractor> =
    ftml_parser::extraction::FtmlRuleSet::new();
impl FtmlStateExtractor for HtmlExtractor {
    type Attributes<'a> = ever::Attributes;
    type Node = ever::NodeRef;
    type Return = ();
    const RULES: &'static ftml_parser::extraction::FtmlRuleSet<Self> = &RULES;
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
