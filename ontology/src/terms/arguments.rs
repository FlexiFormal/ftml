use std::fmt::Write;

use either::Either;

use super::{Term, Variable};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Argument {
    Simple(Term),
    Sequence(
        #[cfg_attr(feature = "typescript", tsify(type = "{Left: Term}|{Right:Term[]}"))]
        Either<Term, Box<[Term]>>,
    ),
}

impl Argument {
    #[must_use]
    pub const fn mode(&self) -> ArgumentMode {
        match self {
            Self::Simple(_) => ArgumentMode::Simple,
            Self::Sequence(_) => ArgumentMode::Sequence,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum BoundArgument {
    Simple(Term),
    Sequence(
        #[cfg_attr(feature = "typescript", tsify(type = "{Left: Term}|{Right:Term[]}"))]
        Either<Term, Box<[Term]>>,
    ),
    Bound(Variable),
    BoundSeq(
        #[cfg_attr(
            feature = "typescript",
            tsify(type = "{Left: Variable}|{Right:Variable[]}")
        )]
        Either<Variable, Box<[Variable]>>,
    ),
}
impl BoundArgument {
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
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
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

#[derive(thiserror::Error, Debug)]
#[error("invalid argument mode")]
pub struct InvalidArgumentMode;

impl TryFrom<u8> for ArgumentMode {
    type Error = InvalidArgumentMode;
    fn try_from(c: u8) -> Result<Self, Self::Error> {
        match c {
            b'i' => Ok(Self::Simple),
            b'a' => Ok(Self::Sequence),
            b'b' => Ok(Self::BoundVariable),
            b'B' => Ok(Self::BoundVariableSequence),
            _ => Err(InvalidArgumentMode),
        }
    }
}
impl std::str::FromStr for ArgumentMode {
    type Err = InvalidArgumentMode;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(InvalidArgumentMode);
        }
        s.as_bytes()[0].try_into()
    }
}

#[cfg(feature = "deepsize")]
#[allow(clippy::redundant_closure_for_method_calls)]
impl deepsize::DeepSizeOf for Argument {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Simple(t) => t.deep_size_of_children(context),
            Self::Sequence(Either::Left(l)) => l.deep_size_of_children(context),
            Self::Sequence(Either::Right(r)) => r
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum(),
        }
    }
}

#[cfg(feature = "deepsize")]
#[allow(clippy::redundant_closure_for_method_calls)]
impl deepsize::DeepSizeOf for BoundArgument {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Simple(t) => t.deep_size_of_children(context),
            Self::Sequence(Either::Left(l)) => l.deep_size_of_children(context),
            Self::Sequence(Either::Right(r)) => r
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum(),
            Self::Bound(v) => v.deep_size_of_children(context),
            Self::BoundSeq(Either::Left(l)) => l.deep_size_of_children(context),
            Self::BoundSeq(Either::Right(r)) => r
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum(),
        }
    }
}
