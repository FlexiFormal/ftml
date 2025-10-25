use std::hint::unreachable_unchecked;

use ftml_ontology::narrative::documents::TocElem;
use ftml_uris::{DocumentElementUri, DocumentUri, IsNarrativeUri};
use leptos::prelude::*;
use smallvec::SmallVec;

use crate::utils::actions::{OneShot, SetOneShotDone};

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum TocSource {
    None,
    #[default]
    Extract,
    Ready(Vec<TocElem>),
    Get,
}
impl ftml_js_utils::conversion::FromWasmBindgen for TocSource {}

impl leptos::wasm_bindgen::convert::TryFromJsValue for TocSource {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: leptos::wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        serde_wasm_bindgen::from_value(value)
    }
}

#[derive(Default)]
pub struct CurrentTOC {
    pub toc: Option<Vec<TocElem>>,
}
impl CurrentTOC {
    pub fn set(toc: Vec<TocElem>) {
        let ctw = expect_context::<RwSignal<Self>>();
        ctw.update(|ctw| ctw.toc = Some(toc));
    }
    pub(crate) fn set_title(&mut self, uri: &DocumentElementUri, title: Box<str>) {
        if let Some(e) = self.find_mut(|e| matches!(e,TocElem::Section { uri:u, .. } if u == uri)) {
            let TocElem::Section { title: t, .. } = e else {
                // SAFETY: match above
                unsafe { unreachable_unchecked() }
            };
            *t = Some(title);
            return;
        }
        tracing::warn!("Entry with uri {uri} not found!");
    }

    fn get_toc_at<'t>(&'t mut self, id: &str) -> Option<&'t mut Vec<TocElem>> {
        let mut path = id.split('/');
        let _ = path.next_back()?;
        let mut toc = match &mut self.toc {
            Some(toc) => toc,
            n @ None => {
                *n = Some(Vec::new());
                // SAFETY we literally just made it Some()
                unsafe { n.as_mut().unwrap_unchecked() }
            }
        };
        loop {
            let Some(next) = path.next() else {
                return Some(toc);
            };
            if let Some(next) = toc.iter_mut().find_map(|t| match t {
                TocElem::Section { id, children, .. } | TocElem::Inputref { id, children, .. }
                    if id.rsplit_once('/').is_some_and(|(_, last)| last == next) || id == next =>
                {
                    Some(children)
                }
                _ => None,
            }) {
                toc = next;
            } else {
                return None;
            }
        }
    }
    pub(crate) fn insert_section(&mut self, id: String, uri: DocumentElementUri) {
        let Some(ch) = self.get_toc_at(&id) else {
            tracing::warn!("Entry with id {id} not found!");
            return;
        };
        ch.push(TocElem::Section {
            title: None,
            uri,
            id,
            children: Vec::new(),
        });
    }
    pub(crate) fn insert_inputref(&mut self, id: String, uri: DocumentUri) {
        let Some(ch) = self.get_toc_at(&id) else {
            return;
        };
        ch.push(TocElem::Inputref {
            uri,
            title: None,
            id,
            children: Vec::new(),
        });
    }
    pub fn iter_dfs(&self) -> Option<impl Iterator<Item = &TocElem>> {
        struct TOCIterator<'b> {
            curr: std::slice::Iter<'b, TocElem>,
            stack: SmallVec<std::slice::Iter<'b, TocElem>, 2>,
        }
        impl<'b> Iterator for TOCIterator<'b> {
            type Item = &'b TocElem;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if let Some(elem) = self.curr.next() {
                        let children: &'b [_] = match elem {
                            TocElem::Section { children, .. }
                            | TocElem::Inputref { children, .. }
                            | TocElem::SkippedSection { children } => children,
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

    pub fn find_mut(&mut self, pred: impl Fn(&TocElem) -> bool) -> Option<&mut TocElem> {
        let mut curr: std::slice::IterMut<TocElem> = self.toc.as_mut()?.iter_mut();
        let mut stack: SmallVec<std::slice::IterMut<TocElem>, 2> = SmallVec::new();
        loop {
            if let Some(elem) = curr.next() {
                if pred(elem) {
                    return Some(elem);
                }
                let children: &mut [_] = match elem {
                    TocElem::Section { children, .. }
                    | TocElem::Inputref { children, .. }
                    | TocElem::SkippedSection { children } => children,
                    _ => return Some(elem),
                };
                stack.push(std::mem::replace(&mut curr, children.iter_mut()));
            } else if let Some(s) = stack.pop() {
                curr = s;
            } else {
                return None;
            }
        }
    }
}

pub struct NavElems {
    //initialized: RwSignal<bool>,
    ids: rustc_hash::FxHashMap<String, SectionOrInputref>,
    titles: rustc_hash::FxHashMap<DocumentUri, RwSignal<String>>,
    redo: Option<String>,
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
            redo: None, //initialized: RwSignal::new(false),
        }
    }

    pub fn update_untracked<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().update_untracked(f)
    }

    pub(crate) fn new_inputref(id: &str) -> (String, OneShot, SetOneShotDone) {
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

    pub fn retry() {
        if let Some(selfie) = use_context::<RwSignal<Self>>()
            && let Some(s) = selfie.update_untracked(|s| std::mem::take(&mut s.redo))
        {
            Self::navigate_to(selfie, &s);
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn navigate_to(selfie: RwSignal<Self>, _id: &str) {
        selfie.update_untracked(|#[allow(unused_variables)] slf| {
            #[cfg(any(feature = "csr", feature = "hydrate"))]
            {
                #[allow(clippy::used_underscore_binding)]
                let id = _id;
                tracing::trace!("Looking for #{id}");
                let mut curr = id;
                loop {
                    match slf.ids.get(curr) {
                        None => {
                            tracing::debug!("navigation id {curr} not known (yet)\n{:?}!", slf.ids);
                            slf.redo = Some(id.to_string());
                        }
                        Some(SectionOrInputref::Section) => {
                            tracing::debug!("Navigating to #{curr}");
                            #[allow(unused_variables)]
                            if let Some(e) = document().get_element_by_id(curr) {
                                tracing::trace!("scrolling to #{curr}");
                                #[cfg(target_family = "wasm")]
                                {
                                    let options = web_sys::ScrollIntoViewOptions::new();
                                    options.set_behavior(web_sys::ScrollBehavior::Smooth);
                                    options.set_block(web_sys::ScrollLogicalPosition::Start);
                                    e.scroll_into_view_with_scroll_into_view_options(&options);
                                }
                            } else {
                                tracing::warn!("section with id {curr} not found!");
                            }
                            return;
                        }
                        Some(SectionOrInputref::Inputref(a)) => {
                            if !a.is_done_untracked() {
                                tracing::trace!("expanding inputref {curr}");
                                let id = id.to_string();
                                a.on_set(move || {
                                    tracing::trace!("resuming navigation to {id}");
                                    Self::navigate_to(selfie, &id);
                                });
                                a.activate();
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
        });
    }

    pub fn navigate_to_fragment() {
        let fragment = RwSignal::new(String::new());
        let selfie = expect_context::<RwSignal<Self>>();
        tracing::trace!("Setting up navigation system");

        #[cfg(any(feature = "csr", feature = "hydrate"))]
        {
            if let Ok(mut frag) = window().location().hash()
                && frag.starts_with('#')
            {
                frag.remove(0);
                tracing::warn!("Current fragment: {frag}");
                fragment.set(frag);
            }
            fragment_listener(fragment);
        }

        let done = RwSignal::new(false);
        Effect::new(move || {
            done.set(true);
        });
        Effect::new(move || {
            fragment.track();
            if done.get() {
                let fragment = fragment.get();
                if !fragment.is_empty() {
                    tracing::warn!("Navigating to {fragment}");
                    Self::navigate_to(selfie, &fragment);
                }
            }
        });
    }

    /*
    pub(crate) fn with_untracked<R>(f: impl FnOnce(&Self) -> R) -> R {
        expect_context::<RwSignal<Self>>()
            .try_with_untracked(f)
            .expect("this should not happen")
    }
     */
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
fn fragment_listener(signal: RwSignal<String>) {
    use leptos::wasm_bindgen::JsCast;
    fn get_anchor(e: leptos::web_sys::Element) -> Option<leptos::web_sys::Element> {
        let mut curr = e;
        loop {
            if curr.tag_name().to_uppercase() == "A" {
                return Some(curr);
            }
            if curr.tag_name().to_uppercase() == "BODY" {
                return None;
            }
            if let Some(parent) = curr.parent_element() {
                curr = parent;
            } else {
                return None;
            }
        }
    }
    tracing::info!("Setting up fragment listener");
    let on_hash_change =
        leptos::wasm_bindgen::prelude::Closure::wrap(Box::new(move |_e: leptos::web_sys::Event| {
            if let Ok(mut frag) = window().location().hash()
                && frag.starts_with('#')
            {
                frag.remove(0);
                tracing::trace!("Updating URL fragment to {frag}");
                signal.set(frag);
            }
        }) as Box<dyn FnMut(_)>);

    let on_anchor_click = leptos::wasm_bindgen::prelude::Closure::wrap(Box::new(
        move |e: leptos::web_sys::MouseEvent| {
            if let Some(e) = e
                .target()
                .and_then(|t| t.dyn_into::<leptos::web_sys::Element>().ok())
                && let Some(e) = get_anchor(e)
                && let Some(mut href) = e.get_attribute("href")
                && href.starts_with('#')
            {
                href.remove(0);
                tracing::trace!("Updating URL fragment as {href}");
                signal.set(href);
            }
        },
    ) as Box<dyn FnMut(_)>);

    tracing::trace!("Setting URL listeners");

    let _ = window()
        .add_event_listener_with_callback("hashchange", on_hash_change.as_ref().unchecked_ref());
    let _ = window()
        .add_event_listener_with_callback("popstate", on_hash_change.as_ref().unchecked_ref());
    let _ = window()
        .add_event_listener_with_callback("click", on_anchor_click.as_ref().unchecked_ref());
    on_hash_change.forget();
    on_anchor_click.forget();
}
