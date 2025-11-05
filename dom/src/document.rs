use crate::{
    //counters::{LogicalLevel, SectionCounters},
    counters::LogicalLevel,
    extractor::DomExtractor,
    markers::ParagraphInfo,
    structure::{DocumentStructure, Inputref, SectionInfo},
    utils::{ContextChain, local_cache::SendBackend, owned},
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
        //document.document_uri()
    }
}

pub fn setup_document<Be: SendBackend, Ch: IntoView + 'static>(
    uri: DocumentUri,
    is_stripped: bool,
    children: impl FnOnce() -> Ch,
) -> impl IntoView {
    provide_context(RwSignal::new(DomExtractor::new(
        uri.clone(),
        uri.clone().into(),
        is_stripped,
    )));
    provide_context(InDocument(uri.clone()));
    provide_context(CurrentUri(uri.clone().into()));
    provide_context(ContextUri(uri.into()));
    DocumentStructure::set::<Be>();
    DocumentStructure::navigate_to_fragment();

    /*
    provide_context(SectionCounters::default());
    let current_toc = match with_context::<TocSource, _>(|c| {
        if let TocSource::Ready(r) = c {
            Some(r.clone())
        } else {
            None
        }
    }) {
        Some(Some(c)) => CurrentTOC { toc: Some(c) },
        Some(None) => CurrentTOC::default(),
        None => {
            provide_context(TocSource::default());
            CurrentTOC::default()
        }
    };
    provide_context(RwSignal::new(current_toc));
    provide_context(RwSignal::new(NavElems::new()));
     */
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
        with_context::<CurrentUri, _>(|s| {
            s.0.clone() //s.with_untracked(|e| e.state.document.clone())
        })
        .expect("Not in a document context")
    }

    /// ### Panics
    pub fn document_uri() -> DocumentUri {
        with_context::<InDocument, _>(|s| {
            s.0.clone() //s.with_untracked(|e| e.state.document.clone())
        })
        .expect("Not in a document context")
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
        /*
        let (id, replace, replacing_done) = NavElems::new_inputref(uri.name.last());
        let counters = SectionCounters::inputref(target.clone(), id.clone());
        let title = NavElems::get_title(target.clone());
        if with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b) {
            let current_toc = expect_context::<RwSignal<CurrentTOC>>();
            current_toc.update(|t| t.insert_inputref(id.clone(), target.clone()));
        }
        provide_context(counters);
        provide_context(CurrentId(id.clone()));
        f(InputrefInfo {
            uri,
            target,
            replace,
            replacing_done,
            id,
            title,
        })
        */
    }

    pub fn inner_document<F: FnOnce() -> AnyView>(
        target: DocumentUri,
        uri: &DocumentElementUri,
        is_stripped: bool,
        f: F,
    ) -> AnyView {
        let context = Self::context_uri() & uri.name();
        provide_context(RwSignal::new(DomExtractor::new(
            target.clone(),
            context.clone().into(),
            is_stripped,
        )));
        provide_context(CurrentUri(target.clone().into()));
        provide_context(InDocument(target));
        provide_context(ContextUri(context.into()));
        f()
    }

    pub fn no_document<V: IntoView, F: FnOnce() -> V>(f: F) -> impl IntoView + use<V, F> {
        provide_context(RwSignal::new(DomExtractor::new(
            DocumentUri::no_doc().clone(),
            DocumentUri::no_doc().clone().into(),
            true,
        )));
        provide_context(InDocument(DocumentUri::no_doc().clone()));
        provide_context(ContextUri(DocumentUri::no_doc().clone().into()));
        provide_context(CurrentUri(DocumentUri::no_doc().clone().into()));
        DocumentStructure::set_empty();
        f()
    }

    #[inline]
    pub(crate) fn new_section(uri: DocumentElementUri) -> SectionInfo {
        DocumentStructure::new_section(uri)
        /*
        let id = NavElems::new_section(uri.name.last());
        let mut counters: SectionCounters = expect_context();
        let (style, class) = counters.next_section();
        let lvl = counters.current_level();
        if with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b) {
            let current_toc = expect_context::<RwSignal<CurrentTOC>>();
            current_toc.update(|t| t.insert_section(id.clone(), uri.clone()));
        }
        provide_context(counters);
        provide_context(CurrentId(id.clone()));
        f(SectionInfo {
            uri,
            style,
            class,
            lvl,
            id,
        })
         */
    }

    pub fn get_toc() -> ReadSignal<Vec<TocElem>> {
        expect_context::<DocumentStructure>().toc.read_only()
    }

    pub(crate) fn new_paragraph<V: IntoView>(
        uri: DocumentElementUri,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        fors: Vec<SymbolUri>,
        f: impl FnOnce(ParagraphInfo) -> V,
    ) -> impl IntoView {
        provide_context(CurrentUri(uri.clone().into()));
        let (style, class) = expect_context::<DocumentStructure>().get_para(kind, &styles);
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
        expect_context::<DocumentStructure>().get_problem(styles)
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
