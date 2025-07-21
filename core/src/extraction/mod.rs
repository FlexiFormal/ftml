use std::borrow::Cow;

use ftml_uris::{
    DocumentUri, Id, ModuleUri, NarrativeUriRef,
    errors::{SegmentParseError, UriParseError},
};

use crate::{
    FtmlKey,
    extraction::{nodes::FtmlNode, state::ExtractorState},
};

pub mod attributes;
pub mod nodes;
mod open;
pub(crate) mod rules;
pub mod state;
pub use open::*;

type Result<T> = std::result::Result<T, FtmlExtractionError>;

pub trait FtmlExtractor: 'static + Sized {
    type Attributes<'a>: attributes::Attributes<Ext = Self>;
    type Node<'n>: FtmlNode;
    const RULES: &'static FtmlRuleSet<Self>;
    const DO_RDF: bool;
    type Return;
    fn in_document(&self) -> &DocumentUri;
    fn iterate_domain(&self) -> impl Iterator<Item = &OpenDomainElement>;
    fn iterate_narrative(&self) -> impl Iterator<Item = &OpenNarrativeElement>;
    /// ### Errors
    fn add_element(&mut self, elem: OpenFtmlElement) -> Result<Self::Return>;
    /// ### Errors
    fn close(&mut self, elem: CloseFtmlElement, node: &Self::Node<'_>) -> Result<()>;
    /// ### Errors
    fn new_id(&mut self, prefix: impl Into<Cow<'static, str>>) -> Result<Id>;
    /// ### Errors
    fn get_domain_uri(&self, in_elem: FtmlKey) -> Result<&ModuleUri> {
        match self.iterate_domain().next() {
            Some(OpenDomainElement::Module { uri, .. }) => Ok(uri),
            Some(OpenDomainElement::Symbol { .. }) | None => {
                Err(FtmlExtractionError::NotInModule(in_elem))
            }
        }
    }

    fn get_narrative_uri(&self) -> NarrativeUriRef<'_> {
        self.iterate_narrative()
            .find_map(|e| match e {
                OpenNarrativeElement::Module { .. } => None,
                OpenNarrativeElement::Section { uri, .. } => Some(uri),
            })
            .map_or_else(
                || NarrativeUriRef::Document(self.in_document()),
                NarrativeUriRef::Element,
            )
    }
}
pub trait FtmlStateExtractor: 'static + Sized {
    type Attributes<'a>: attributes::Attributes<Ext = Self>;
    type Node<'n>: FtmlNode;
    const RULES: &'static FtmlRuleSet<Self>;
    const DO_RDF: bool;
    type Return;

    fn state_mut(&mut self) -> &mut ExtractorState;
    fn state(&self) -> &ExtractorState;
    /// ### Errors
    fn on_add(&mut self, elem: &OpenFtmlElement) -> Result<Self::Return>;
}
impl<E: FtmlStateExtractor> FtmlExtractor for E {
    type Attributes<'a> = <Self as FtmlStateExtractor>::Attributes<'a>;
    type Return = <Self as FtmlStateExtractor>::Return;
    type Node<'n> = <Self as FtmlStateExtractor>::Node<'n>;

    const RULES: &'static FtmlRuleSet<Self> = <Self as FtmlStateExtractor>::RULES;
    const DO_RDF: bool = <Self as FtmlStateExtractor>::DO_RDF;
    fn iterate_domain(&self) -> impl Iterator<Item = &OpenDomainElement> {
        self.state().domain().iter().rev()
    }
    fn iterate_narrative(&self) -> impl Iterator<Item = &OpenNarrativeElement> {
        self.state().narrative().iter().rev()
    }
    #[inline]
    fn new_id(&mut self, prefix: impl Into<Cow<'static, str>>) -> Result<Id> {
        self.state_mut().new_id(prefix)
    }
    #[inline]
    fn in_document(&self) -> &DocumentUri {
        self.state().in_document()
    }
    fn add_element(&mut self, elem: OpenFtmlElement) -> Result<Self::Return> {
        let r = self.on_add(&elem)?;
        self.state_mut().add(elem);
        Ok(r)
    }
    #[inline]
    fn close(&mut self, elem: CloseFtmlElement, node: &Self::Node<'_>) -> Result<()> {
        self.state_mut().close(elem, node)
    }
}

pub struct KeyList(pub(crate) smallvec::SmallVec<FtmlKey, 4>);
impl KeyList {
    #[inline]
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = FtmlKey> {
        self.0.iter().rev().copied()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn apply<'e, E: FtmlExtractor>(
        self,
        extractor: &'e mut E,
        attributes: &'e mut E::Attributes<'e>,
    ) -> impl Iterator<Item = Result<(E::Return, Option<CloseFtmlElement>)>> {
        struct AttrI<'e, E: FtmlExtractor>(KeyList, &'e mut E, &'e mut E::Attributes<'e>);
        impl<E: FtmlExtractor> Iterator for AttrI<'_, E> {
            type Item = Result<(E::Return, Option<CloseFtmlElement>)>;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let next = self.0.0.pop()?;
                Some((E::RULES.0[next as u8 as usize])(
                    self.1,
                    self.2,
                    &mut self.0,
                ))
            }
        }
        AttrI(self, extractor, attributes)
    }
}
impl FromIterator<FtmlKey> for KeyList {
    fn from_iter<T: IntoIterator<Item = FtmlKey>>(iter: T) -> Self {
        let mut ret = smallvec::SmallVec::new();
        for e in iter {
            if let Some(i) = ret.iter().enumerate().find_map(|(i, k)| {
                if (*k as u8) < (e as u8) {
                    Some(i)
                } else {
                    None
                }
            }) {
                ret.insert(i, e);
            } else {
                ret.push(e);
            }
        }
        Self(ret)
    }
}

/*
pub trait FtmlNode {
    type Ancestors<'a>: Iterator<Item = Self>
    where
        Self: 'a;
    fn ancestors(&self) -> Self::Ancestors<'_>;

    fn delete(&self);
    fn delete_children(&self);
    fn string(&self) -> String;
    fn inner_string(&self) -> String;
    /*
        fn with_elements<R>(&mut self, f: impl FnMut(Option<&mut FTMLElements>) -> R) -> R;
        fn range(&self) -> DocumentRange;
        fn inner_range(&self) -> DocumentRange;
        fn as_notation(&self) -> Option<NotationSpec>;
        fn as_op_notation(&self) -> Option<OpNotation>;
        fn as_term(&self) -> Term;
    */
}
 */

#[allow(clippy::type_complexity)]
pub struct FtmlRuleSet<E: FtmlExtractor>(
    pub(crate)  [fn(
        &mut E,
        &mut E::Attributes<'_>,
        &mut KeyList,
    ) -> Result<(E::Return, Option<CloseFtmlElement>)>;
        crate::keys::NUM_KEYS as usize],
);
impl<E: FtmlExtractor> FtmlRuleSet<E> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        crate::keys::FtmlKey::all_rules()
    }
}
impl<E: FtmlExtractor> Default for FtmlRuleSet<E> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FtmlExtractionError {
    #[error("`{0}` key missing in attributes")]
    MissingKey(FtmlKey),
    #[error("invalid language identifier: `{0}`")]
    InvalidLanguage(String),
    #[error("invalid uri: {0}")]
    Uri(#[from] UriParseError),
    #[error("key {0} not allowed outside of a module (or inside a declaration)")]
    NotInModule(FtmlKey),
    #[error("value for key {0} invalid")]
    InvalidValue(FtmlKey),
    #[error("{0} ended unexpectedly")]
    UnexpectedEndOf(FtmlKey),
}
impl From<SegmentParseError> for FtmlExtractionError {
    #[inline]
    fn from(value: SegmentParseError) -> Self {
        Self::Uri(value.into())
    }
}
