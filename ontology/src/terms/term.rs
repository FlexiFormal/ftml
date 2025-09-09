use super::{BoundArgument, arguments::Argument, variables::Variable};
use crate::terms::opaque::OpaqueNode;
use crate::terms::{VarOrSym, arguments::MaybeSequence};
use crate::utils::TreeIter;
use ftml_uris::{SymbolUri, UriName};
use std::fmt::Write;

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
        #[cfg_attr(feature = "serde", serde(default))]
        df: Option<Box<Self>>,
        #[cfg_attr(feature = "typescript", tsify(type = "Term | undefined"))]
        #[cfg_attr(feature = "serde", serde(default))]
        tp: Option<Box<Self>>,
    },
    /// An opaque/informal expression; may contain formal islands, which are collected in
    /// `expressions`.
    Opaque(OpaqueTerm),
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ApplicationTerm(pub(crate) triomphe::Arc<Application>);

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Application {
    pub head: Term,
    pub arguments: Box<[Argument]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub presentation: Option<VarOrSym>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) hash: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct BindingTerm(pub(crate) triomphe::Arc<Binding>);

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Binding {
    pub head: Term,
    pub arguments: Box<[BoundArgument]>,
    pub body: Term,
    #[cfg_attr(feature = "serde", serde(default))]
    pub presentation: Option<VarOrSym>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) hash: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct RecordFieldTerm(pub(crate) triomphe::Arc<RecordField>);

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct RecordField {
    pub record: Term,
    pub key: UriName,
    /// does not count as a subterm
    #[cfg_attr(feature = "serde", serde(default))]
    pub record_type: Option<Term>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub presentation: Option<VarOrSym>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) hash: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpaqueTerm(pub(crate) triomphe::Arc<Opaque>);

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Opaque {
    pub node: OpaqueNode,
    #[cfg_attr(feature = "serde", serde(default))]
    pub terms: Box<[Term]>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) hash: u64,
}

impl Term {
    /*#[must_use]
    #[inline]
    pub const fn normalize(self) -> Self {
        self
    }*/

    /// implements [`Debug`](std::fmt::Debug), but only prints the *names* of [`Uri`](ftml_uris::Uri)s
    #[inline]
    #[must_use]
    pub fn debug_short(&self) -> impl std::fmt::Debug {
        Short(self)
    }

    /// Iterates over all symbols occuring in this expression.
    #[inline]
    pub fn symbols(&self) -> impl Iterator<Item = &SymbolUri> {
        ExprChildrenIter::One(self).dfs().filter_map(|e| {
            if let Self::Symbol { uri, .. } = e {
                Some(uri)
            } else {
                None
            }
        })
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
                b.body.clone(),
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
}

impl crate::utils::RefTree for Term {
    type Child<'a>
        = &'a Self
    where
        Self: 'a;

    #[allow(refining_impl_trait_reachable)]
    fn tree_children(&self) -> ExprChildrenIter<'_> {
        match self {
            Self::Symbol{..}
            | Self::Var{..}
            //| Self::Module(_)
            | Self::Label {
                df: None, tp: None, ..
            } => ExprChildrenIter::E,
            Self::Application(a) => ExprChildrenIter::App(Some(&a.head), &a.arguments),
            Self::Bound(b) => ExprChildrenIter::Bound(Some(&b.head), &b.arguments, &b.body),
            Self::Label {
                df: Some(df),
                tp: Some(tp),
                ..
            } => ExprChildrenIter::Two(tp, df),
            Self::Field(f) => ExprChildrenIter::One(&f.record),
            Self::Label { df: Some(t), .. } | Self::Label { tp: Some(t), .. } => {
                ExprChildrenIter::One(t)
            }
            Self::Opaque(o) => ExprChildrenIter::Slice(o.terms.iter()),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn fmt<const LONG: bool>(e: &Term, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match e {
        Term::Symbol { uri, .. } if LONG => write!(f, "Sym({uri})"),
        Term::Symbol { uri, .. } => write!(f, "Sym({})", uri.name()),
        Term::Var {
            variable: Variable::Name {
                notated: Some(n), ..
            },
            ..
        } => write!(f, "Var({n})"),
        Term::Var {
            variable: Variable::Name { name, .. },
            ..
        } => write!(f, "Var({name})"),
        Term::Var {
            variable: Variable::Ref { declaration, .. },
            ..
        } if LONG => write!(f, "Var({declaration})"),
        Term::Var {
            variable: Variable::Ref { declaration, .. },
            ..
        } => write!(f, "Var({})", declaration.name()),
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
        Term::Application(a) => f
            .debug_struct("OMA")
            .field("head", &a.head)
            .field("arguments", &a.arguments)
            .finish(),
        Term::Bound(b) => f
            .debug_struct("OMBIND")
            .field("head", &b.head)
            .field("arguments", &b.arguments)
            .field("body", &b.body)
            .finish(),
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

pub enum ExprChildrenIter<'a> {
    E,
    App(Option<&'a Term>, &'a [Argument]),
    Bound(Option<&'a Term>, &'a [BoundArgument], &'a Term),
    Arg(&'a [Term], &'a [Argument]),
    BArg(&'a [Term], &'a [BoundArgument], &'a Term),
    One(&'a Term),
    Two(&'a Term, &'a Term),
    Slice(std::slice::Iter<'a, Term>),
}
impl<'a> Iterator for ExprChildrenIter<'a> {
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
            Self::Bound(head, args, b) => {
                if let Some(f) = head.take() {
                    return Some(f);
                }
                loop {
                    let Some(next) = args.first() else {
                        let b = *b;
                        *self = Self::E;
                        return Some(b);
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
                                    *self = Self::BArg(&seq[1..], args, b);
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
            Self::BArg(ls, rest, b) => {
                // SAFETY: only constructed with non-empty ls (see above)
                // and replaced by Self::Bound after emptying (below)
                let f = unsafe { ls.first().unwrap_unchecked() };
                *ls = &ls[1..];
                if ls.is_empty() {
                    *self = Self::Bound(None, rest, b);
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
