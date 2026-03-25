use std::fmt::Write;

use crate::terms::{Argument, BoundArgument, ComponentVar, MaybeSequence, Term, Variable};

impl std::fmt::Debug for Term {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symbol { uri, .. } => write!(f, "{uri}"),
            Self::Var {
                variable: Variable::Name { name, .. },
                ..
            } => write!(f, "V({name})"),
            Term::Var {
                variable: Variable::Ref { declaration, .. },
                ..
            } => write!(f, "V({declaration})"),
            Self::Number(n) => std::fmt::Debug::fmt(n, f),
            Self::Field(field) if field.record_type.is_none() => {
                write!(f, "({:?}).{}", field.record, field.key)
            }
            Self::Field(field) => {
                write!(
                    f,
                    "({:?} : {:?}).{}",
                    field.record,
                    field.record_type.as_ref().expect("pattern match"),
                    field.key
                )
            }
            Self::Label {
                name,
                df: None,
                tp: None,
            } => write!(f, "Label({name})"),
            Self::Label {
                name,
                df: Some(df),
                tp: Some(tp),
            } => f
                .debug_struct("Label")
                .field("", name)
                .field(":", tp)
                .field(":=", df)
                .finish(),

            Term::Label {
                name, tp: Some(tp), ..
            } => f
                .debug_struct("Label")
                .field("", name)
                .field(":", tp)
                .finish(),
            Term::Label {
                name, df: Some(df), ..
            } => f
                .debug_struct("Label")
                .field("", name)
                .field(":=", df)
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
                //.field("body", &b.body)
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
                    t.fmt(f)?;
                    f.write_char('\n')?;
                }
                write!(f, "</{}>", o.node.tag)
            }
        }
    }
}
pub(super) struct Short<'e>(pub &'e Term);
impl std::fmt::Debug for Short<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Term::Symbol { uri, .. } => write!(f, "\"{}\"", uri.name()),
            Term::Var {
                variable: Variable::Name { name, .. },
                ..
            } => write!(f, "V({name})"),
            Term::Var {
                variable: Variable::Ref { declaration, .. },
                ..
            } => write!(f, "V(..?{})", declaration.name()),
            Term::Number(n) => std::fmt::Debug::fmt(n, f),
            Term::Field(field) if field.record_type.is_none() => {
                write!(f, "({:?}).{}", Short(&field.record), field.key)
            }
            Term::Field(field) => {
                write!(
                    f,
                    "({:?} : {:?}).{}",
                    Short(&field.record),
                    Short(field.record_type.as_ref().expect("pattern match")),
                    field.key
                )
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
            } => f
                .debug_struct("Label")
                .field("", name)
                .field(":", &tp.debug_short())
                .field(":=", &df.debug_short())
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
            } => f
                .debug_struct("Label")
                .field("", name)
                .field(":=", &df.debug_short())
                .finish(),
            Term::Application(a) => {
                write!(f, "{:?}", a.head.debug_short())?;
                let mut tup = f.debug_list();
                for a in &a.arguments {
                    tup.entry(&ShortArg(a));
                }
                tup.finish()
            }
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
                    Short(t).fmt(f)?;
                    f.write_char('\n')?;
                }
                write!(f, "</{}>", o.node.tag)
            }
        }
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
                //f.write_char('[')?;
                let mut fl = f.debug_list();
                for t in s {
                    fl.entry(&t.debug_short());
                }
                fl.finish()
                //f.write_char(']')
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
            BoundArgument::Bound(cv) => ShortCV(cv).fmt(f),
            BoundArgument::BoundSeq(MaybeSequence::One(cv)) => {
                write!(f, "({:?})", ShortCV(cv))
            }
            BoundArgument::Sequence(MaybeSequence::Seq(s)) => {
                //f.write_char('[')?;
                let mut fl = f.debug_list();
                for t in s {
                    fl.entry(&t.debug_short());
                }
                fl.finish()
                //f.write_char(']')
            }
            BoundArgument::BoundSeq(MaybeSequence::Seq(s)) => {
                //f.write_char('[')?;
                let mut fl = f.debug_list();
                for v in s {
                    fl.entry(&ShortCV(v));
                }
                fl.finish()
                //f.write_char(']')
            }
        }
    }
}

struct ShortCV<'e>(&'e ComponentVar);
impl std::fmt::Debug for ShortCV<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ComponentVar { var, tp, df } = self.0;
        match var {
            Variable::Name { name, .. } => write!(f, "{{{name}")?,
            Variable::Ref { declaration, .. } => write!(f, "{{..{:?}", declaration.name())?,
        }

        if let Some(tp) = tp.as_ref() {
            write!(f, " : {:?}", tp.debug_short())?;
        }
        if let Some(df) = df.as_ref() {
            write!(f, " := {:?}", df.debug_short())?;
        }
        f.write_char('}')
    }
}
