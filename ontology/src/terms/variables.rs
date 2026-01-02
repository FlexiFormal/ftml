use std::fmt::{Debug, Display, Formatter};

use ftml_uris::{DocumentElementUri, Id};

use crate::terms::IsTerm;

#[derive(Clone, PartialEq, Eq, Hash)]
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
pub enum Variable {
    Name {
        name: Id,
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        notated: Option<Id>,
    },
    Ref {
        declaration: DocumentElementUri,
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        is_sequence: Option<bool>,
    },
}

impl IsTerm for Variable {
    #[inline]
    fn head(&self) -> Option<either::Either<&ftml_uris::SymbolUri, &Variable>> {
        Some(either::Right(self))
    }
    #[inline]
    fn subterms(&self) -> impl Iterator<Item = &super::Term> {
        std::iter::empty()
    }
    #[inline]
    fn symbols(&self) -> impl Iterator<Item = &ftml_uris::SymbolUri> {
        std::iter::empty()
    }
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        std::iter::once(self)
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name {
                notated: Some(n), ..
            } => Display::fmt(n, f),
            Self::Name { name, .. } => Display::fmt(name, f),
            Self::Ref { declaration, .. } => Display::fmt(declaration.name().last(), f),
        }
    }
}
impl Debug for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name {
                notated: Some(n), ..
            } => Debug::fmt(n, f),
            Self::Name { name, .. } => Debug::fmt(name, f),
            Self::Ref { declaration, .. } => Debug::fmt(declaration, f),
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Variable {
    #[inline]
    fn deep_size_of_children(&self, _: &mut deepsize::Context) -> usize {
        0
    }
}
