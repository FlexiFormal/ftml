use std::str::FromStr;

use crate::{
    VarOrSym,
    counters::{LogicalLevel, SectionCounters},
    extractor::DomExtractor,
    markers::{InputrefInfo, SectionInfo},
    toc::{CurrentId, NavElems, TOCElem},
};
use ftml_core::extraction::{FtmlExtractor, OpenDomainElement};
use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::{DocumentElementUri, DocumentUri, Language, NarrativeUri};
use leptos::prelude::*;
use smallvec::SmallVec;

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
    children: impl FnOnce() -> Ch,
) -> impl IntoView {
    provide_context(RwSignal::new(DomExtractor::new(uri.clone(), uri.into())));
    provide_context(SectionCounters::default());
    provide_context(RwSignal::new(CurrentTOC::default()));
    provide_context(RwSignal::new(NavElems::new()));
    children()
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct InInputref(bool);

pub struct DocumentState;
impl DocumentState {
    /// ### Panics
    pub fn current_uri() -> NarrativeUri {
        with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|e| e.get_narrative_uri().owned())
        })
        .expect("Not in a document context")
    }

    /// ### Panics
    pub fn document_uri() -> DocumentUri {
        with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|e| e.state.document.clone())
        })
        .expect("Not in a document context")
    }

    /// ### Panics
    pub fn context_uri() -> NarrativeUri {
        with_context::<RwSignal<DomExtractor>, _>(|s| s.with_untracked(|e| e.context.clone()))
            .expect("Not in a document context")
    }

    pub fn current_term_head() -> Option<VarOrSym> {
        with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|e| match e.iterate_domain().next() {
                Some(OpenDomainElement::SymbolReference { uri, .. }) => {
                    Some(VarOrSym::S(uri.clone()))
                }
                Some(
                    OpenDomainElement::Module { .. } | OpenDomainElement::SymbolDeclaration { .. },
                )
                | None => None,
            })
        })
        .flatten()
    }

    /*
    // ### Panics
    pub fn with_styles_untracked<R>(
        f: impl FnOnce(&[DocumentCounter], &[DocumentStyle]) -> R,
    ) -> R {
        with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|e| f(&e.state.counters, &e.state.styles))
        })
        .expect("Not in a document context")
    }
     */

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
        f: F,
    ) -> impl IntoView + use<V, F> {
        let context = Self::context_uri() & uri.name();
        provide_context(DomExtractor::new(target, context.into()));
        f()
    }

    pub(crate) fn new_section<V: IntoView>(
        uri: DocumentElementUri,
        f: impl FnOnce(SectionInfo) -> V,
    ) -> impl IntoView {
        let id = NavElems::new_section(uri.name.last());
        let mut counters: SectionCounters = expect_context();
        let (style, class) = counters.next_section();
        let lvl = counters.current_level();
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

#[derive(Default)]
pub struct CurrentTOC {
    pub toc: Option<Vec<TOCElem>>,
}
impl CurrentTOC {
    pub fn iter_dfs(&self) -> Option<impl Iterator<Item = &TOCElem>> {
        struct TOCIterator<'b> {
            curr: std::slice::Iter<'b, TOCElem>,
            stack: SmallVec<std::slice::Iter<'b, TOCElem>, 2>,
        }
        impl<'b> Iterator for TOCIterator<'b> {
            type Item = &'b TOCElem;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if let Some(elem) = self.curr.next() {
                        let children: &'b [_] = match elem {
                            TOCElem::Section { children, .. }
                            | TOCElem::Inputref { children, .. }
                            | TOCElem::SkippedSection { children } => children,
                            _ => return Some(elem),
                        };
                        self.stack
                            .push(std::mem::replace(&mut self.curr, children.iter()));
                        return Some(elem);
                    } else if let Some(s) = self.stack.pop() {
                        self.curr = s;
                    } else {
                        return None;
                    }
                }
            }
        }
        self.toc.as_deref().map(|t| TOCIterator {
            curr: t.iter(),
            stack: SmallVec::new(),
        })
    }
}
