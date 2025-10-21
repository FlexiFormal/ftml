use std::borrow::Borrow;

use ftml_uris::{Language, ModuleUri, SymbolUri};

use crate::domain::{
    HasDeclarations, SharedDeclaration,
    declarations::{
        AnyDeclarationRef, Declaration, IsDeclaration,
        morphisms::Morphism,
        structures::{MathStructure, StructureExtension},
    },
};

#[derive(Clone, Hash, Debug)]
pub enum ModuleLike {
    Module(Module),
    Structure(SharedDeclaration<MathStructure>),
    Extension(SharedDeclaration<StructureExtension>),
    Nested(SharedDeclaration<NestedModule>),
    Morphism(SharedDeclaration<Morphism>),
}
impl From<Module> for ModuleLike {
    #[inline]
    fn from(value: Module) -> Self {
        Self::Module(value)
    }
}
impl From<SharedDeclaration<MathStructure>> for ModuleLike {
    #[inline]
    fn from(value: SharedDeclaration<MathStructure>) -> Self {
        Self::Structure(value)
    }
}
impl From<SharedDeclaration<StructureExtension>> for ModuleLike {
    #[inline]
    fn from(value: SharedDeclaration<StructureExtension>) -> Self {
        Self::Extension(value)
    }
}
impl From<SharedDeclaration<Morphism>> for ModuleLike {
    #[inline]
    fn from(value: SharedDeclaration<Morphism>) -> Self {
        Self::Morphism(value)
    }
}
impl crate::__private::Sealed for ModuleLike {}
impl HasDeclarations for ModuleLike {
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        use either_of::EitherOf5::{A, B, C, D, E};
        match self {
            Self::Module(m) => A(m.declarations()),
            Self::Structure(s) => B(s.declarations()),
            Self::Extension(e) => C(e.declarations()),
            Self::Morphism(m) => D(m.declarations()),
            Self::Nested(n) => E(n.declarations()),
        }
    }
    fn domain_uri(&self) -> ftml_uris::DomainUriRef<'_> {
        match self {
            Self::Module(m) => ftml_uris::DomainUriRef::Module(&m.uri),
            Self::Structure(s) => ftml_uris::DomainUriRef::Symbol(&s.uri),
            Self::Extension(s) => ftml_uris::DomainUriRef::Symbol(&s.uri),
            Self::Morphism(s) => ftml_uris::DomainUriRef::Symbol(&s.uri),
            Self::Nested(s) => ftml_uris::DomainUriRef::Symbol(&s.uri),
        }
    }
}
impl crate::Ftml for ModuleLike {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use either_of::EitherOf5::{A, B, C, D, E};
        match self {
            Self::Module(m) => A(m.triples().into_iter()),
            Self::Structure(s) => B(s.triples().into_iter()),
            Self::Extension(e) => C(e.triples().into_iter()),
            Self::Morphism(m) => D(m.triples().into_iter()),
            Self::Nested(n) => E(n.triples().into_iter()),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ModuleData {
    pub uri: ModuleUri,
    #[cfg_attr(feature = "serde", serde(default))]
    pub meta_module: Option<ModuleUri>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub signature: Option<Language>,
    pub declarations: Box<[Declaration]>,
}
impl crate::__private::Sealed for ModuleData {}
impl ModuleData {
    #[inline]
    #[must_use]
    pub fn close(self) -> Module {
        Module(triomphe::Arc::new(self))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module(pub(crate) triomphe::Arc<ModuleData>);
impl std::ops::Deref for Module {
    type Target = ModuleData;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::hash::Hash for Module {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}
impl Borrow<ModuleUri> for Module {
    #[inline]
    fn borrow(&self) -> &ModuleUri {
        &self.uri
    }
}

impl HasDeclarations for ModuleData {
    #[inline]
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        self.declarations.iter().map(Declaration::as_ref)
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct NestedModule {
    pub uri: SymbolUri,
    pub declarations: Box<[Declaration]>,
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
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        self.declarations.iter().map(Declaration::as_ref)
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

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::domain::modules::ModuleData;

    impl<Context> bincode::Decode<Context> for super::Module {
        fn decode<D: bincode::de::Decoder<Context = Context>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            ModuleData::decode(decoder).map(|d| Self(triomphe::Arc::new(d)))
        }
    }
    impl bincode::Encode for super::Module {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            self.0.encode(encoder)
        }
    }

    impl serde::Serialize for super::Module {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }
    impl<'de> serde::Deserialize<'de> for super::Module {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            ModuleData::deserialize(deserializer).map(|d| Self(triomphe::Arc::new(d)))
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for ModuleData {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.declarations
            .iter()
            .map(|v| std::mem::size_of_val(v) + v.deep_size_of_children(context))
            .sum::<usize>()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Module {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        std::mem::size_of::<ModuleData>() + self.0.deep_size_of_children(context)
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for NestedModule {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.declarations
            .iter()
            .map(|v| std::mem::size_of_val(v) + v.deep_size_of_children(context))
            .sum::<usize>()
    }
}
