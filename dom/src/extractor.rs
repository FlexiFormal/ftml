use crate::{extractor, markers::Marker};
use dashmap::Entry;
use either::Either;
use ftml_ontology::narrative::DocumentRange;
use ftml_parser::{
    FtmlKey,
    extraction::{
        FtmlExtractionError, FtmlRuleSet, FtmlStateExtractor, KeyList,
        attributes::Attributes,
        nodes::FtmlNode,
        state::{ExtractionResult, ExtractorState},
    },
};
use ftml_uris::{DocumentUri, NarrativeUri};
use leptos::{
    prelude::ReadSignal,
    web_sys::{Element, NodeList, js_sys::JsString},
};
use leptos::{
    prelude::{RwSignal, WriteSignal},
    wasm_bindgen::{JsCast, JsValue},
};
use send_wrapper::SendWrapper;
use std::borrow::Cow;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExtractorMode {
    Pending,
    Extracting,
    Done,
}

pub struct DomExtractor {
    pub state: ExtractorState<FtmlDomElement>,
    pub context: NarrativeUri,
    pub mode: ExtractorMode,
    pub is_done: RwSignal<bool>,
    pub is_done_read: ReadSignal<bool>,
    pub is_stripped: bool,
}
impl DomExtractor {
    #[inline]
    pub fn new(uri: DocumentUri, context: NarrativeUri, is_stripped: bool) -> Self {
        let is_done = RwSignal::new(false);
        Self {
            state: ExtractorState::new(uri, false),
            context,
            mode: ExtractorMode::Pending,
            is_done_read: is_done.read_only(),
            is_done,
            is_stripped,
        }
    }

    pub fn finish(&mut self) -> Option<WriteSignal<bool>> {
        tracing::info!("Finishing extraction for {}", self.state.document);
        if self.state.document == *DocumentUri::no_doc() {
            return if self.mode == ExtractorMode::Done {
                None
            } else {
                Some(self.is_done.write_only())
            };
        }
        let ExtractionResult {
            document,
            modules,
            notations,
            ..
        } = self.state.finish();
        crate::utils::local_cache::LOCAL_CACHE
            .documents
            .insert(document);
        for m in modules {
            crate::utils::local_cache::LOCAL_CACHE.modules.insert(m);
        }
        for (uri, sol) in std::mem::take(&mut self.state.solutions) {
            crate::utils::local_cache::LOCAL_CACHE
                .solutions
                .insert(uri, sol);
        }
        for (sym, uri, not) in notations {
            match crate::utils::local_cache::LOCAL_CACHE.notations.entry(sym) {
                Entry::Vacant(v) => {
                    v.insert(vec![(uri, not)]);
                }
                Entry::Occupied(mut v) => {
                    let v = v.get_mut();
                    if !v.iter().any(|(u, _)| *u == uri) {
                        v.push((uri, not));
                    }
                }
            }
        }
        self.mode = ExtractorMode::Done;
        Some(self.is_done.write_only())
    }
}

static RULES: FtmlRuleSet<DomExtractor> = FtmlRuleSet::new();

impl FtmlStateExtractor for DomExtractor {
    type Attributes<'a> = NodeAttrs;
    type Return = Option<Marker>;
    type Node = FtmlDomElement;
    const RULES: &'static FtmlRuleSet<Self> = &RULES;
    const DO_RDF: bool = false;
    #[inline]
    fn state(&self) -> &ExtractorState<Self::Node> {
        &self.state
    }
    #[inline]
    fn state_mut(&mut self) -> &mut ExtractorState<Self::Node> {
        &mut self.state
    }
    #[inline]
    fn on_add(
        &mut self,
        elem: &ftml_parser::extraction::OpenFtmlElement,
    ) -> Result<Self::Return, FtmlExtractionError> {
        Ok(Marker::from(self, elem))
    }
    //type Node = ;
}

#[derive(Clone, Debug)]
pub struct FtmlDomElement(pub SendWrapper<Element>);
impl FtmlDomElement {
    #[inline]
    pub fn new(e: Element) -> Self {
        Self(SendWrapper::new(e))
    }
}
impl FtmlNode for FtmlDomElement {
    #[inline]
    fn delete(&self) {
        self.0.remove();
    }
    #[inline]
    fn range(&self) -> DocumentRange {
        DocumentRange::default()
    }
    #[inline]
    fn inner_range(&self) -> DocumentRange {
        DocumentRange::default()
    }
    #[inline]
    fn string(&self) -> Cow<'_, str> {
        self.0.outer_html().into()
    }
    #[inline]
    fn inner_string(&self) -> Cow<'_, str> {
        // TODO this can probably be done smarter, but we need to eliminate spurious comments
        let mut ret = String::new();
        for c in self.children() {
            match c {
                None => (),
                Some(Either::Right(r)) => ret.push_str(&r),
                Some(Either::Left(n)) => ret.push_str(&n.0.outer_html()),
            }
        }
        ret.into()
    }
    fn path_from(&self, ancestor: &Self) -> smallvec::SmallVec<u32, 4> {
        fn path_from_i(slf: &Element, ancestor: &Element) -> smallvec::SmallVec<u32, 4> {
            if slf == ancestor {
                return smallvec::SmallVec::new();
            }
            let p = slf.parent_element().expect("element has no parent??");
            let mut index = None;
            let mut i = 0;
            while let Some(e) = p.child_nodes().get(i) {
                if e == **slf {
                    index = Some(i);
                    break;
                }
                i += 1;
            }
            let index = index.expect("wut??");
            let mut ret = path_from_i(&p, ancestor);
            ret.push(index);
            ret
        }
        path_from_i(&self.0, &ancestor.0)
    }
    fn children(&self) -> impl Iterator<Item = Option<either::Either<Self, String>>> {
        struct NodeIter(NodeList, u32);
        impl Iterator for NodeIter {
            type Item = Option<Either<FtmlDomElement, String>>;
            fn next(&mut self) -> Option<Self::Item> {
                self.0.get(self.1).map(|n| {
                    self.1 += 1;
                    match n.dyn_into() {
                        Ok(e) => Some(Either::Left(FtmlDomElement(SendWrapper::new(e)))),
                        Err(n) if n.node_type() == 3 => n.text_content().map(Either::Right),
                        _ => None,
                    }
                })
            }
        }
        NodeIter(self.0.child_nodes(), 0)
    }
    #[inline]
    fn tag_name(&self) -> Result<Cow<'_, str>, String> {
        Ok(Cow::Owned(self.0.tag_name()))
    }
    fn iter_attributes(&self) -> impl Iterator<Item = Result<(Cow<'_, str>, String), String>> {
        struct AttrIter<'a>(leptos::web_sys::js_sys::Array, &'a Element, u32);
        impl<'a> Iterator for AttrIter<'a> {
            type Item = Result<(Cow<'a, str>, String), String>;
            fn next(&mut self) -> Option<Self::Item> {
                let next = self.0.get(self.2);
                if next.is_undefined() {
                    return None;
                }
                self.2 += 1;
                let Some(key) = next.as_string() else {
                    return Some(Err("invalid attribute".to_string()));
                };
                let Some(value) = self.1.get_attribute(key.as_ref()) else {
                    return Some(Err("invalid attribute".to_string()));
                };
                Some(Ok((Cow::Owned(key), value)))
            }
        }
        AttrIter(self.0.get_attribute_names(), &self.0, 0)
    }
}

pub struct NodeAttrs {
    elem: Element,
}

std::thread_local! {
    static PREFIX : std::cell::LazyCell<JsString> = std::cell::LazyCell::new(||
        JsValue::from_str(ftml_parser::PREFIX).dyn_into().expect("is valid string")
    );
}
#[allow(clippy::cast_possible_truncation)]
const PREFIX_LEN: u32 = ftml_parser::PREFIX.len() as u32;

impl NodeAttrs {
    pub(crate) fn new(elem: &Element) -> Self {
        Self { elem: elem.clone() }
    }

    pub(crate) fn keys(&self) -> KeyList {
        self.elem
            .get_attribute_names()
            .into_iter()
            .filter_map(|k| {
                k.dyn_ref::<JsString>().and_then(|s| {
                    if PREFIX.with(|p| s.slice(0, PREFIX_LEN) == **p) {
                        s.as_string().and_then(|str| {
                            FtmlKey::from_attr(&str).or_else(|| {
                                tracing::warn!("Unknown ftml attribute: {}", str);
                                #[cfg(any(feature = "csr", feature = "hydrate"))]
                                web_sys::console::error_1(&self.elem);
                                None
                            })
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}
impl Attributes for NodeAttrs {
    /*type KeyIter<'a>
        = std::iter::Copied<std::slice::Iter<'a, FtmlKey>>
    where
        Self: 'a;*/
    type Value<'a>
        = String
    where
        Self: 'a;
    type Ext = extractor::DomExtractor;
    /*fn keys(&self) -> Self::KeyIter<'_> {
        self.keys.iter().copied()
    }*/
    fn value(&self, key: &str) -> Option<Self::Value<'_>> {
        self.elem.get_attribute(key)
    }
    fn set(&mut self, key: &str, value: impl std::fmt::Display) {
        let _ = self.elem.set_attribute(key, &value.to_string());
    }
    fn take(&mut self, key: &str) -> Option<String> {
        let r = self.elem.get_attribute(key);
        let _ = self.elem.remove_attribute(key);
        r
    }
}
