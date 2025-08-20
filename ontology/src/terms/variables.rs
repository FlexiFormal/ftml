use std::fmt::{Debug, Display, Formatter};

use ftml_uris::{DocumentElementUri, Id};

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Variable {
    Name {
        name: Id,
        #[cfg_attr(feature = "serde", serde(default))]
        notated: Option<Id>,
    },
    Ref {
        declaration: DocumentElementUri,
        #[cfg_attr(feature = "serde", serde(default))]
        is_sequence: Option<bool>,
    },
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
