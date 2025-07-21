use std::str::FromStr;

use crate::{
    counters::SectionCounters,
    extractor::DomExtractor,
    markers::SectionInfo,
    toc::{NavElems, TOCElem},
};
use ftml_ontology::narrative::documents::{DocumentCounter, DocumentStyle};
use ftml_uris::{DocumentElementUri, DocumentUri, Language};
use leptos::{prelude::*, tachys::reactive_graph::OwnedView};
use smallvec::SmallVec;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
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
    children: impl FnOnce() -> Ch,
    uri: DocumentUri,
) -> impl IntoView {
    provide_context(RwSignal::new(DomExtractor::new(uri)));
    provide_context(SectionCounters::default());
    provide_context(RwSignal::new(CurrentTOC::default()));
    provide_context(RwSignal::new(NavElems::new()));
    children()
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct InInputref(bool);

pub struct DocumentState;
impl DocumentState {
    pub fn with_styles_untracked<R>(
        f: impl FnOnce(&[DocumentCounter], &[DocumentStyle]) -> R,
    ) -> R {
        with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|e| f(&e.state.counters, &e.state.styles))
        })
        .expect("Not in a document context")
    }
    #[inline]
    pub fn in_inputref() -> bool {
        use_context::<InInputref>().is_some_and(|b| b.0)
    }

    pub(crate) fn new_section<V: IntoView>(
        uri: DocumentElementUri,
        f: impl FnOnce(SectionInfo) -> V,
    ) -> impl IntoView {
        let id = NavElems::new_section(uri.name.last());
        let mut counters: SectionCounters = expect_context();
        let (style, class) = counters.next_section();
        let lvl = counters.current_level();
        let owner = Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let children = owner.with(move || {
            provide_context(counters);
            f(SectionInfo {
                uri,
                style,
                class,
                lvl,
                id,
            })
        });
        OwnedView::new_with_owner(children, owner)
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
