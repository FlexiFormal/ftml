use ftml_uris::{DomainUriRef, Id, ModuleUri, SimpleUriName, SymbolUri};

use crate::{
    domain::{
        HasDeclarations,
        declarations::{AnyDeclarationRef, IsDeclaration},
    },
    terms::Term,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Morphism {
    pub uri: SymbolUri,
    pub domain: ModuleUri,
    pub total: bool,
    pub elements: Box<[Assignment]>,
}

impl crate::__private::Sealed for Morphism {}
impl crate::Ftml for Morphism {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;

        let iri = self.uri.to_iri();
        [
            triple!(<(iri.clone())> : ulo:morphism),
            triple!(<(iri)> rdfs:DOMAIN <(self.domain.to_iri())>),
        ]
        .into_iter()
        .chain(self.declares_triples())
    }
}
impl IsDeclaration for Morphism {
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        Some(&self.uri)
    }
    #[inline]
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        match decl {
            AnyDeclarationRef::Morphism(m) => Some(m),
            _ => None,
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::Morphism(self)
    }
}
impl HasDeclarations for Morphism {
    #[inline]
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        std::iter::empty() //self.elements.iter().map(Declaration::as_ref)
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Assignment {
    pub original: SymbolUri,
    pub morphism: SymbolUri,
    #[cfg_attr(feature = "serde", serde(default))]
    pub definiens: Option<Term>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub refined_type: Option<Term>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub new_name: Option<SimpleUriName>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub macroname: Option<Id>,
}
impl Assignment {
    #[must_use]
    pub fn elaborated_uri(&self) -> SymbolUri {
        self.new_name.as_ref().map_or_else(
            || {
                // SAFETY: segment already validated
                unsafe {
                    self.morphism.clone() / &self.original.name.last().parse().unwrap_unchecked()
                }
            },
            |name| self.morphism.module.clone() | name.clone(),
        )
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Assignment {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.definiens
            .as_ref()
            .map(|t| t.deep_size_of_children(context))
            .unwrap_or_default()
            + self
                .refined_type
                .as_ref()
                .map(|t| t.deep_size_of_children(context))
                .unwrap_or_default()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Morphism {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.elements
            .iter()
            .map(|v| std::mem::size_of_val(v) + v.deep_size_of_children(context))
            .sum::<usize>()
    }
}
