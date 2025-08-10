use crate::domain::{
    HasDeclarations,
    declarations::{AnyDeclarationRef, IsDeclaration, morphisms::Morphism, symbols::Symbol},
};
use ftml_uris::{DomainUriRef, Id, ModuleUri, SymbolUri};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct MathStructure {
    pub uri: SymbolUri,
    pub elements: Box<[StructureDeclaration]>,
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
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        self.elements.iter().map(StructureDeclaration::as_ref)
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum StructureDeclaration {
    Import(ModuleUri),
    Symbol(Symbol),
    Morphism(Morphism),
}
impl crate::__private::Sealed for StructureDeclaration {}
impl crate::Ftml for StructureDeclaration {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        #[allow(clippy::enum_glob_use)]
        use either_of::EitherOf3::*;
        match self {
            Self::Symbol(s) => A(s.triples().into_iter()),
            Self::Morphism(m) => B(m.triples().into_iter()),
            Self::Import(_) => C(std::iter::empty()),
        }
    }
}
impl IsDeclaration for StructureDeclaration {
    #[inline]
    fn from_declaration(_: AnyDeclarationRef<'_>) -> Option<&Self> {
        None
    }
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        match self {
            Self::Symbol(s) => Some(&s.uri),
            Self::Morphism(m) => Some(&m.uri),
            Self::Import(_) => None,
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        match self {
            Self::Import(u) => AnyDeclarationRef::Import(u),
            Self::Symbol(s) => AnyDeclarationRef::Symbol(s),
            Self::Morphism(m) => AnyDeclarationRef::Morphism(m),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct StructureExtension {
    pub uri: SymbolUri,
    pub target: SymbolUri,
    pub elements: Box<[StructureDeclaration]>,
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
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        self.elements.iter().map(StructureDeclaration::as_ref)
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for StructureDeclaration {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Symbol(s) => s.deep_size_of_children(context),
            Self::Morphism(m) => m.deep_size_of_children(context),
            Self::Import(_) => 0,
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for MathStructure {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.elements
            .iter()
            .map(|v| std::mem::size_of_val(v) + v.deep_size_of_children(context))
            .sum::<usize>()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for StructureExtension {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.elements
            .iter()
            .map(|v| std::mem::size_of_val(v) + v.deep_size_of_children(context))
            .sum::<usize>()
    }
}
