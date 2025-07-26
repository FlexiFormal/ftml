use std::fmt::Write;

use super::opaque::Opaque;
use super::{BoundArgument, arguments::Argument, variables::Variable};
use crate::utils::TreeIter;
use ftml_uris::{ModuleUri, SymbolUri, UriName};

/// The type of FTML expressions.
///
/// Similarly to
/// [<span style="font-variant:small-caps;">OpenMath</span>](https://openmath.org),
/// FTML expressions are foundation-independent, but more expressive by hardcoding
/// [Theories-as-Types]()-like record "types".
#[derive(Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Term {
    /// A reference to a symbol (e.g. $\mathbb N$)
    Symbol(SymbolUri),
    // A reference to a module (e.g. $\mathbb N$)
    //Module(ModuleUri),
    /// A reference to a (bound) variable (e.g. $x$)
    Var(Variable),
    /// An application of `head` to `arguments` (e.g. $n + m$)
    Application {
        head: Box<Self>,
        arguments: Box<[Argument]>,
    },
    /// A *binding* application with `head` as operator, `arguments`
    /// being either variable bindings or arbitrary expression arguments,
    /// and `body` being the (final) expression *in which* the variables are bound
    /// (e.g. $\int_{t=0}^\infty f(t) \mathrm dt$)
    Bound {
        head: Box<Self>,
        arguments: Box<[BoundArgument]>,
        body: Box<Self>,
    },
    /// Record projection; the field named `key` in the record `record`.
    /// The optional `record_type` ideally references the type in which field names
    /// can be looked up.
    Field {
        record: Box<Term>,
        key: UriName,
        /// does not count as a subterm
        record_type: Option<Box<Term>>,
    },
    /// A non-alpha-renamable variable
    Label {
        name: UriName,
        df: Option<Box<Term>>,
        tp: Option<Box<Term>>,
    },
    /// An opaque/informal expression; may contain formal islands, which are collected in
    /// `expressions`.
    Opaque {
        tag: String,
        attributes: Box<[(Box<str>, Box<str>)]>,
        children: Box<[Opaque]>,
        expressions: Box<[Term]>,
    },
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
            if let Self::Symbol(s) = e {
                Some(s)
            } else {
                None
            }
        })
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
            Self::Symbol(_)
            | Self::Var(_)
            //| Self::Module(_)
            | Self::Label {
                df: None, tp: None, ..
            } => ExprChildrenIter::E,
            Self::Application { head, arguments } => ExprChildrenIter::App(Some(head), arguments),
            Self::Bound {
                head,
                arguments,
                body,
            } => ExprChildrenIter::Bound(Some(head), arguments, body),
            Self::Label {
                df: Some(df),
                tp: Some(tp),
                ..
            } => ExprChildrenIter::Two(tp, df),
            Self::Field { record, .. } => ExprChildrenIter::One(record),
            Self::Label { df: Some(t), .. } | Self::Label { tp: Some(t), .. } => {
                ExprChildrenIter::One(t)
            }
            Self::Opaque { expressions, .. } => ExprChildrenIter::Slice(expressions.iter()),
        }
    }
}

fn fmt<const LONG: bool>(e: &Term, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match e {
        Term::Symbol(s) if LONG => write!(f, "Sym({s})"),
        Term::Symbol(s) => write!(f, "Sym({})", s.name()),
        Term::Var(Variable::Name(n)) => write!(f, "Var({n})"),
        Term::Var(Variable::Ref { declaration, .. }) if LONG => write!(f, "Var({declaration})"),
        Term::Var(Variable::Ref { declaration, .. }) => write!(f, "Var({})", declaration.name()),
        Term::Field {
            record,
            key,
            record_type: None,
        } => {
            fmt::<LONG>(record, f)?;
            f.write_char('.')?;
            std::fmt::Debug::fmt(key, f)
        }
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
        _ => f.write_str("(opaque)"), // TODO
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
                        Argument::Simple(tm) | Argument::Sequence(either::Left(tm)) => {
                            return Some(tm);
                        }
                        Argument::Sequence(either::Right(seq)) => {
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
                        BoundArgument::Simple(f) | BoundArgument::Sequence(either::Left(f)) => {
                            return Some(f);
                        }
                        BoundArgument::Sequence(either::Right(seq)) => {
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
