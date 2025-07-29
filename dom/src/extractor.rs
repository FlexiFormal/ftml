use crate::{extractor, markers::Marker};
use either::Either;
use ftml_core::{
    FtmlKey,
    extraction::{
        FtmlExtractionError, FtmlRuleSet, FtmlStateExtractor, KeyList, attributes::Attributes,
        nodes::FtmlNode, state::ExtractorState,
    },
};
use ftml_ontology::narrative::{
    DocumentRange,
    documents::{DocumentData, DocumentStyles},
};
use ftml_uris::{DocumentUri, NarrativeUri};
use leptos::wasm_bindgen::{JsCast, JsValue};
use leptos::web_sys::{Element, NodeList, js_sys::JsString};
use send_wrapper::SendWrapper;
use std::borrow::Cow;

pub enum ExtractorMode {
    Pending,
    Extracting,
    Done,
}

pub struct DomExtractor {
    pub state: ExtractorState<FtmlDomElement>,
    pub context: NarrativeUri,
    pub mode: ExtractorMode,
}
impl DomExtractor {
    #[inline]
    pub fn new(uri: DocumentUri, context: NarrativeUri) -> Self {
        Self {
            state: ExtractorState::new(uri, false),
            context,
            mode: ExtractorMode::Pending,
        }
    }

    pub fn finish(&mut self) {
        self.mode = ExtractorMode::Done;
        if self.state.document != *DocumentUri::no_doc() {
            let doc = DocumentData {
                uri: self.state.document.clone(),
                title: None, // todo
                elements: std::mem::take(&mut self.state.top).into_boxed_slice(),
                styles: DocumentStyles {
                    counters: std::mem::take(&mut self.state.counters).into_boxed_slice(),
                    styles: std::mem::take(&mut self.state.styles).into_boxed_slice(),
                },
            }
            .close();
            tracing::info!("Finished document {doc:?}");
            crate::utils::local_cache::LOCAL_CACHE.documents.insert(doc);
            for m in std::mem::take(&mut self.state.modules) {
                let m = m.close();
                tracing::info!("Found module {m:?}");
                crate::utils::local_cache::LOCAL_CACHE.modules.insert(m);
            }
        }
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
        elem: &ftml_core::extraction::OpenFtmlElement,
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
        JsValue::from_str(ftml_core::PREFIX).dyn_into().expect("is valid string")
    );
}
#[allow(clippy::cast_possible_truncation)]
const PREFIX_LEN: u32 = ftml_core::PREFIX.len() as u32;

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
    fn set(&mut self, key: &str, value: &str) {
        let _ = self.elem.set_attribute(key, value);
    }
    fn take(&mut self, key: &str) -> Option<String> {
        let r = self.elem.get_attribute(key);
        let _ = self.elem.remove_attribute(key);
        r
    }
}
