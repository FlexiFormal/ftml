use ftml_uris::{DocumentElementUri, Id};

use crate::{
    domain::declarations::symbols::{ArgumentSpec, AssocType},
    narrative::{
        Narrative,
        elements::{DocumentElementRef, IsDocumentElement},
    },
    terms::{Term, TermContainer},
    utils::{Permutation, SourceRange},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VariableDeclaration {
    pub uri: DocumentElementUri,
    pub data: Box<VariableData>,
}
impl crate::__private::Sealed for VariableDeclaration {}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VariableData {
    pub arity: ArgumentSpec,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub macroname: Option<Id>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub role: Box<[Id]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub tp: TermContainer,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub df: TermContainer,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub bind: bool,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub assoctype: Option<AssocType>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub reordering: Option<Permutation>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub argument_types: Box<[Term]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub return_type: Option<Term>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub is_seq: bool,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub source: SourceRange,
}
impl crate::Ftml for VariableDeclaration {
    #[cfg(feature = "rdf")]
    #[allow(clippy::enum_glob_use)]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use crate::terms::IsTerm;
        use either_of::EitherOf6::*;
        use ftml_uris::FtmlUri;
        use rustc_hash::FxHashSet;
        use ulo::triple;
        let iri = self.uri.to_iri();
        macro_rules! syms {
            ($e:expr) => {{
                let iri2 = iri.clone();
                $e.symbols().collect::<FxHashSet<_>>().into_iter()
                    .map(move |s| triple!(<(iri2.clone())> dc:hasPart <(s.to_iri())>))
            }};
        }
        match (self.data.tp.parsed(), self.data.df.parsed()) {
            (Some(Term::Symbol { uri: tp, .. }), Some(df)) => A(syms!(df).chain([
                triple!(<(iri.clone())> : ulo:variable),
                triple!(<(iri)> ulo:has_type  <(tp.to_iri())>),
            ])),
            (Some(tp), Some(df)) => B(syms!(tp)
                .chain(syms!(df))
                .chain(std::iter::once(triple!(<(iri)> : ulo:variable)))),
            (Some(Term::Symbol { uri: tp, .. }), _) => C([
                triple!(<(iri.clone())> : ulo:variable),
                triple!(<(iri)> ulo:has_type  <(tp.to_iri())>),
            ]
            .into_iter()),
            (Some(tp), _) => D(syms!(tp).chain(std::iter::once(triple!(<(iri)> : ulo:variable)))),
            (_, Some(df)) => E(syms!(df).chain(std::iter::once(triple!(<(iri)> : ulo:variable)))),
            (None, None) => F(std::iter::once(triple!(<(iri)> : ulo:variable))),
        }
    }
    #[inline]
    fn source_range(&self) -> SourceRange {
        self.data.source
    }
}
impl Narrative for VariableDeclaration {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        Some(ftml_uris::NarrativeUriRef::Element(&self.uri))
    }
    #[inline]
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        std::iter::empty()
    }
}
impl IsDocumentElement for VariableDeclaration {
    #[inline]
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
    #[inline]
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::VariableDeclaration(self)
    }
    #[inline]
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        match e {
            DocumentElementRef::VariableDeclaration(p) => Some(p),
            _ => None,
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for VariableData {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        (self.role.len() * std::mem::size_of::<Id>())
            + self.tp.deep_size_of_children(context)
            + self.df.deep_size_of_children(context)
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for VariableDeclaration {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        std::mem::size_of::<VariableData>() + (*self.data).deep_size_of_children(context)
    }
}
