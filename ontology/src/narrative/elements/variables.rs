use ftml_uris::{DocumentElementUri, Id};

use crate::{
    domain::declarations::symbols::{ArgumentSpec, AssocType},
    narrative::{
        Narrative,
        elements::{DocumentElementRef, IsDocumentElement},
    },
    terms::Term,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VariableDeclaration {
    pub uri: DocumentElementUri,
    pub data: Box<VariableData>,
}
impl crate::__private::Sealed for VariableDeclaration {}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VariableData {
    pub arity: ArgumentSpec,
    pub macroname: Option<Id>,
    pub role: Box<[Id]>,
    pub tp: Option<Term>,
    pub df: Option<Term>,
    pub bind: bool,
    pub assoctype: Option<AssocType>,
    pub reordering: Option<Id>,
    pub is_seq: bool,
}
impl crate::Ftml for VariableDeclaration {
    #[cfg(feature = "rdf")]
    #[allow(clippy::enum_glob_use)]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
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
        match (&self.data.tp, &self.data.df) {
            (Some(Term::Symbol(tp)), Some(df)) => A(syms!(df).chain([
                triple!(<(iri.clone())> : ulo:variable),
                triple!(<(iri)> ulo:has_type  <(tp.to_iri())>),
            ])),
            (Some(tp), Some(df)) => B(syms!(tp)
                .chain(syms!(df))
                .chain(std::iter::once(triple!(<(iri)> : ulo:variable)))),
            (Some(Term::Symbol(tp)), _) => C([
                triple!(<(iri.clone())> : ulo:variable),
                triple!(<(iri)> ulo:has_type  <(tp.to_iri())>),
            ]
            .into_iter()),
            (Some(tp), _) => D(syms!(tp).chain(std::iter::once(triple!(<(iri)> : ulo:variable)))),
            (_, Some(df)) => E(syms!(df).chain(std::iter::once(triple!(<(iri)> : ulo:variable)))),
            (None, None) => F(std::iter::once(triple!(<(iri)> : ulo:variable))),
        }
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
