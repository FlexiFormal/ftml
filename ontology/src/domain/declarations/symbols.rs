use ftml_uris::{Id, SymbolUri};
use smallvec::SmallVec;
use std::str::FromStr;

use crate::{
    domain::declarations::{AnyDeclarationRef, IsDeclaration},
    terms::{ArgumentMode, Term},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Symbol {
    pub uri: SymbolUri,
    pub data: Box<SymbolData>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SymbolData {
    pub arity: ArgumentSpec,
    #[cfg_attr(feature = "serde", serde(default))]
    pub macroname: Option<Id>,
    pub role: Box<[Id]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub tp: Option<Term>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub df: Option<Term>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub return_type: Option<Term>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub argument_types: Box<[Term]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub assoctype: Option<AssocType>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub reordering: Option<Id>,
}

impl crate::__private::Sealed for Symbol {}
impl crate::Ftml for Symbol {
    #[cfg(feature = "rdf")]
    #[allow(clippy::enum_glob_use)]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use either_of::EitherOf6::*;
        use ftml_uris::FtmlUri;
        use rustc_hash::FxHashSet;
        use ulo::triple;
        let iri = self.uri.to_iri();
        macro_rules! syms {
            ($e:expr) => {{
                let iri2 = iri.clone();
                $e.symbols().collect::<FxHashSet<_>>().into_iter()
                    .map(move |s| triple!(<(iri2.clone())> dc:hasPart <(s.to_iri())>))
            }};
        }
        match (&self.data.tp, &self.data.df) {
            (Some(Term::Symbol { uri: tp, .. }), Some(df)) => A(syms!(df).chain([
                triple!(<(iri.clone())> : ulo:declaration),
                triple!(<(iri)> ulo:has_type  <(tp.to_iri())>),
            ])),
            (Some(tp), Some(df)) => B(syms!(tp)
                .chain(syms!(df))
                .chain(std::iter::once(triple!(<(iri)> : ulo:declaration)))),
            (Some(Term::Symbol { uri: tp, .. }), _) => C([
                triple!(<(iri.clone())> : ulo:declaration),
                triple!(<(iri)> ulo:has_type  <(tp.to_iri())>),
            ]
            .into_iter()),
            (Some(tp), _) => {
                D(syms!(tp).chain(std::iter::once(triple!(<(iri)> : ulo:declaration))))
            }
            (_, Some(df)) => {
                E(syms!(df).chain(std::iter::once(triple!(<(iri)> : ulo:declaration))))
            }
            (None, None) => F(std::iter::once(triple!(<(iri)> : ulo:declaration))),
        }
    }
}
impl IsDeclaration for Symbol {
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        Some(&self.uri)
    }
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        match decl {
            AnyDeclarationRef::Symbol(m) => Some(m),
            _ => None,
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::Symbol(self)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[non_exhaustive]
pub enum AssocType {
    LeftAssociativeBinary,
    RightAssociativeBinary,
    Conjunctive,
    PairwiseConjunctive,
    Prenex,
}

#[derive(thiserror::Error, Debug)]
#[error("invalid assoc type for symbol")]
pub struct InvalidAssocType;

impl FromStr for AssocType {
    type Err = InvalidAssocType;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binl" | "bin" => Ok(Self::LeftAssociativeBinary),
            "binr" => Ok(Self::RightAssociativeBinary),
            "conj" => Ok(Self::Conjunctive),
            "pwconj" => Ok(Self::PairwiseConjunctive),
            "pre" => Ok(Self::Prenex),
            _ => Err(InvalidAssocType),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ArgumentSpec(
    #[cfg_attr(feature = "typescript", tsify(type = "ArgumentMode[]"))] SmallVec<ArgumentMode, 8>,
);
impl IntoIterator for ArgumentSpec {
    type Item = ArgumentMode;
    type IntoIter = smallvec::IntoIter<ArgumentMode, 8>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ArgumentSpec {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn num(&self) -> u8 {
        self.0.len() as u8
    }
}

impl Default for ArgumentSpec {
    #[inline]
    fn default() -> Self {
        Self(SmallVec::new())
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid arguments-string for symbol")]
pub struct InvalidArgumentSpec;

impl FromStr for ArgumentSpec {
    type Err = InvalidArgumentSpec;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(u) = s.parse::<u8>() {
            return Ok(Self((0..u).map(|_| ArgumentMode::Simple).collect()));
        }
        let mut ret = SmallVec::new();
        for c in s.bytes() {
            ret.push(match c {
                b'i' => ArgumentMode::Simple,
                b'a' => ArgumentMode::Sequence,
                b'b' => ArgumentMode::BoundVariable,
                b'B' => ArgumentMode::BoundVariableSequence,
                _ => return Err(InvalidArgumentSpec),
            });
        }
        Ok(Self(ret))
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for SymbolData {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        (self.role.len() * std::mem::size_of::<Id>())
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
            + self
                .return_type
                .as_ref()
                .map(|t| t.deep_size_of_children(context))
                .unwrap_or_default()
            + self
                .argument_types
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum::<usize>()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Symbol {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        std::mem::size_of::<SymbolData>() + self.data.deep_size_of_children(context)
    }
}
