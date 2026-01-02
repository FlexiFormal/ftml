mod arguments;
mod bank;
#[cfg(feature = "openmath")]
pub mod om;
pub mod opaque;
pub mod records;
pub mod simplify;
mod term;
pub mod traverser;
mod variables;

pub use arguments::{Argument, ArgumentMode, BoundArgument, MaybeSequence};
pub use bank::clear_term_cache;
#[cfg(feature = "deepsize")]
pub use bank::{TermCacheSize, get_cache_size};
use ftml_uris::{LeafUri, SymbolUri};
pub use term::{
    Application, ApplicationTerm, Binding, BindingTerm, Opaque, OpaqueTerm, RecordField,
    RecordFieldTerm, Term,
};
pub use variables::Variable;

pub trait IsTerm: Clone + std::hash::Hash + PartialEq + Eq {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>>;
    fn subterms(&self) -> impl Iterator<Item = &Term>;

    /// Iterates over all symbols occuring in this expression.
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri>;
    fn variables(&self) -> impl Iterator<Item = &Variable>;
}

//mod syn;

/// Either a variable or a symbol reference
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
pub enum VarOrSym {
    Sym(SymbolUri),
    Var(Variable),
}

impl IsTerm for VarOrSym {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        Some(match self {
            Self::Sym(s) => either::Either::Left(s),
            Self::Var(v) => either::Either::Right(v),
        })
    }
    #[inline]
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::empty()
    }
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        match self {
            Self::Sym(uri) => either::Left(std::iter::once(uri)),
            Self::Var(_) => either::Right(std::iter::empty()),
        }
    }
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        match self {
            Self::Sym(_) => either::Left(std::iter::empty()),
            Self::Var(var) => either::Right(std::iter::once(var)),
        }
    }
}
impl std::fmt::Display for VarOrSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sym(s) => s.fmt(f),
            Self::Var(v) => v.fmt(f),
        }
    }
}
impl From<LeafUri> for VarOrSym {
    fn from(value: LeafUri) -> Self {
        match value {
            LeafUri::Symbol(s) => Self::Sym(s),
            LeafUri::Element(e) => Self::Var(Variable::Ref {
                declaration: e,
                is_sequence: None,
            }),
        }
    }
}

impl From<SymbolUri> for VarOrSym {
    #[inline]
    fn from(value: SymbolUri) -> Self {
        Self::Sym(value)
    }
}
impl From<Variable> for VarOrSym {
    #[inline]
    fn from(value: Variable) -> Self {
        Self::Var(value)
    }
}
