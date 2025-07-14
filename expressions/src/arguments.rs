use std::fmt::Write;

use either::Either;

use crate::{Expr, Variable};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Argument {
    Simple(Expr),
    Sequence(Either<Expr, Box<[Expr]>>),
    Bound(Variable),
    BoundSeq(Either<Variable, Box<[Variable]>>),
}
impl Argument {
    #[must_use]
    pub const fn mode(&self) -> ArgumentMode {
        match self {
            Self::Simple(_) => ArgumentMode::Simple,
            Self::Sequence(_) => ArgumentMode::Sequence,
            Self::Bound(_) => ArgumentMode::BoundVariable,
            Self::BoundSeq(_) => ArgumentMode::BoundVariableSequence,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum ArgumentMode {
    #[default]
    Simple,
    Sequence,
    BoundVariable,
    BoundVariableSequence,
}
impl ArgumentMode {
    #[inline]
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Simple => 'i',
            Self::Sequence => 'a',
            Self::BoundVariable => 'b',
            Self::BoundVariableSequence => 'B',
        }
    }
}
impl std::fmt::Display for ArgumentMode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.as_char())
    }
}
impl TryFrom<u8> for ArgumentMode {
    type Error = ();
    fn try_from(c: u8) -> Result<Self, Self::Error> {
        match c {
            b'i' => Ok(Self::Simple),
            b'a' => Ok(Self::Sequence),
            b'b' => Ok(Self::BoundVariable),
            b'B' => Ok(Self::BoundVariableSequence),
            _ => Err(()),
        }
    }
}
impl std::str::FromStr for ArgumentMode {
    type Err = ();
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(());
        }
        s.as_bytes()[0].try_into()
    }
}

/*
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Arg {
    pub term: Expr,
    pub mode: ArgumentMode,
}
impl From<(Expr, ArgumentMode)> for Arg {
    fn from((term, mode): (Expr, ArgumentMode)) -> Self {
        Self { term, mode }
    }
}
*/
