mod arguments;
mod bank;
pub mod eq;
mod macros;
#[cfg(feature = "openmath")]
pub mod om;
pub mod opaque;
pub mod records;
pub mod simplify;
pub mod subst;
mod term;
pub mod termpaths;
//pub mod traverser;
mod variables;

pub use arguments::{Argument, ArgumentMode, BoundArgument, ComponentVar, MaybeSequence};
pub use bank::clear_term_cache;
#[cfg(feature = "deepsize")]
pub use bank::{TermCacheSize, get_cache_size};
use ftml_uris::{LeafUri, SymbolUri};
pub use term::{
    Application, ApplicationTerm, Binding, BindingTerm, Numeric, Opaque, OpaqueTerm, RecordField,
    RecordFieldTerm, Term,
};
pub use variables::Variable;

use crate::utils::SourceRange;

pub trait IsTerm: Clone + std::hash::Hash + PartialEq + Eq {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>>;
    fn subterms(&self) -> impl Iterator<Item = &Term>;

    /// Iterates over all symbols occuring in this expression.
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri>;
    fn variables(&self) -> impl Iterator<Item = &Variable>;
}

#[derive(Clone, Debug, Default)]
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
pub struct TermContainer {
    parsed: Option<Term>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    #[cfg_attr(feature = "typescript", tsify(type = "Term | undefined"))]
    checked: CheckedContainer,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub source: SourceRange,
}
impl TermContainer {
    #[must_use]
    pub fn new(t: Term, source: Option<SourceRange>) -> Self {
        Self {
            parsed: Some(t),
            source: source.unwrap_or_default(),
            checked: CheckedContainer::default(),
        }
    }

    #[must_use]
    pub const fn is_some(&self) -> bool {
        self.parsed.is_some()
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.parsed.is_none()
    }

    #[must_use]
    pub fn has_checked(&self) -> bool {
        self.checked.0.lock().is_some()
    }
    #[inline]
    #[must_use]
    pub const fn parsed(&self) -> Option<&Term> {
        self.parsed.as_ref()
    }

    #[must_use]
    pub fn checked_or_parsed(&self) -> Option<(Term, bool)> {
        self.checked.0.lock().as_ref().map_or_else(
            || self.parsed.clone().map(|t| (t, false)),
            |t| Some((t.clone(), true)),
        )
    }
    pub fn set_checked(&self, t: Term) {
        *self.checked.0.lock() = Some(t);
    }
}

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

impl std::hash::Hash for TermContainer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parsed.hash(state);
    }
}
impl PartialEq for TermContainer {
    fn eq(&self, other: &Self) -> bool {
        self.parsed.eq(&other.parsed)
    }
}
impl Eq for TermContainer {}

#[derive(Default, Debug, Clone)]
struct CheckedContainer(std::sync::Arc<parking_lot::Mutex<Option<Term>>>);
impl From<Option<Term>> for CheckedContainer {
    fn from(value: Option<Term>) -> Self {
        Self(std::sync::Arc::new(parking_lot::Mutex::new(value)))
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for TermContainer {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.parsed.deep_size_of_children(context)
            + self.checked.0.lock().deep_size_of_children(context)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::terms::{CheckedContainer, Term};

    impl serde::Serialize for CheckedContainer {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.lock().serialize(serializer)
        }
    }
    impl<'de> serde::Deserialize<'de> for CheckedContainer {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(Option::<Term>::deserialize(deserializer)?.into())
        }
    }
    impl bincode::Encode for CheckedContainer {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            self.0.lock().encode(encoder)
        }
    }
    impl<'de, Ctx> bincode::BorrowDecode<'de, Ctx> for CheckedContainer {
        fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Ctx>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Ok(Option::<Term>::borrow_decode(decoder)?.into())
        }
    }
    impl<Ctx> bincode::Decode<Ctx> for CheckedContainer {
        fn decode<D: bincode::de::Decoder<Context = Ctx>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Ok(Option::<Term>::decode(decoder)?.into())
        }
    }
}

#[cfg(feature = "serde-lite")]
mod serde_lite_impl {
    use crate::terms::{CheckedContainer, Term};

    impl serde_lite::Serialize for CheckedContainer {
        fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
            self.0.lock().serialize()
        }
    }
    impl serde_lite::Deserialize for CheckedContainer {
        fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error>
        where
            Self: Sized,
        {
            Ok(Option::<Term>::deserialize(val)?.into())
        }
    }
}
