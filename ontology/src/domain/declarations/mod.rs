pub mod morphisms;
pub mod structures;
pub mod symbols;

use crate::domain::{
    declarations::{
        morphisms::Morphism,
        structures::{MathStructure, StructureExtension},
        symbols::Symbol,
    },
    modules::NestedModule,
};
use ftml_uris::{ModuleUri, SymbolUri};

pub trait IsDeclaration: crate::Ftml {
    fn uri(&self) -> Option<&SymbolUri>;
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self>;
    fn as_ref(&self) -> AnyDeclarationRef<'_>;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum AnyDeclaration {
    NestedModule(NestedModule),
    Import(ModuleUri),
    Symbol(Symbol),
    MathStructure(MathStructure),
    Morphism(Morphism),
    Extension(StructureExtension),
}

impl crate::__private::Sealed for AnyDeclaration {}
impl AnyDeclaration {
    #[inline]
    #[must_use]
    pub fn uri(&self) -> Option<&SymbolUri> {
        match self {
            Self::NestedModule(m) => m.uri(),
            Self::Symbol(s) => s.uri(),
            Self::MathStructure(s) => s.uri(),
            Self::Extension(e) => e.uri(),
            Self::Morphism(m) => m.uri(),
            Self::Import(_) => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn as_ref(&self) -> AnyDeclarationRef<'_> {
        match self {
            Self::NestedModule(m) => AnyDeclarationRef::NestedModule(m),
            Self::Symbol(s) => AnyDeclarationRef::Symbol(s),
            Self::MathStructure(s) => AnyDeclarationRef::MathStructure(s),
            Self::Extension(e) => AnyDeclarationRef::Extension(e),
            Self::Morphism(m) => AnyDeclarationRef::Morphism(m),
            Self::Import(i) => AnyDeclarationRef::Import(i),
        }
    }
}
impl crate::Ftml for AnyDeclaration {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        #[allow(clippy::enum_glob_use)]
        use either_of::EitherOf6::*;
        match self {
            Self::NestedModule(m) => A(m.triples().into_iter()),
            Self::Symbol(s) => B(s.triples().into_iter()),
            Self::MathStructure(s) => C(s.triples().into_iter()),
            Self::Extension(e) => D(e.triples().into_iter()),
            Self::Morphism(m) => E(m.triples().into_iter()),
            Self::Import(_) => F(std::iter::empty()),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum AnyDeclarationRef<'d> {
    NestedModule(&'d NestedModule),
    Import(&'d ModuleUri),
    Symbol(&'d Symbol),
    MathStructure(&'d MathStructure),
    Morphism(&'d Morphism),
    Extension(&'d StructureExtension),
}

impl crate::__private::Sealed for AnyDeclarationRef<'_> {}
impl<'d> AnyDeclarationRef<'d> {
    #[inline]
    #[must_use]
    pub fn uri(&self) -> Option<&'d SymbolUri> {
        match self {
            Self::NestedModule(m) => m.uri(),
            Self::Symbol(s) => s.uri(),
            Self::MathStructure(s) => s.uri(),
            Self::Extension(e) => e.uri(),
            Self::Morphism(m) => m.uri(),
            Self::Import(_) => None,
        }
    }
}
impl crate::Ftml for AnyDeclarationRef<'_> {
    #[cfg(feature = "rdf")]
    #[allow(clippy::enum_glob_use)]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use either_of::EitherOf6::*;
        match self {
            Self::NestedModule(m) => A(m.triples().into_iter()),
            Self::Symbol(s) => B(s.triples().into_iter()),
            Self::MathStructure(s) => C(s.triples().into_iter()),
            Self::Extension(e) => D(e.triples().into_iter()),
            Self::Morphism(m) => E(m.triples().into_iter()),
            Self::Import(_) => F(std::iter::empty()),
        }
    }
}
