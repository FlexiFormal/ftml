use super::{BoundArgument, arguments::Argument, variables::Variable};
use crate::terms::IsTerm;
use crate::terms::arguments::ComponentVar;
use crate::terms::opaque::OpaqueNode;
use crate::terms::{VarOrSym, arguments::MaybeSequence};
use crate::utils::{Float, RefTree, TreeIter};
use ftml_uris::{SymbolUri, UriName};
use std::fmt::Write;
use std::str::FromStr;

/// The type of FTML expressions.
///
/// Similarly to
/// [<span style="font-variant:small-caps;">OpenMath</span>](https://openmath.org),
/// FTML expressions are foundation-independent, but more expressive by hardcoding
/// [Theories-as-Types]()-like record "types".
#[derive(Clone, Hash, PartialEq, Eq)]
#[allow(clippy::unsafe_derive_deserialize)]
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
pub enum Term {
    /// A reference to a symbol (e.g. $\mathbb N$)
    Symbol {
        uri: SymbolUri,
        presentation: Option<VarOrSym>,
    },
    /// A reference to a (bound) variable (e.g. $x$)
    Var {
        variable: Variable,
        presentation: Option<VarOrSym>,
    },
    /// An application of `head` to `arguments` (e.g. $n + m$)
    Application(ApplicationTerm),
    /// A *binding* application with `head` as operator, `arguments`
    /// being either variable bindings or arbitrary expression arguments,
    /// and `body` being the (final) expression *in which* the variables are bound
    /// (e.g. $\int_{t=0}^\infty f(t) \mathrm dt$)
    Bound(BindingTerm),
    /// Record projection; the field named `key` in the record `record`.
    /// The optional `record_type` ideally references the type in which field names
    /// can be looked up.
    Field(RecordFieldTerm),
    /// A non-alpha-renamable variable
    Label {
        name: UriName,
        #[cfg_attr(feature = "typescript", tsify(type = "Term | undefined"))]
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        df: Option<Box<Self>>,
        #[cfg_attr(feature = "typescript", tsify(type = "Term | undefined"))]
        #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
        tp: Option<Box<Self>>,
    },
    /// An opaque/informal expression; may contain formal islands, which are collected in
    /// `expressions`.
    Opaque(OpaqueTerm),
    // A numeric literal
    Number(Numeric),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
pub enum Numeric {
    Int(i64),
    Float(Float),
}
impl FromStr for Numeric {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map_or_else(
            |_| {
                s.parse::<f32>()
                    .map_or(Err(()), |f| Ok(Self::Float(f.into())))
            },
            |i| Ok(Self::Int(i)),
        )
    }
}

impl IsTerm for Term {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        match self {
            Self::Symbol { uri, .. } => Some(either::Left(uri)),
            Self::Var { variable, .. } => Some(either::Right(variable)),
            Self::Application(a) => a.head(),
            Self::Bound(b) => b.head(),
            Self::Field(f) => f.head(),
            Self::Opaque(_) | Self::Label { .. } | Self::Number(_) => None,
        }
    }
    #[inline]
    fn subterms(&self) -> impl Iterator<Item = &Self> {
        self.tree_children()
    }

    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        use either_of::EitherOf3 as E;
        match self {
            Self::Symbol { uri, .. } => E::A(std::iter::once(uri)),
            Self::Var { .. } => E::B(std::iter::empty()),
            o => E::C(SubtermIter::One(o).dfs().filter_map(|t| {
                if let Self::Symbol { uri, .. } = t {
                    Some(uri)
                } else {
                    None
                }
            })),
        }
    }

    // TODO: does this need to be boxed? -.-
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        use either_of::EitherOf3 as E;
        match self {
            Self::Symbol { .. } | Self::Label { .. } | Self::Number(_) => {
                E::A(std::iter::empty::<&Variable>())
            }
            Self::Var { variable, .. } => E::B(std::iter::once(variable)),
            Self::Application(a) => E::C(Box::new(a.variables()) as Box<dyn Iterator<Item = _>>),
            Self::Bound(b) => E::C(Box::new(b.variables()) as Box<dyn Iterator<Item = _>>),
            Self::Opaque(o) => E::C(Box::new(o.variables()) as Box<dyn Iterator<Item = _>>),
            Self::Field(f) => E::C(Box::new(f.record.variables()) as Box<dyn Iterator<Item = _>>),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ApplicationTerm(pub(crate) triomphe::Arc<Application>);
impl IsTerm for ApplicationTerm {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        self.head.head()
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::once(&self.head).chain(self.arguments.iter().flat_map(Argument::terms))
    }
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        SubtermIter::App(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Symbol { uri, .. } = t {
                    Some(uri)
                } else {
                    None
                }
            })
    }
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        SubtermIter::App(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Var { variable, .. } = t {
                    Some(variable)
                } else {
                    None
                }
            })
    }
}
impl IsTerm for Application {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        self.head.head()
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::once(&self.head).chain(self.arguments.iter().flat_map(Argument::terms))
    }

    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        SubtermIter::App(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Symbol { uri, .. } = t {
                    Some(uri)
                } else {
                    None
                }
            })
    }

    fn variables(&self) -> impl Iterator<Item = &Variable> {
        SubtermIter::App(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Var { variable, .. } = t {
                    Some(variable)
                } else {
                    None
                }
            })
    }
}

#[derive(Clone, PartialEq, Eq)]
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
pub struct Application {
    pub head: Term,
    pub arguments: Box<[Argument]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub presentation: Option<VarOrSym>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(skip))]
    pub(crate) hash: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct BindingTerm(pub(crate) triomphe::Arc<Binding>);

impl IsTerm for BindingTerm {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        self.head.head()
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::once(&self.head).chain(self.arguments.iter().flat_map(BoundArgument::terms))
    }

    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        SubtermIter::Bound(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Symbol { uri, .. } = t {
                    Some(uri)
                } else {
                    None
                }
            })
    }

    fn variables(&self) -> impl Iterator<Item = &Variable> {
        use either_of::EitherOf3 as E;
        SubtermIter::Bound(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Var { variable, .. } = t {
                    Some(variable)
                } else {
                    None
                }
            })
            .chain(self.arguments.iter().flat_map(|ba| match ba {
                BoundArgument::Bound(v) | BoundArgument::BoundSeq(MaybeSequence::One(v)) => {
                    E::A(std::iter::once(&v.var))
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(s)) => E::B(s.iter().map(|v| &v.var)),
                _ => E::C(std::iter::empty()),
            }))
    }
}
impl IsTerm for Binding {
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        self.head.head()
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::once(&self.head).chain(self.arguments.iter().flat_map(BoundArgument::terms))
    }
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        SubtermIter::Bound(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Symbol { uri, .. } = t {
                    Some(uri)
                } else {
                    None
                }
            })
    }

    fn variables(&self) -> impl Iterator<Item = &Variable> {
        use either_of::EitherOf3 as E;
        SubtermIter::Bound(Some(&self.head), &self.arguments)
            .dfs()
            .filter_map(|t| {
                if let Term::Var { variable, .. } = t {
                    Some(variable)
                } else {
                    None
                }
            })
            .chain(self.arguments.iter().flat_map(|ba| match ba {
                BoundArgument::Bound(v) | BoundArgument::BoundSeq(MaybeSequence::One(v)) => {
                    E::A(std::iter::once(&v.var))
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(s)) => E::B(s.iter().map(|v| &v.var)),
                _ => E::C(std::iter::empty()),
            }))
    }
}

#[derive(Clone, PartialEq, Eq)]
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
pub struct Binding {
    pub head: Term,
    pub arguments: Box<[BoundArgument]>,
    //pub body: BoundArgument, //Term,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub presentation: Option<VarOrSym>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(skip))]
    pub(crate) hash: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct RecordFieldTerm(pub(crate) triomphe::Arc<RecordField>);

impl IsTerm for RecordFieldTerm {
    #[inline]
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        self.record.head()
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::once(&self.record)
    }
    #[inline]
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        self.record.symbols()
    }
    #[inline]
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.record.variables()
    }
}
impl IsTerm for RecordField {
    #[inline]
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        self.record.head()
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        std::iter::once(&self.record)
    }
    #[inline]
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        self.record.symbols()
    }
    #[inline]
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.record.variables()
    }
}

#[derive(Clone, PartialEq, Eq)]
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
pub struct RecordField {
    pub record: Term,
    pub key: UriName,
    /// does not count as a subterm
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub record_type: Option<Term>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub presentation: Option<VarOrSym>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(skip))]
    pub(crate) hash: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpaqueTerm(pub(crate) triomphe::Arc<Opaque>);

impl IsTerm for OpaqueTerm {
    #[inline]
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        None
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        self.terms.iter()
    }
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        self.terms.iter().dfs().filter_map(|t| {
            if let Term::Symbol { uri, .. } = t {
                Some(uri)
            } else {
                None
            }
        })
    }
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.terms.iter().flat_map(Term::variables)
    }
}
impl IsTerm for Opaque {
    #[inline]
    fn head(&self) -> Option<either::Either<&SymbolUri, &Variable>> {
        None
    }
    fn subterms(&self) -> impl Iterator<Item = &Term> {
        self.terms.iter()
    }
    fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        self.terms.iter().dfs().filter_map(|t| {
            if let Term::Symbol { uri, .. } = t {
                Some(uri)
            } else {
                None
            }
        })
    }
    fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.terms.iter().flat_map(Term::variables)
    }
}

#[derive(Clone, PartialEq, Eq)]
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
pub struct Opaque {
    pub node: OpaqueNode,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub terms: Box<[Term]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(skip))]
    pub(crate) hash: u64,
}

impl Term {
    /// implements [`Debug`](std::fmt::Debug), but only prints the *names* of [`Uri`](ftml_uris::Uri)s
    #[inline]
    #[must_use]
    pub fn debug_short(&self) -> impl std::fmt::Debug {
        Short(self)
    }

    #[must_use]
    pub fn with_presentation(self, pres: VarOrSym) -> Self {
        match self {
            Self::Symbol { uri, .. } if !matches!(&pres,VarOrSym::Sym(s) if *s == uri) => {
                Self::Symbol {
                    uri,
                    presentation: Some(pres),
                }
            }
            Self::Var { variable, .. } if !matches!(&pres,VarOrSym::Var(v) if *v == variable) => {
                Self::Var {
                    variable,
                    presentation: Some(pres),
                }
            }
            Self::Application(a) => Self::Application(ApplicationTerm::new(
                a.head.clone(),
                a.arguments.clone(),
                Some(pres),
            )),
            Self::Bound(b) => Self::Bound(BindingTerm::new(
                b.head.clone(),
                b.arguments.clone(),
                //b.body.clone(),
                Some(pres),
            )),
            Self::Field(f) => Self::Field(RecordFieldTerm::new(
                f.record.clone(),
                f.key.clone(),
                f.record_type.clone(),
                Some(pres),
            )),
            o => o,
        }
    }

    pub fn into_seq(seqs: impl Iterator<Item = Self>) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::SEQUENCE_EXPRESSION.clone(),
                presentation: None,
            },
            seqs.map(Argument::Simple).collect(),
            None,
        ))
    }
}

impl crate::utils::RefTree for Term {
    type Child<'a>
        = &'a Self
    where
        Self: 'a;

    #[allow(refining_impl_trait_reachable)]
    fn tree_children(&self) -> SubtermIter<'_> {
        match self {
            Self::Symbol{..}
            | Self::Var{..}
            //| Self::Module(_)
            | Self::Label {
                df: None, tp: None, ..
            } | Self::Number(_) => SubtermIter::E,
            Self::Application(a) => SubtermIter::App(Some(&a.head), &a.arguments),
            Self::Bound(b) => SubtermIter::Bound(Some(&b.head), &b.arguments),//, &b.body),
            Self::Label {
                df: Some(df),
                tp: Some(tp),
                ..
            } => SubtermIter::Two(tp, df),
            Self::Field(f) => SubtermIter::One(&f.record),
            Self::Label { df: Some(t), .. } | Self::Label { tp: Some(t), .. } => {
                SubtermIter::One(t)
            }
            Self::Opaque(o) => SubtermIter::Slice(o.terms.iter()),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn fmt<const LONG: bool>(e: &Term, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match e {
        Term::Symbol { uri, .. } if LONG => write!(f, "{uri}"),
        Term::Symbol { uri, .. } => write!(f, "\"{}\"", uri.name()),
        Term::Var {
            variable: Variable::Name {
                notated: Some(n), ..
            },
            ..
        } => write!(f, "V({n})"),
        Term::Var {
            variable: Variable::Name { name, .. },
            ..
        } => write!(f, "V({name})"),
        Term::Number(n) => std::fmt::Debug::fmt(n, f),
        Term::Var {
            variable: Variable::Ref { declaration, .. },
            ..
        } if LONG => write!(f, "V({declaration})"),
        Term::Var {
            variable: Variable::Ref { declaration, .. },
            ..
        } => write!(f, "V({})", declaration.name()),
        Term::Field(field) if field.record_type.is_none() => {
            fmt::<LONG>(&field.record, f)?;
            f.write_char('.')?;
            std::fmt::Debug::fmt(&field.key, f)
        }
        Term::Field(field) => f
            .debug_struct("Field")
            .field("name", &field.key)
            .field("record", &field.record)
            .field("type", &field.record_type)
            .finish(),
        Term::Label {
            name,
            df: None,
            tp: None,
        } => write!(f, "Label({name})"),
        Term::Label {
            name,
            df: Some(df),
            tp: Some(tp),
        } if LONG => f
            .debug_struct("Label")
            .field("", name)
            .field(":", tp)
            .field(":=", df)
            .finish(),
        Term::Label {
            name,
            df: Some(df),
            tp: Some(tp),
        } => f
            .debug_struct("Label")
            .field("", name)
            .field(":", &tp.debug_short())
            .field(":=", &df.debug_short())
            .finish(),
        Term::Label {
            name, tp: Some(tp), ..
        } if LONG => f
            .debug_struct("Label")
            .field("", name)
            .field(":", tp)
            .finish(),
        Term::Label {
            name, tp: Some(tp), ..
        } => f
            .debug_struct("Label")
            .field("", name)
            .field(":", &tp.debug_short())
            .finish(),
        Term::Label {
            name, df: Some(df), ..
        } if LONG => f
            .debug_struct("Label")
            .field("", name)
            .field(":=", df)
            .finish(),
        Term::Label {
            name, df: Some(df), ..
        } => f
            .debug_struct("Label")
            .field("", name)
            .field(":=", &df.debug_short())
            .finish(),
        Term::Application(a) if LONG => f
            .debug_struct("OMA")
            .field("head", &a.head)
            .field("arguments", &a.arguments)
            .finish(),
        Term::Application(a) => {
            write!(f, "{:?}", a.head.debug_short())?;
            let mut tup = f.debug_list();
            for a in &a.arguments {
                tup.entry(&ShortArg(a));
            }
            tup.finish()
        }
        Term::Bound(b) if LONG => f
            .debug_struct("OMBIND")
            .field("head", &b.head)
            .field("arguments", &b.arguments)
            //.field("body", &b.body)
            .finish(),
        Term::Bound(b) => {
            write!(f, "{:?}", b.head.debug_short())?;
            let mut tup = f.debug_list();
            for a in &b.arguments {
                tup.entry(&ShortBoundArg(a));
            }
            tup.finish()
        }
        Term::Opaque(o) => {
            write!(f, "<{}", o.node.tag)?;
            for (k, v) in &o.node.attributes {
                write!(f, " {k}=\"{v}\"")?;
            }
            f.write_str(">\n")?;
            for t in &o.node.children {
                writeln!(f, "{t}")?;
            }
            f.write_str("<terms>\n")?;
            for t in &o.terms {
                fmt::<LONG>(t, f)?;
                f.write_char('\n')?;
            }
            write!(f, "</{}>", o.node.tag)
        }
    }
}
impl std::fmt::Debug for Term {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::<true>(self, f)
    }
}
struct Short<'e>(&'e Term);
impl std::fmt::Debug for Short<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::<false>(self.0, f)
    }
}

struct ShortArg<'e>(&'e Argument);
impl std::fmt::Debug for ShortArg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Argument::Simple(t) => t.debug_short().fmt(f),
            Argument::Sequence(MaybeSequence::One(t)) => {
                write!(f, "({:?})", t.debug_short())
            }
            Argument::Sequence(MaybeSequence::Seq(s)) => {
                f.write_char('[')?;
                let mut fl = f.debug_list();
                for t in s {
                    fl.entry(&t.debug_short());
                }
                fl.finish()?;
                f.write_char(']')
            }
        }
    }
}

struct ShortBoundArg<'e>(&'e BoundArgument);
impl std::fmt::Debug for ShortBoundArg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BoundArgument::Simple(t) => t.debug_short().fmt(f),
            BoundArgument::Sequence(MaybeSequence::One(t)) => {
                write!(f, "({:?})", t.debug_short())
            }
            BoundArgument::Bound(ComponentVar {
                var: Variable::Name { name, .. },
                tp,
                df,
            }) => {
                write!(f, "{{{name}")?;
                if let Some(tp) = tp.as_ref() {
                    write!(f, " : {:?}", tp.debug_short())?;
                }
                if let Some(df) = df.as_ref() {
                    write!(f, " := {:?}", df.debug_short())?;
                }
                f.write_char('}')
            }
            BoundArgument::Bound(ComponentVar {
                var: Variable::Ref { declaration, .. },
                tp,
                df,
            }) => {
                write!(f, "{{{}", declaration.name())?;
                if let Some(tp) = tp.as_ref() {
                    write!(f, " : {:?}", tp.debug_short())?;
                }
                if let Some(df) = df.as_ref() {
                    write!(f, " := {:?}", df.debug_short())?;
                }
                f.write_char('}')
            }
            BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                var: Variable::Name { name, .. },
                tp,
                df,
            })) => {
                write!(f, "{{[{name}")?;
                if let Some(tp) = tp.as_ref() {
                    write!(f, " : {:?}", tp.debug_short())?;
                }
                if let Some(df) = df.as_ref() {
                    write!(f, " := {:?}", df.debug_short())?;
                }
                f.write_str("]}")
            }
            BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                var: Variable::Ref { declaration, .. },
                tp,
                df,
            })) => {
                write!(f, "{{[{}", declaration.name())?;
                if let Some(tp) = tp.as_ref() {
                    write!(f, " : {:?}", tp.debug_short())?;
                }
                if let Some(df) = df.as_ref() {
                    write!(f, " := {:?}", df.debug_short())?;
                }
                f.write_str("]}")
            }
            BoundArgument::Sequence(MaybeSequence::Seq(s)) => {
                f.write_char('[')?;
                let mut fl = f.debug_list();
                for t in s {
                    fl.entry(&t.debug_short());
                }
                fl.finish()?;
                f.write_char(']')
            }
            BoundArgument::BoundSeq(MaybeSequence::Seq(s)) => {
                f.write_char('{')?;
                let mut fl = f.debug_list();
                for v in s {
                    match &v.var {
                        Variable::Name { name, .. } => {
                            fl.entry(name);
                        }
                        Variable::Ref { declaration, .. } => {
                            fl.entry(declaration.name());
                        }
                    }
                }
                fl.finish()?;
                f.write_char('}')
            }
        }
    }
}

pub enum SubtermIter<'a> {
    E,
    App(Option<&'a Term>, &'a [Argument]),
    Bound(Option<&'a Term>, &'a [BoundArgument]), //, &'a Term),
    Arg(&'a [Term], &'a [Argument]),
    BArg(&'a [Term], &'a [BoundArgument]), //, &'a Term),
    One(&'a Term),
    Two(&'a Term, &'a Term),
    Slice(std::slice::Iter<'a, Term>),
}
impl<'a> Iterator for SubtermIter<'a> {
    type Item = &'a Term;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::E => None,
            Self::One(e) => {
                let e = *e;
                *self = Self::E;
                Some(e)
            }
            Self::Two(a, b) => {
                let a = *a;
                let b = *b;
                *self = Self::One(b);
                Some(a)
            }
            Self::Slice(i) => i.next(),
            Self::App(head, args) => {
                if let Some(f) = head.take() {
                    return Some(f);
                }
                loop {
                    let next = args.first()?;
                    *args = &args[1..];
                    match next {
                        Argument::Simple(tm) | Argument::Sequence(MaybeSequence::One(tm)) => {
                            return Some(tm);
                        }
                        Argument::Sequence(MaybeSequence::Seq(seq)) => {
                            if let Some(f) = seq.first() {
                                if seq.len() > 1 {
                                    *self = Self::Arg(&seq[1..], args);
                                }
                                return Some(f);
                            }
                        }
                    }
                }
            }
            Self::Bound(head, args) => {
                //, b) => {
                if let Some(f) = head.take() {
                    return Some(f);
                }
                loop {
                    let Some(next) = args.first() else {
                        /*
                        let b = *b;
                        *self = Self::E;
                        return Some(b);
                        */
                        return None;
                    };
                    *args = &args[1..];
                    match next {
                        BoundArgument::Simple(f)
                        | BoundArgument::Sequence(MaybeSequence::One(f)) => {
                            return Some(f);
                        }
                        BoundArgument::Sequence(MaybeSequence::Seq(seq)) => {
                            if let Some(f) = seq.first() {
                                if seq.len() > 1 {
                                    while matches!(
                                        args.first(),
                                        Some(BoundArgument::Bound(_) | BoundArgument::BoundSeq(_))
                                    ) {
                                        *args = &args[1..];
                                    }
                                    *self = Self::BArg(&seq[1..], args); //, b);
                                }
                                return Some(f);
                            }
                        }
                        _ => (),
                    }
                }
            }
            Self::Arg(ls, rest) => {
                // SAFETY: only constructed with non-empty ls (see above)
                // and replaced by Self::App after emptying (below)
                let f = unsafe { ls.first().unwrap_unchecked() };
                *ls = &ls[1..];
                if ls.is_empty() {
                    *self = Self::App(None, rest);
                }
                Some(f)
            }
            Self::BArg(ls, rest) => {
                //, b) => {
                // SAFETY: only constructed with non-empty ls (see above)
                // and replaced by Self::Bound after emptying (below)
                let f = unsafe { ls.first().unwrap_unchecked() };
                *ls = &ls[1..];
                if ls.is_empty() {
                    *self = Self::Bound(None, rest); //, b);
                }
                Some(f)
            }
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Term {
    #[allow(clippy::only_used_in_recursion)]
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Label { df, tp, .. } => {
                tp.as_ref()
                    .map(|t| std::mem::size_of::<Self>() + (**t).deep_size_of_children(context))
                    .unwrap_or_default()
                    + df.as_ref()
                        .map(|t| std::mem::size_of::<Self>() + (**t).deep_size_of_children(context))
                        .unwrap_or_default()
            }
            /*Self::Application(a) => {
                std::mem::size_of::<Self>()
                    + (**head).deep_size_of_children(context)
                    + arguments.iter().map(Argument::deep_size_of).sum::<usize>()
            }
            Self::Bound {
                head,
                arguments,
                body,
                ..
            } => {
                std::mem::size_of::<Self>()
                    + (**head).deep_size_of_children(context)
                    + std::mem::size_of::<Self>()
                    + (**body).deep_size_of_children(context)
                    + arguments
                        .iter()
                        .map(BoundArgument::deep_size_of)
                        .sum::<usize>()
            }
            Self::Field {
                record,
                record_type,
                ..
            } => {
                std::mem::size_of::<Self>()
                    + (**record).deep_size_of_children(context)
                    + record_type
                        .as_ref()
                        .map(|t| std::mem::size_of::<Self>() + (**t).deep_size_of_children(context))
                        .unwrap_or_default()
            }
            Self::Opaque {
                attributes,
                children,
                terms,
                ..
            } => {
                attributes
                    .iter()
                    .map(|p| std::mem::size_of_val(p) + p.1.len())
                    .sum::<usize>()
                    + children
                        .iter()
                        .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                        .sum::<usize>()
                    + terms
                        .iter()
                        .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                        .sum::<usize>()
            }
            */
            _ => 0,
        }
    }
}
