use ftml_core::{
    FtmlKey,
    extraction::{
        FtmlExtractionError, FtmlRuleSet, FtmlStateExtractor, KeyList, attributes::Attributes,
        nodes::FtmlNode, state::ExtractorState,
    },
};
use ftml_ontology::narrative::DocumentRange;
use ftml_uris::DocumentUri;
use leptos::wasm_bindgen::{JsCast, JsValue};
use leptos::web_sys::{Element, js_sys::JsString};

use crate::{extractor, markers::Marker};

pub struct DomExtractor {
    pub(crate) state: ExtractorState,
}
impl DomExtractor {
    #[inline]
    pub fn new(uri: DocumentUri) -> Self {
        Self {
            state: ExtractorState::new(uri, false),
        }
    }
}

static RULES: FtmlRuleSet<DomExtractor> = FtmlRuleSet::new();

impl FtmlStateExtractor for DomExtractor {
    type Attributes<'a> = NodeAttrs;
    type Return = Option<Marker>;
    type Node<'n> = FtmlDomElement<'n>;
    const RULES: &'static FtmlRuleSet<Self> = &RULES;
    const DO_RDF: bool = false;
    #[inline]
    fn state(&self) -> &ExtractorState {
        &self.state
    }
    #[inline]
    fn state_mut(&mut self) -> &mut ExtractorState {
        &mut self.state
    }
    #[inline]
    fn on_add(
        &mut self,
        elem: &ftml_core::extraction::OpenFtmlElement,
    ) -> Result<Self::Return, FtmlExtractionError> {
        Ok(Marker::from(elem))
    }
    //type Node = ;
}

#[derive(Clone)]
pub struct FtmlDomElement<'n>(pub(crate) &'n Element);
impl FtmlNode for FtmlDomElement<'_> {
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
