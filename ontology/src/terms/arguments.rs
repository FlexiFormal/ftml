use super::{Term, Variable};
use std::fmt::Write;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
pub enum Argument {
    Simple(Term),
    Sequence(MaybeSequence<Term>),
}

impl Argument {
    #[must_use]
    pub const fn mode(&self) -> ArgumentMode {
        match self {
            Self::Simple(_) => ArgumentMode::Simple,
            Self::Sequence(_) => ArgumentMode::Sequence,
        }
    }
    pub fn terms(&self) -> impl Iterator<Item = &Term> {
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => {
                either::Left(std::iter::once(t))
            }
            Self::Sequence(MaybeSequence::Seq(ts)) => either::Right(ts.iter()),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
pub enum BoundArgument {
    Simple(Term),
    Sequence(MaybeSequence<Term>),
    Bound(ComponentVar),
    BoundSeq(MaybeSequence<ComponentVar>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
pub struct ComponentVar {
    pub var: Variable,
    pub tp: Option<Term>,
    pub df: Option<Term>,
}

#[cfg(not(feature = "serde-lite"))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum MaybeSequence<T>
where
    T: 'static,
{
    One(T),
    Seq(Box<[T]>),
}

#[cfg(feature = "serde-lite")]
#[derive(Debug, Clone, Hash, PartialEq, Eq, serde_lite::Serialize, serde_lite::Deserialize)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum MaybeSequence<T>
where
    T: serde_lite::Serialize + serde_lite::Deserialize + 'static,
{
    One(T),
    Seq(Box<[T]>),
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

    pub fn terms(&self) -> impl Iterator<Item = &Term> {
        use either_of::EitherOf3 as E;
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => E::A(std::iter::once(t)),
            Self::Sequence(MaybeSequence::Seq(ts)) => E::B(ts.iter()),
            _ => E::C(std::iter::empty()),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
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
            Self::Sequence(MaybeSequence::One(l)) => l.deep_size_of_children(context),
            Self::Sequence(MaybeSequence::Seq(r)) => r
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum(),
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for ComponentVar {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.var.deep_size_of_children(context)
            + self
                .tp
                .as_ref()
                .map(|t| t.deep_size_of_children(context))
                .unwrap_or_default()
            + self
                .df
                .as_ref()
                .map(|t| t.deep_size_of_children(context))
                .unwrap_or_default()
    }
}

#[cfg(feature = "deepsize")]
#[allow(clippy::redundant_closure_for_method_calls)]
impl deepsize::DeepSizeOf for BoundArgument {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Simple(t) => t.deep_size_of_children(context),
            Self::Sequence(MaybeSequence::One(l)) => l.deep_size_of_children(context),
            Self::Sequence(MaybeSequence::Seq(r)) => r
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum(),
            Self::Bound(v) => v.deep_size_of_children(context),
            Self::BoundSeq(MaybeSequence::One(l)) => l.deep_size_of_children(context),
            Self::BoundSeq(MaybeSequence::Seq(r)) => r
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum(),
        }
    }
}
