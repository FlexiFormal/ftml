use ftml_ontology::narrative::elements::paragraphs::ParagraphKind;
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri};
use leptos::prelude::*;

use crate::utils::actions::{OneShot, SetOneShotDone};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(tag = "type")]
/// An entry in a table of contents. Either:
/// 1. a section; the title is assumed to be an HTML string, or
/// 2. an inputref to some other document; the URI is the one for the
///    inputref itself; not the referenced Document. For the TOC,
///    which document is inputrefed is actually irrelevant.
pub enum TOCElem {
    /// A section; the title is assumed to be an HTML string
    Section {
        title: Option<String>,
        uri: DocumentElementUri,
        id: String,
        children: Vec<TOCElem>,
    },
    SkippedSection {
        children: Vec<TOCElem>,
    },
    /// An inputref to some other document; the URI is the one for the
    /// referenced Document.
    Inputref {
        uri: DocumentUri,
        title: Option<String>,
        id: String,
        children: Vec<TOCElem>,
    },
    Paragraph {
        styles: Vec<Id>,
        kind: ParagraphKind,
    },
    Slide, //{uri:DocumentElementUri}
}

pub struct NavElems {
    initialized: RwSignal<bool>,
    ids: rustc_hash::FxHashMap<String, SectionOrInputref>,
    titles: rustc_hash::FxHashMap<DocumentUri, RwSignal<String>>,
}

#[derive(Debug, Clone)]
pub enum SectionOrInputref {
    Section,
    Inputref(OneShot),
}

#[derive(Clone)]
pub(crate) struct CurrentId(pub String);

impl NavElems {
    pub(crate) fn new() -> Self {
        Self {
            ids: std::collections::HashMap::default(),
            titles: std::collections::HashMap::default(),
            initialized: RwSignal::new(false),
        }
    }

    pub(crate) fn update_untracked<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().update_untracked(f)
    }

    pub(crate) fn new_inpuref(id: &str) -> (String, OneShot, SetOneShotDone) {
        let (os, done) = OneShot::new();
        let id = Self::update_untracked(|ne| {
            let id = Self::new_id(id);
            ne.ids.insert(id.clone(), SectionOrInputref::Inputref(os));
            id
        });
        (id, os, done)
    }
    pub(crate) fn new_section(id: &str) -> String {
        Self::update_untracked(|ne| {
            let id = Self::new_id(id);
            ne.ids.insert(id.clone(), SectionOrInputref::Section);
            id
        })
    }
    /*
    pub(crate) fn in_id<V: IntoView>(id: String, then: impl FnOnce() -> V) -> impl IntoView {
        let owner = Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let children = owner.with(move || {
            provide_context(CurrentId(id));
            then()
        });
        OwnedView::new_with_owner(children, owner)
    }
     */
    fn new_id(s: &str) -> String {
        with_context::<CurrentId, _>(|id| {
            if id.0.is_empty() {
                s.to_string()
            } else {
                format!("{}/{s}", id.0)
            }
        })
        .unwrap_or_else(|| s.to_string())
    }
    pub fn get_title(uri: DocumentUri) -> RwSignal<String> {
        Self::update_untracked(|slf| match slf.titles.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => *e.get(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let name = e.key().document_name().to_string();
                *e.insert(RwSignal::new(name))
            }
        })
    }
    pub fn set_title(&mut self, uri: DocumentUri, title: String) {
        match self.titles.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => e.get().set(title),
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(RwSignal::new(title));
            }
        }
    }
    #[allow(clippy::missing_const_for_fn)]
    pub fn navigate_to(&self, _id: &str) {
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        {
            #[allow(clippy::used_underscore_binding)]
            let id = _id;
            tracing::trace!("Looking for #{id}");
            let mut curr = id;
            loop {
                match self.ids.get(curr) {
                    None => (),
                    Some(SectionOrInputref::Section) => {
                        tracing::trace!("Navigating to #{curr}");
                        #[allow(unused_variables)]
                        if let Some(e) = document().get_element_by_id(curr) {
                            #[cfg(target_family = "wasm")]
                            {
                                let options = web_sys::ScrollIntoViewOptions::new();
                                options.set_behavior(web_sys::ScrollBehavior::Smooth);
                                options.set_block(web_sys::ScrollLogicalPosition::Start);
                                e.scroll_into_view_with_scroll_into_view_options(&options);
                            }
                        }
                        return;
                    }
                    Some(SectionOrInputref::Inputref(a)) => {
                        if !a.is_done_untracked() {
                            a.activate();
                            if a.is_done() {
                                return self.navigate_to(id);
                            }
                        }
                    }
                }
                if let Some((a, _)) = curr.rsplit_once('/') {
                    curr = a;
                } else {
                    return;
                }
            }
        }
    }

    /*
    pub(crate) fn with_untracked<R>(f: impl FnOnce(&Self) -> R) -> R {
        expect_context::<RwSignal<Self>>()
            .try_with_untracked(f)
            .expect("this should not happen")
    }
     */
}
