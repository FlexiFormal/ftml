use std::fmt::Write;

use super::opaque::Opaque;
use super::{BoundArgument, arguments::Argument, variables::Variable};
use crate::terms::VarOrSym;
use crate::utils::TreeIter;
use ftml_uris::{Id, SymbolUri, UriName};

/// The type of FTML expressions.
///
/// Similarly to
/// [<span style="font-variant:small-caps;">OpenMath</span>](https://openmath.org),
/// FTML expressions are foundation-independent, but more expressive by hardcoding
/// [Theories-as-Types]()-like record "types".
#[derive(Clone, Hash, PartialEq, Eq)]
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    Application {
        #[cfg_attr(feature = "typescript", tsify(type = "Term"))]
        head: Box<Self>,
        arguments: Box<[Argument]>,
        #[cfg_attr(feature = "serde", serde(default))]
        presentation: Option<VarOrSym>,
    },
    /// A *binding* application with `head` as operator, `arguments`
    /// being either variable bindings or arbitrary expression arguments,
    /// and `body` being the (final) expression *in which* the variables are bound
    /// (e.g. $\int_{t=0}^\infty f(t) \mathrm dt$)
    Bound {
        #[cfg_attr(feature = "typescript", tsify(type = "Term"))]
        head: Box<Self>,
        arguments: Box<[BoundArgument]>,
        #[cfg_attr(feature = "typescript", tsify(type = "Term"))]
        body: Box<Self>,
        #[cfg_attr(feature = "serde", serde(default))]
        presentation: Option<VarOrSym>,
    },
    /// Record projection; the field named `key` in the record `record`.
    /// The optional `record_type` ideally references the type in which field names
    /// can be looked up.
    Field {
        #[cfg_attr(feature = "typescript", tsify(type = "Term"))]
        record: Box<Self>,
        key: UriName,
        /// does not count as a subterm
        #[cfg_attr(feature = "typescript", tsify(type = "Term | undefined"))]
        #[cfg_attr(feature = "serde", serde(default))]
        record_type: Option<Box<Self>>,
        #[cfg_attr(feature = "serde", serde(default))]
        presentation: Option<VarOrSym>,
    },
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
    Opaque {
        tag: Id,
        attributes: Box<[(Id, Box<str>)]>,
        children: Box<[Opaque]>,
        #[cfg_attr(feature = "typescript", tsify(type = "Term[]"))]
        terms: Box<[Self]>,
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
            Self::Application {
                head, arguments, ..
            } => Self::Application {
                head,
                arguments,
                presentation: Some(pres),
            },
            Self::Bound {
                head,
                arguments,
                body,
                ..
            } => Self::Bound {
                head,
                arguments,
                body,
                presentation: Some(pres),
            },
            Self::Field {
                record,
                key,
                record_type,
                ..
            } => Self::Field {
                record,
                key,
                record_type,
                presentation: Some(pres),
            },
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
            Self::Application { head, arguments,.. } => ExprChildrenIter::App(Some(head), arguments),
            Self::Bound {
                head,
                arguments,
                body,..
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
            Self::Opaque { terms, .. } => ExprChildrenIter::Slice(terms.iter()),
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
        Term::Field {
            record,
            key,
            record_type: None,
            ..
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
        Term::Application {
            head, arguments, ..
        } => f
            .debug_struct("OMA")
            .field("head", head)
            .field("arguments", arguments)
            .finish(),
        Term::Bound {
            head,
            arguments,
            body,
            ..
        } => f
            .debug_struct("OMBIND")
            .field("head", head)
            .field("arguments", arguments)
            .field("body", body)
            .finish(),
        Term::Opaque {
            tag,
            attributes,
            children,
            terms,
        } => {
            write!(f, "<{tag}")?;
            for (k, v) in attributes {
                write!(f, " {k}=\"{v}\"")?;
            }
            f.write_str(">\n")?;
            for t in children {
                writeln!(f, "{t}")?;
            }
            f.write_str("<terms>\n")?;
            for t in terms {
                fmt::<LONG>(t, f)?;
                f.write_char('\n')?;
            }
            write!(f, "</{tag}>")
        }
        Term::Field {
            record,
            key,
            record_type,
            ..
        } => f
            .debug_struct("Field")
            .field("name", key)
            .field("record", record)
            .field("type", record_type)
            .finish(),
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

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Term {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Application {
                head, arguments, ..
            } => {
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
            Self::Label { df, tp, .. } => {
                tp.as_ref()
                    .map(|t| std::mem::size_of::<Self>() + (**t).deep_size_of_children(context))
                    .unwrap_or_default()
                    + df.as_ref()
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
            _ => 0,
        }
    }
}
