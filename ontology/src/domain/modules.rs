use ftml_uris::{Language, ModuleUri, SymbolUri};

use crate::domain::{
    HasDeclarations,
    declarations::{AnyDeclaration, AnyDeclarationRef, IsDeclaration},
};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ModuleData {
    pub uri: ModuleUri,
    pub meta_module: Option<ModuleUri>,
    pub signature: Option<Language>,
    pub declarations: Box<[AnyDeclaration]>,
}
impl crate::__private::Sealed for ModuleData {}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Module(triomphe::Arc<ModuleData>);
impl std::ops::Deref for Module {
    type Target = ModuleData;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl HasDeclarations for ModuleData {
    #[inline]
    fn declarations(&self) -> &[AnyDeclaration] {
        &self.declarations
    }
    #[inline]
    fn domain_uri(&self) -> ftml_uris::DomainUriRef<'_> {
        ftml_uris::DomainUriRef::Module(&self.uri)
    }
}
impl crate::Ftml for ModuleData {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use arrayvec::ArrayVec;
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();

        let mut others = ArrayVec::<_, 3>::new();
        if let Some(meta) = &self.meta_module {
            others.push(triple!(<(iri.clone())> ulo:has_meta_theory <(meta.to_iri())>));
        }
        if let Some(sig) = &self.signature {
            others.push(triple!(<(iri.clone())> ulo:has_signature = (sig.to_string())));
        }
        others.push(triple!(<(iri)> : ulo:theory));
        others.into_iter().chain(self.declares_triples())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct NestedModule {
    pub uri: SymbolUri,
    pub declarations: Box<[AnyDeclaration]>,
}
impl crate::__private::Sealed for NestedModule {}
impl crate::Ftml for NestedModule {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();

        std::iter::once(triple!(<(iri)> : ulo:theory)).chain(self.declares_triples())
    }
}
impl HasDeclarations for NestedModule {
    #[inline]
    fn declarations(&self) -> &[AnyDeclaration] {
        &self.declarations
    }
    #[inline]
    fn domain_uri(&self) -> ftml_uris::DomainUriRef<'_> {
        ftml_uris::DomainUriRef::Symbol(&self.uri)
    }
}
impl IsDeclaration for NestedModule {
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        Some(&self.uri)
    }
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        if let AnyDeclarationRef::NestedModule(m) = decl {
            Some(m)
        } else {
            None
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::NestedModule(self)
    }
}
