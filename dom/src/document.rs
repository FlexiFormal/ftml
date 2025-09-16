use crate::{
    counters::{LogicalLevel, SectionCounters},
    extractor::DomExtractor,
    markers::{InputrefInfo, ParagraphInfo, SectionInfo},
    toc::{CurrentId, CurrentTOC, NavElems, TocSource},
    utils::{ContextChain, owned},
};
use ftml_ontology::{
    narrative::elements::{
        SectionLevel,
        paragraphs::{ParagraphFormatting, ParagraphKind},
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

pub fn setup_document<Ch: IntoView + 'static>(
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
    NavElems::navigate_to_fragment();
    children()
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct InInputref(bool);

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

    pub fn with_head<V: IntoView, F: FnOnce() -> V>(
        head: VarOrSym,
        then: F,
    ) -> leptos::tachys::reactive_graph::OwnedView<V> {
        owned(|| {
            provide_context(WithHead(Some(head)));
            then()
        })
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
        use_context::<InInputref>().is_some_and(|b| b.0)
    }

    pub(crate) fn do_inputref<V: IntoView>(
        target: DocumentUri,
        uri: DocumentElementUri,
        f: impl FnOnce(InputrefInfo) -> V,
    ) -> impl IntoView {
        let (id, replace, replacing_done) = NavElems::new_inpuref(uri.name.last());
        let counters = SectionCounters::inputref(target.clone(), id.clone());
        let title = NavElems::get_title(target.clone());
        if with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b) {
            let current_toc = expect_context::<RwSignal<CurrentTOC>>();
            current_toc.update(|t| t.insert_inputref(id.clone(), target.clone()));
        }
        provide_context(counters);
        provide_context(InInputref(true));
        provide_context(CurrentId(id.clone()));
        f(InputrefInfo {
            uri,
            target,
            replace,
            replacing_done,
            id,
            title,
        })
    }

    pub fn inner_document<V: IntoView, F: FnOnce() -> V>(
        target: DocumentUri,
        uri: &DocumentElementUri,
        is_stripped: bool,
        f: F,
    ) -> impl IntoView + use<V, F> {
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
        f()
    }

    pub fn arguments() -> impl Iterator<Item = ArgumentPosition> {
        ContextChain::<Option<ArgumentPosition>>::iter().flatten()
    }

    pub(crate) fn new_section<V: IntoView>(
        uri: DocumentElementUri,
        f: impl FnOnce(SectionInfo) -> V,
    ) -> impl IntoView {
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
    }

    pub fn get_toc() -> ReadSignal<CurrentTOC> {
        expect_context::<RwSignal<CurrentTOC>>().read_only()
    }

    pub(crate) fn new_paragraph<V: IntoView>(
        uri: DocumentElementUri,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        fors: Vec<SymbolUri>,
        f: impl FnOnce(ParagraphInfo) -> V,
    ) -> impl IntoView {
        let mut counters: SectionCounters = expect_context();
        let (style, class) = counters.get_para(kind, &styles);
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

    pub(crate) fn new_problem(styles: &[Id]) -> (Memo<String>, String) {
        let mut counters: SectionCounters = expect_context();
        counters.get_problem(styles)
    }

    pub(crate) fn new_slide() {
        let counters = SectionCounters::slide_inc();
        provide_context(counters);
    }

    pub fn current_section_level() -> LogicalLevel {
        with_context(|cntrs: &SectionCounters| cntrs.current_level()).unwrap_or(LogicalLevel::None)
    }

    pub(crate) fn title_class() -> (LogicalLevel, &'static str) {
        with_context(|cntrs: &SectionCounters| {
            let lvl = cntrs.current_level();
            (
                lvl,
                match lvl {
                    LogicalLevel::Section(l) => match l {
                        SectionLevel::Part => "ftml-title-part",
                        SectionLevel::Chapter => "ftml-title-chapter",
                        SectionLevel::Section => "ftml-title-section",
                        SectionLevel::Subsection => "ftml-title-subsection",
                        SectionLevel::Subsubsection => "ftml-title-subsubsection",
                        SectionLevel::Paragraph => "ftml-title-paragraph",
                        SectionLevel::Subparagraph => "ftml-title-subparagraph",
                    },
                    LogicalLevel::BeamerSlide => "ftml-title-slide",
                    LogicalLevel::Paragraph => "ftml-title-paragraph",
                    LogicalLevel::None => "ftml-title",
                },
            )
        })
        .unwrap_or((LogicalLevel::None, "ftml-title"))
    }

    pub(crate) fn skip_section<V: IntoView>(f: impl FnOnce() -> V) -> impl IntoView {
        let mut counters: SectionCounters = expect_context();
        match counters.current_level() {
            LogicalLevel::Section(l) => {
                counters.current = LogicalLevel::Section(l.inc());
            }
            LogicalLevel::None => {
                counters.current = LogicalLevel::Section(counters.max);
            }
            _ => (),
        }
        provide_context(counters);
        f()
    }

    pub(crate) fn update_counters<R>(f: impl FnOnce(&mut SectionCounters) -> R) -> R {
        update_context::<SectionCounters, _>(f).expect("Not in a document context")
    }
}
