use std::fmt::{Debug, Display, Formatter};

use ftml_uris::{DocumentElementUri, UriName};

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Variable {
    Name(UriName),
    Ref {
        declaration: DocumentElementUri,
        is_sequence: Option<bool>,
    },
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => Display::fmt(n, f),
            Self::Ref { declaration, .. } => Display::fmt(declaration.name().last(), f),
        }
    }
}
impl Debug for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => Debug::fmt(n, f),
            Self::Ref { declaration, .. } => Debug::fmt(declaration, f),
        }
    }
}
