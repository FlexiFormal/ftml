use std::borrow::Borrow;

use ftml_uris::{Language, ModuleUri, SymbolUri};

use crate::{
    domain::{
        HasDeclarations, SharedDeclaration,
        declarations::{
            AnyDeclarationRef, Declaration, IsDeclaration,
            morphisms::Morphism,
            structures::{MathStructure, StructureExtension},
        },
    },
    utils::SharedArc,
};

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum ModuleLike {
    Module(Module),
    Structure(SharedDeclaration<MathStructure>),
    Extension(SharedDeclaration<StructureExtension>),
    Nested(SharedDeclaration<NestedModule>),
    Morphism(SharedDeclaration<Morphism>),
}
impl ModuleLike {
    pub fn get_as<T: IsDeclaration>(
        &self,
        name: &ftml_uris::UriName,
    ) -> Option<SharedDeclaration<T>> {
        match self {
            Self::Module(m) => m.get_as(name),
            Self::Structure(s) => {
                SharedArc::inherit::<_, _>(s.clone().0, |s| s.find(name.steps()).ok_or(()))
                    .ok()
                    .map(SharedDeclaration)
            }
            Self::Extension(s) => {
                SharedArc::inherit::<_, _>(s.clone().0, |s| s.find(name.steps()).ok_or(()))
                    .ok()
                    .map(SharedDeclaration)
            }
            Self::Nested(s) => {
                SharedArc::inherit::<_, _>(s.clone().0, |s| s.find(name.steps()).ok_or(()))
                    .ok()
                    .map(SharedDeclaration)
            }
            Self::Morphism(s) => {
                SharedArc::inherit::<_, _>(s.clone().0, |s| s.find(name.steps()).ok_or(()))
                    .ok()
                    .map(SharedDeclaration)
            }
        }
    }
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
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ModuleData {
    pub uri: ModuleUri,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub meta_module: Option<ModuleUri>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
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
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
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

#[cfg(feature = "serde-lite")]
mod serde_lt_impl {
    use crate::domain::modules::ModuleData;
    use ftml_uris::UriName;

    impl serde_lite::Serialize for super::Module {
        #[inline]
        fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
            self.0.serialize()
        }
    }
    impl serde_lite::Deserialize for super::Module {
        fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error>
        where
            Self: Sized,
        {
            Ok(Self(triomphe::Arc::new(ModuleData::deserialize(val)?)))
        }
    }
    impl serde_lite::Serialize for super::ModuleLike {
        fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
            match self {
                Self::Module(m) => (m, None::<&UriName>).serialize(),
                Self::Structure(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize()
                }
                Self::Extension(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize()
                }
                Self::Nested(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize()
                }
                Self::Morphism(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize()
                }
            }
        }
    }
    impl serde_lite::Deserialize for super::ModuleLike {
        fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error>
        where
            Self: Sized,
        {
            type T = (super::Module, Option<UriName>);
            let (m, o) = T::deserialize(val)?;
            if let Some(name) = o {
                m.as_module_like(&name).ok_or_else(move || {
                    serde_lite::Error::custom(format_args!(
                        "module does not contain element named {name}"
                    ))
                })
            } else {
                Ok(Self::Module(m))
            }
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use ftml_uris::UriName;

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

    impl serde::Serialize for super::ModuleLike {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Module(m) => (m, None::<&UriName>).serialize(serializer),
                Self::Structure(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize(serializer)
                }
                Self::Extension(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize(serializer)
                }
                Self::Nested(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize(serializer)
                }
                Self::Morphism(s) => {
                    (s.0.outer(), Some(s.uri.clone().simple_module().name())).serialize(serializer)
                }
            }
        }
    }
    impl<'de> serde::Deserialize<'de> for super::ModuleLike {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            type T = (super::Module, Option<UriName>);
            let (m, o) = T::deserialize(deserializer)?;
            if let Some(name) = o {
                m.as_module_like(&name).ok_or_else(move || {
                    serde::de::Error::custom(format_args!(
                        "module does not contain element named {name}"
                    ))
                })
            } else {
                Ok(Self::Module(m))
            }
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for ModuleLike {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Module(m) => m.deep_size_of_children(context),
            Self::Nested(n) => n.deep_size_of_children(context),
            Self::Structure(s) => s.deep_size_of_children(context),
            Self::Extension(e) => e.deep_size_of_children(context),
            Self::Morphism(m) => m.deep_size_of_children(context),
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
