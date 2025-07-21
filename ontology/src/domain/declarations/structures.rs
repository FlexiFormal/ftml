use ftml_uris::{DomainUriRef, Id, SymbolUri};

use crate::domain::{
    HasDeclarations,
    declarations::{AnyDeclaration, AnyDeclarationRef, IsDeclaration},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct MathStructure {
    pub uri: SymbolUri,
    pub elements: Box<[AnyDeclaration]>,
    pub macroname: Option<Id>,
}
impl crate::__private::Sealed for MathStructure {}
impl crate::Ftml for MathStructure {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;

        let iri = self.uri.to_iri();
        std::iter::once(triple!(<(iri)> : ulo:structure)).chain(self.declares_triples())
    }
}
impl IsDeclaration for MathStructure {
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        Some(&self.uri)
    }
    #[inline]
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        match decl {
            AnyDeclarationRef::MathStructure(m) => Some(m),
            _ => None,
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::MathStructure(self)
    }
}
impl HasDeclarations for MathStructure {
    #[inline]
    fn declarations(&self) -> &[AnyDeclaration] {
        &self.elements
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct StructureExtension {
    pub uri: SymbolUri,
    pub target: SymbolUri,
    pub elements: Box<[AnyDeclaration]>,
}
impl crate::__private::Sealed for StructureExtension {}
impl crate::Ftml for StructureExtension {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;

        let iri = self.uri.to_iri();
        let iri2 = self.uri.to_iri();
        let target = self.target.to_iri();
        self.elements
            .iter()
            .filter_map(move |e| {
                e.uri()
                    .map(|e| triple!(<(e.to_iri())> ulo:declares <(iri2.clone())>))
            })
            .chain([
                triple!(<(iri.clone())> : ulo:structure),
                triple!(<(iri)> ulo:extends <(target)>),
            ])
    }
}
impl IsDeclaration for StructureExtension {
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        Some(&self.uri)
    }
    #[inline]
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        match decl {
            AnyDeclarationRef::Extension(m) => Some(m),
            _ => None,
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::Extension(self)
    }
}
impl HasDeclarations for StructureExtension {
    #[inline]
    fn declarations(&self) -> &[AnyDeclaration] {
        &self.elements
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }
}
