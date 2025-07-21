use ftml_uris::{DomainUriRef, ModuleUri, SymbolUri};

use crate::domain::{
    HasDeclarations,
    declarations::{AnyDeclaration, AnyDeclarationRef, IsDeclaration},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Morphism {
    pub uri: SymbolUri,
    pub domain: ModuleUri,
    pub total: bool,
    pub elements: Box<[AnyDeclaration]>,
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
    fn declarations(&self) -> &[AnyDeclaration] {
        &self.elements
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }
}
