use ftml_ontology::narrative::elements::paragraphs::ParagraphKind;
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri};
use leptos::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
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
    pub initialized: RwSignal<bool>,
    pub ids: rustc_hash::FxHashMap<String, SectionOrInputref>,
    pub titles: rustc_hash::FxHashMap<DocumentUri, RwSignal<String>>,
    id_prefix: String,
}

#[derive(Debug, Clone)]
pub enum SectionOrInputref {
    Section,
    Inputref(RwSignal<bool>, RwSignal<bool>),
}

impl NavElems {
    pub(crate) fn new() -> Self {
        Self {
            ids: std::collections::HashMap::default(),
            titles: std::collections::HashMap::default(),
            initialized: RwSignal::new(false),
            id_prefix: String::new(),
        }
    }
    pub(crate) fn new_section(id: &str) -> String {
        Self::update_untracked(|ne| {
            let id = ne.new_id(id);
            ne.ids.insert(id.clone(), SectionOrInputref::Section);
            id
        })
    }
    fn new_id(&self, s: &str) -> String {
        if self.id_prefix.is_empty() {
            s.to_string()
        } else {
            format!("{}/{s}", self.id_prefix)
        }
    }
    pub fn get_title(&mut self, uri: DocumentUri) -> RwSignal<String> {
        match self.titles.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => *e.get(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let name = e.key().document_name().to_string();
                *e.insert(RwSignal::new(name))
            }
        }
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
    pub fn navigate_to(&self, id: &str) {
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        {
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
                    Some(SectionOrInputref::Inputref(s1, s2)) => {
                        if !s2.get_untracked() {
                            s1.set(true);
                            if s2.get() {
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

    pub(crate) fn update_untracked<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().update_untracked(f)
    }
    /*
    pub(crate) fn with_untracked<R>(f: impl FnOnce(&Self) -> R) -> R {
        expect_context::<RwSignal<Self>>()
            .try_with_untracked(f)
            .expect("this should not happen")
    }
     */
}
