use std::hint::unreachable_unchecked;

use crate::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
    Variable,
};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};

mod __sealed {
    use ftml_uris::{DocumentElementUri, Id, SymbolUri};

    pub trait Sealed {}
    impl Sealed for SymbolUri {}
    impl Sealed for Id {}
    impl Sealed for DocumentElementUri {}
}

pub struct Bound<'s> {
    pub var: &'s Variable,
    pub tp: &'s Term,
    pub body: &'s Term,
    pub is_sequence: bool,
}

impl Term {
    #[inline]
    pub fn is(&self, v: &impl IntoTerm) -> bool {
        v.term_is(self)
    }
    pub fn unapply<'s>(&'s self, v: &impl IntoTerm) -> Option<&'s [Argument]> {
        match self {
            Self::Application(a) => {
                if v.term_is(&a.head) {
                    Some(&a.arguments)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn unbind<'s>(&'s self, v: &impl IntoTerm) -> Option<Bound<'s>> {
        match self {
            Self::Bound(b) => {
                if v.term_is(&b.head)
                    && let [
                        b @ (BoundArgument::Bound(ComponentVar { tp: Some(_), .. })
                        | BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                            tp: Some(_),
                            ..
                        }))),
                        BoundArgument::Simple(body),
                    ] = &*b.arguments
                {
                    let (var, tp, is_sequence) = match b {
                        BoundArgument::Bound(ComponentVar {
                            var, tp: Some(tp), ..
                        }) => (var, tp, false),
                        BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                            var,
                            tp: Some(tp),
                            ..
                        })) => (var, tp, true),
                        // SAFETY: pattern match above
                        _ => unsafe { unreachable_unchecked() },
                    };
                    Some(Bound {
                        var,
                        tp,
                        body,
                        is_sequence,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub trait IntoTerm: __sealed::Sealed + Into<Term> {
    fn term_is(&self, t: &Term) -> bool;
    fn apply_tms(self, tms: impl IntoIterator<Item = Term>) -> Term {
        Term::Application(ApplicationTerm::new(
            self.into(),
            tms.into_iter().map(Argument::Simple).collect(),
            None,
        ))
    }
    fn simple_bind(self, var: Variable, tp: Option<Term>, df: Option<Term>, body: Term) -> Term {
        Term::Bound(BindingTerm::new(
            self.into(),
            Box::new([
                BoundArgument::Bound(ComponentVar { var, tp, df }),
                BoundArgument::Simple(body),
            ]),
            None,
        ))
    }
}
impl From<SymbolUri> for Term {
    #[inline]
    fn from(value: SymbolUri) -> Self {
        Self::Symbol {
            uri: value,
            presentation: None,
        }
    }
}
impl From<DocumentElementUri> for Variable {
    #[inline]
    fn from(value: DocumentElementUri) -> Self {
        Self::Ref {
            declaration: value,
            is_sequence: None,
        }
    }
}
impl From<DocumentElementUri> for Term {
    #[inline]
    fn from(value: DocumentElementUri) -> Self {
        Self::Var {
            variable: value.into(),
            presentation: None,
        }
    }
}
impl From<Id> for Variable {
    #[inline]
    fn from(value: Id) -> Self {
        Self::Name {
            name: value,
            notated: None,
        }
    }
}
impl From<Id> for Term {
    #[inline]
    fn from(value: Id) -> Self {
        Self::Var {
            variable: value.into(),
            presentation: None,
        }
    }
}
impl From<Variable> for Term {
    #[inline]
    fn from(value: Variable) -> Self {
        Self::Var {
            variable: value,
            presentation: None,
        }
    }
}

impl IntoTerm for SymbolUri {
    #[inline]
    fn term_is(&self, t: &Term) -> bool {
        matches!(t,Term::Symbol { uri, .. } if *uri == *self)
    }
}
impl IntoTerm for Id {
    fn term_is(&self, t: &Term) -> bool {
        match t {
            Term::Var {
                variable: Variable::Name { name, .. },
                ..
            } => name == self,
            Term::Var {
                variable: Variable::Ref { declaration, .. },
                ..
            } => declaration.name.last() == self.as_ref(),
            _ => false,
        }
    }
}

impl IntoTerm for DocumentElementUri {
    fn term_is(&self, t: &Term) -> bool {
        match t {
            Term::Var {
                variable: Variable::Name { name, .. },
                ..
            } => name.as_ref() == self.name.last(),
            Term::Var {
                variable: Variable::Ref { declaration, .. },
                ..
            } => declaration.name.last() == self.name.last(),
            _ => false,
        }
    }
}
