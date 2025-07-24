use std::borrow::Cow;

use ftml_ontology::{
    narrative::elements::{DocumentElement, VariableDeclaration},
    terms::Variable,
};
use ftml_uris::{
    DocumentUri, Id, ModuleUri, NarrativeUriRef, UriName,
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
    fn new_id(&mut self, key: FtmlKey, prefix: impl Into<Cow<'static, str>>) -> Result<Id>;
    /// ### Errors
    fn get_domain_uri(&self, in_elem: FtmlKey) -> Result<&ModuleUri> {
        match self.iterate_domain().next() {
            Some(OpenDomainElement::Module { uri, .. }) => Ok(uri),
            Some(
                OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. },
            )
            | None => Err(FtmlExtractionError::NotIn(
                in_elem,
                "a module (or inside a declaration)",
            )),
        }
    }

    fn get_narrative_uri(&self) -> NarrativeUriRef<'_> {
        self.iterate_narrative()
            .find_map(|e| match e {
                OpenNarrativeElement::Module { .. } | OpenNarrativeElement::SkipSection { .. } => {
                    None
                }
                OpenNarrativeElement::Section { uri, .. } => Some(uri),
            })
            .map_or_else(
                || NarrativeUriRef::Document(self.in_document()),
                NarrativeUriRef::Element,
            )
    }

    fn in_notation(&self) -> bool {
        match self.iterate_domain().next() {
            None
            | Some(
                OpenDomainElement::Module { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. },
            ) => false,
        }
        /*
        for n in self.iterate_domain() {
            match n {
                OpenDomainElement::Symbol { .. } | OpenDomainElement::Module { .. } => {
                    return false;
                }
            }
        }
        false
         */
    }

    fn in_term(&self) -> bool {
        match self.iterate_domain().next() {
            None
            | Some(
                OpenDomainElement::Module { .. } | OpenDomainElement::SymbolDeclaration { .. },
            ) => false,
            Some(OpenDomainElement::SymbolReference { .. }) => true,
        }
    }

    fn resolve_variable_name(&self, name: UriName) -> Variable {
        fn ew(a: &UriName, b: &UriName) -> bool {
            let mut steps = a.steps().rev();
            for s in b.steps().rev() {
                if steps.next() != Some(s) {
                    return false;
                }
            }
            true
        }
        for n in self.iterate_narrative() {
            let ch = match n {
                OpenNarrativeElement::Module { children, .. }
                | OpenNarrativeElement::Section { children, .. }
                | OpenNarrativeElement::SkipSection { children } => children,
                // Narrative::Notation(_) => continue,
            };
            for c in ch.iter().rev() {
                match c {
                    DocumentElement::VariableDeclaration(VariableDeclaration { uri, data })
                        if ew(uri.name(), &name) =>
                    {
                        return Variable::Ref {
                            declaration: uri.clone(),
                            is_sequence: Some(data.is_seq),
                        };
                    }
                    _ => (),
                }
            }
        }
        Variable::Name(name)
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
    #[inline]
    fn iterate_domain(&self) -> impl Iterator<Item = &OpenDomainElement> {
        self.state().domain()
    }
    #[inline]
    fn iterate_narrative(&self) -> impl Iterator<Item = &OpenNarrativeElement> {
        self.state().narrative()
    }
    #[inline]
    fn new_id(&mut self, key: FtmlKey, prefix: impl Into<Cow<'static, str>>) -> Result<Id> {
        self.state_mut().new_id(key, prefix)
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
    InvalidLanguage(FtmlKey, String),
    #[error("invalid uri in {0}: {1}")]
    Uri(FtmlKey, #[source] UriParseError),
    #[error("key {0} not allowed outside of {1}")]
    NotIn(FtmlKey, &'static str),
    #[error("value for key {0} invalid")]
    InvalidValue(FtmlKey),
    #[error("{0} ended unexpectedly")]
    UnexpectedEndOf(FtmlKey),
    #[error("duplicate property: {0}")]
    DuplicateValue(FtmlKey),
}
impl From<(FtmlKey, Self)> for FtmlExtractionError {
    #[inline]
    fn from(value: (FtmlKey, Self)) -> Self {
        value.1
    }
}
impl From<(FtmlKey, ())> for FtmlExtractionError {
    #[inline]
    fn from(value: (FtmlKey, ())) -> Self {
        Self::InvalidValue(value.0)
    }
}
impl From<(FtmlKey, SegmentParseError)> for FtmlExtractionError {
    #[inline]
    fn from(p: (FtmlKey, SegmentParseError)) -> Self {
        Self::Uri(p.0, p.1.into())
    }
}

impl From<(FtmlKey, UriParseError)> for FtmlExtractionError {
    #[inline]
    fn from(p: (FtmlKey, UriParseError)) -> Self {
        Self::Uri(p.0, p.1)
    }
}
