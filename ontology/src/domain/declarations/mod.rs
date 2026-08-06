pub mod morphisms;
pub mod structures;
pub mod symbols;

use crate::{
    domain::{
        DeclIter, HasDeclarations, SharedDeclaration,
        declarations::{
            morphisms::Morphism,
            structures::{MathStructure, StructureExtension},
            symbols::Symbol,
        },
        modules::NestedModule,
    },
    terms::Term,
    utils::{SourceRange, TreeChild},
};
use ftml_uris::{Id, ModuleUri, SymbolUri};

pub trait IsDeclaration: crate::Ftml {
    fn uri(&self) -> Option<&SymbolUri>;
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self>;
    fn as_ref(&self) -> AnyDeclarationRef<'_>;
    /*fn elaborated_from(&self) -> Option<&SymbolUri>;
    #[inline]
    fn is_primitive(&self) -> bool {
        self.elaborated_from().is_none()
    }*/
}

pub trait IsSymbol: crate::Ftml {
    fn symbol_uri(&self) -> &SymbolUri;
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self>;
    fn as_decl(&self) -> AnyDeclarationRef<'_>;
}
impl<T: IsSymbol> IsDeclaration for T {
    fn uri(&self) -> Option<&SymbolUri> {
        Some(IsSymbol::symbol_uri(self))
    }
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        IsSymbol::from_declaration(decl)
    }
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        IsSymbol::as_decl(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SharedSymbolLike {
    Symbol(SharedDeclaration<Symbol>),
    MathStructure(SharedDeclaration<MathStructure>),
    Extension(SharedDeclaration<StructureExtension>),
    Morphism(SharedDeclaration<Morphism>),
}
/*
impl TryFrom<Declaration> for SharedSymbolLike {
    type Error = ();
    fn try_from(value: Declaration) -> Result<Self, Self::Error> {
        match value {
            Declaration::Symbol(s) => Ok(Self::Symbol(s)),
            Declaration::MathStructure(s) => Ok(Self::MathStructure(s)),
            Declaration::Extension(s) => Ok(Self::Extension(s)),
            Declaration::Morphism(s) => Ok(Self::Morphism(s)),
            _ => Err(()),
        }
    }
}
 */
#[derive(Debug, Clone)]
pub enum SymbolLikeRef<'a> {
    Symbol(&'a Symbol),
    MathStructure(&'a MathStructure),
    Extension(&'a StructureExtension),
    Morphism(&'a Morphism),
}

impl<'s> TryFrom<AnyDeclarationRef<'s>> for SymbolLikeRef<'s> {
    type Error = ();
    fn try_from(value: AnyDeclarationRef<'s>) -> Result<Self, Self::Error> {
        match value {
            AnyDeclarationRef::Symbol(s) => Ok(Self::Symbol(s)),
            AnyDeclarationRef::MathStructure(s) => Ok(Self::MathStructure(s)),
            AnyDeclarationRef::Extension(s) => Ok(Self::Extension(s)),
            AnyDeclarationRef::Morphism(s) => Ok(Self::Morphism(s)),
            _ => Err(()),
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
pub enum Declaration {
    NestedModule(NestedModule),
    Import {
        uri: ModuleUri,
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        source: SourceRange,
        //#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        //elaborated_from: Option<SymbolUri>,
    },
    Symbol(Symbol),
    MathStructure(MathStructure),
    Morphism(Morphism),
    Extension(StructureExtension),
    Rule {
        id: Id,
        parameters: Box<[Term]>,
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        source: SourceRange,
    },
}

impl crate::__private::Sealed for Declaration {}
impl Declaration {
    #[inline]
    #[must_use]
    pub fn uri(&self) -> Option<&SymbolUri> {
        match self {
            Self::NestedModule(m) => m.uri(),
            Self::Symbol(s) => s.uri(),
            Self::MathStructure(s) => s.uri(),
            Self::Extension(e) => e.uri(),
            Self::Morphism(m) => m.uri(),
            Self::Import { .. } | Self::Rule { .. } => None,
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
            Self::Import {
                uri,
                source,
                //elaborated_from,
            } => AnyDeclarationRef::Import {
                uri,
                source: *source,
                //elaborated_from: elaborated_from.as_ref(),
            },
            Self::Rule {
                id,
                parameters: args,
                source,
            } => AnyDeclarationRef::Rule {
                id,
                parameters: args,
                source: *source,
            },
        }
    }
    /*
    pub fn elaborated_from(&self) -> Option<&SymbolUri> {
        match self {
            Self::NestedModule(_) | Self::Rule { .. } => None,
            Self::Symbol(s) => s.elaborated_from(),
            Self::MathStructure(s) => s.elaborated_from(),
            Self::Extension(e) => e.elaborated_from(),
            Self::Morphism(m) => m.elaborated_from(),
            Self::Import {
                elaborated_from, ..
            } => elaborated_from.as_ref(),
        }
    }
    #[inline]
    pub fn is_primitive(&self) -> bool {
        self.elaborated_from().is_none()
    }
    */
}
impl crate::Ftml for Declaration {
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
            Self::Import { .. } | Self::Rule { .. } => F(std::iter::empty()),
        }
    }
    fn source_range(&self) -> SourceRange {
        match self {
            Self::NestedModule(m) => m.source_range(),
            Self::Symbol(s) => s.source_range(),
            Self::MathStructure(s) => s.source_range(),
            Self::Extension(e) => e.source_range(),
            Self::Morphism(m) => m.source_range(),
            Self::Import { source, .. } | Self::Rule { source, .. } => *source,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, bincode::Encode))]
#[cfg_attr(feature = "serde-lite", derive(serde_lite::Serialize))]
#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(tag = "type"))]
pub enum AnyDeclarationRef<'d> {
    NestedModule(&'d NestedModule),
    Import {
        uri: &'d ModuleUri,
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        source: SourceRange,
        //#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        //elaborated_from: Option<&'d SymbolUri>,
    },
    Symbol(&'d Symbol),
    MathStructure(&'d MathStructure),
    Morphism(&'d Morphism),
    Extension(&'d StructureExtension),
    Rule {
        id: &'d Id,
        parameters: &'d [Term],
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        source: SourceRange,
    },
}
impl<'d> TreeChild<'d> for AnyDeclarationRef<'d> {
    fn tree_children(self) -> impl Iterator<Item = Self> {
        static EMPTY: &[Declaration] = &[];
        match self {
            Self::NestedModule(m) => m.declarations().either(),
            Self::MathStructure(s) => s.declarations().either(),
            Self::Morphism(m) => m.declarations().either(),
            Self::Extension(e) => e.declarations().either(),
            _ => either::Left(EMPTY.iter().map(Declaration::as_ref as _)),
        }
    }
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
            Self::Import { .. } | Self::Rule { .. } => None,
        }
    }
    /*
    #[must_use]
    pub fn elaborated_from(&self) -> Option<&'d SymbolUri> {
        match self {
            Self::NestedModule(_) | Self::Rule { .. } => None,
            Self::Symbol(s) => s.elaborated_from(),
            Self::MathStructure(s) => s.elaborated_from(),
            Self::Extension(e) => e.elaborated_from(),
            Self::Morphism(m) => m.elaborated_from(),
            Self::Import {
                elaborated_from, ..
            } => *elaborated_from,
        }
    }
    #[inline]
    #[must_use]
    pub fn is_primitive(&self) -> bool {
        self.elaborated_from().is_none()
    }
    */
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
            Self::Import { .. } | Self::Rule { .. } => F(std::iter::empty()),
        }
    }
    fn source_range(&self) -> SourceRange {
        match self {
            Self::NestedModule(m) => m.source_range(),
            Self::Symbol(s) => s.source_range(),
            Self::MathStructure(s) => s.source_range(),
            Self::Extension(e) => e.source_range(),
            Self::Morphism(m) => m.source_range(),
            Self::Import { source, .. } | Self::Rule { source, .. } => *source,
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Declaration {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::NestedModule(m) => m.deep_size_of_children(context),
            Self::Symbol(s) => s.deep_size_of_children(context),
            Self::MathStructure(s) => s.deep_size_of_children(context),
            Self::Morphism(s) => s.deep_size_of_children(context),
            Self::Extension(s) => s.deep_size_of_children(context),
            Self::Import { .. } => 0,
            Self::Rule { parameters, .. } => {
                parameters.len() * std::mem::size_of::<Term>()
                    + parameters
                        .iter()
                        .map(|t| t.deep_size_of_children(context))
                        .sum::<usize>()
            }
        }
    }
}
