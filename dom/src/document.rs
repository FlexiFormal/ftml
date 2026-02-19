use crate::{
    //counters::{LogicalLevel, SectionCounters},
    counters::LogicalLevel,
    extractor::DomExtractor,
    markers::ParagraphInfo,
    structure::{DocumentStructure, Inputref, SectionInfo},
    toc::TocSource,
    utils::{ContextChain, ModuleContext, local_cache::SendBackend, owned},
};
use ftml_ontology::{
    narrative::{
        documents::TocElem,
        elements::{
            SectionLevel,
            paragraphs::{ParagraphFormatting, ParagraphKind},
        },
    },
    terms::VarOrSym,
};
use ftml_parser::extraction::ArgumentPosition;
use ftml_uris::{DocumentElementUri, DocumentUri, Id, Language, NarrativeUri, SymbolUri};
use leptos::prelude::*;
use std::str::FromStr;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentMeta {
    pub uri: Option<DocumentUri>,
    pub language: Option<Language>,
}
impl DocumentMeta {
    pub fn get() -> Self {
        leptos_meta::provide_meta_context();
        let mut meta = Self::default();
        let document = leptos::tachys::dom::document();
        let Some(elem) = document.document_element() else {
            return meta;
        };
        if let Some(res) = elem.get_attribute("resource") {
            match DocumentUri::from_str(&res) {
                Ok(u) => meta.uri = Some(u),
                Err(e) => tracing::warn!("Invalid document uri in `resource` of <html> node: {e}"),
            }
        }
        meta
    }
}

pub fn setup_document<Be: SendBackend, Ch: IntoView + 'static>(
    uri: DocumentUri,
    is_stripped: bool,
    toc: TocSource,
    children: impl FnOnce() -> Ch,
) -> impl IntoView {
    fn setup<Be: SendBackend>(uri: DocumentUri, is_stripped: bool, toc: TocSource) {
        provide_context(RwSignal::new(DomExtractor::new(
            uri.clone(),
            uri.clone().into(),
            is_stripped,
        )));
        provide_context(InDocument(uri.clone()));
        provide_context(CurrentUri(uri.clone().into()));
        provide_context(ContextUri(uri.into()));
        DocumentStructure::set::<Be>(toc);
        DocumentStructure::navigate_to_fragment();
        ModuleContext::reset();
    }
    setup::<Be>(uri, is_stripped, toc);
    children()
}

#[derive(Clone, PartialEq, Eq)]
pub struct WithHead(pub Option<VarOrSym>);

#[derive(Clone, PartialEq, Eq)]
struct InDocument(DocumentUri);

#[derive(Clone, PartialEq, Eq)]
struct ContextUri(NarrativeUri);

#[derive(Clone, PartialEq, Eq)]
pub struct CurrentUri(pub NarrativeUri);

pub struct DocumentState;
impl DocumentState {
    /// ### Panics
    pub fn current_uri() -> NarrativeUri {
        with_context::<CurrentUri, _>(|s| s.0.clone()).expect("Not in a document context")
    }

    /// ### Panics
    pub fn document_uri() -> DocumentUri {
        with_context::<InDocument, _>(|s| s.0.clone()).expect("Not in a document context")
    }

    /// ### Panics
    pub fn context_uri() -> NarrativeUri {
        with_context::<ContextUri, _>(|s| s.0.clone()).expect("Not in a document context")
    }

    pub fn current_term_head() -> Option<VarOrSym> {
        use_context::<WithHead>().and_then(|w| w.0)
    }

    pub fn with_head<F: FnOnce() -> AnyView>(head: VarOrSym, then: F) -> AnyView {
        owned(|| {
            provide_context(WithHead(Some(head)));
            then()
        })
        .into_any()
    }

    /// ### Panics
    pub fn force_uri(uri: DocumentElementUri) {
        with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.update_untracked(|e| e.state.set_next_uri(uri));
        })
        .expect("Not in a document context");
    }

    /// ### Panics
    pub fn finished_parsing() -> ReadSignal<bool> {
        with_context::<RwSignal<DomExtractor>, _>(|s| s.with_untracked(|e| e.is_done_read))
            .expect("Not in a document context")
    }

    #[inline]
    pub fn in_inputref() -> bool {
        DocumentStructure::in_inputref()
    }

    pub fn arguments() -> impl Iterator<Item = ArgumentPosition> {
        ContextChain::<Option<ArgumentPosition>>::iter().flatten()
    }

    #[inline]
    pub(crate) fn do_inputref(target: DocumentUri, uri: DocumentElementUri) -> Inputref {
        DocumentStructure::new_inputref(uri, target)
    }

    pub fn inner_document<F: FnOnce() -> AnyView>(
        target: DocumentUri,
        uri: &DocumentElementUri,
        is_stripped: bool,
        f: F,
    ) -> AnyView {
        fn setup(target: DocumentUri, uri: &DocumentElementUri, is_stripped: bool) {
            let context = DocumentState::context_uri() & uri.name();
            provide_context(RwSignal::new(DomExtractor::new(
                target.clone(),
                context.clone().into(),
                is_stripped,
            )));
            provide_context(CurrentUri(target.clone().into()));
            provide_context(InDocument(target));
            provide_context(ContextUri(context.into()));
            ModuleContext::reset();
        }
        setup(target, uri, is_stripped);
        f()
    }

    pub fn no_document<V: IntoView, F: FnOnce() -> V>(f: F) -> impl IntoView + use<V, F> {
        fn setup() {
            provide_context(RwSignal::new(DomExtractor::new(
                DocumentUri::no_doc().clone(),
                DocumentUri::no_doc().clone().into(),
                true,
            )));
            provide_context(InDocument(DocumentUri::no_doc().clone()));
            provide_context(ContextUri(DocumentUri::no_doc().clone().into()));
            provide_context(CurrentUri(DocumentUri::no_doc().clone().into()));
            DocumentStructure::set_empty();
            ModuleContext::reset();
        }
        setup();
        f()
    }

    #[inline]
    pub(crate) fn new_section(uri: DocumentElementUri) -> SectionInfo {
        DocumentStructure::new_section(uri)
    }

    /*
    pub fn get_toc() -> ReadSignal<Vec<TocElem>> {
        expect_context::<DocumentStructure>().toc.read_only()
    }
     */

    pub(crate) fn new_paragraph<V: IntoView>(
        uri: DocumentElementUri,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        fors: Vec<SymbolUri>,
        f: impl FnOnce(ParagraphInfo) -> V,
    ) -> impl IntoView {
        provide_context(CurrentUri(uri.clone().into()));
        //leptos::logging::log!("Paragraph {uri}");
        let (style, class) = DocumentStructure::get_para(kind, &styles);
        f(ParagraphInfo {
            uri,
            style,
            class,
            kind,
            formatting,
            styles,
            fors,
        })
    }

    pub(crate) fn new_problem(uri: DocumentElementUri, styles: &[Id]) -> (Memo<String>, String) {
        provide_context(CurrentUri(uri.into()));
        DocumentStructure::get_problem(styles)
    }

    pub(crate) fn new_slide(uri: DocumentElementUri) {
        provide_context(CurrentUri(uri.into()));
        DocumentStructure::slide_inc();
    }

    pub fn current_section_level() -> LogicalLevel {
        use_context().unwrap_or(LogicalLevel::None)
    }

    /*

    pub(crate) fn update_counters<R>(f: impl FnOnce(&mut SectionCounters) -> R) -> R {
        update_context::<SectionCounters, _>(f).expect("Not in a document context")
    }
     */

    #[inline]
    pub(crate) fn skip_section() {
        DocumentStructure::skip_section();
    }
}
