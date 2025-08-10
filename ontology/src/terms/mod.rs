mod term;
use ftml_uris::{LeafUri, SymbolUri};
pub use term::Term;
mod variables;
pub use variables::Variable;
mod arguments;
pub mod opaque;
pub use arguments::{Argument, ArgumentMode, BoundArgument};
#[cfg(feature = "openmath")]
pub mod om;
pub mod records;
pub mod simplify;

//mod syn;

/// Either a variable or a symbol reference
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum VarOrSym {
    Sym(SymbolUri),
    Var(Variable),
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
