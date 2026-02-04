use crate::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, MaybeSequence, Term, Variable,
};

impl Term {
    #[must_use]
    pub fn similar(&self, other: &Self) -> bool {
        if *self == *other {
            return true;
        }
        match (self, other) {
            (Self::Symbol { uri: a, .. }, Self::Symbol { uri: b, .. }) => *a == *b,
            (Self::Var { variable: a, .. }, Self::Var { variable: b, .. }) => a.similar(b),
            (Self::Application(a), Self::Application(b)) => a.similar(b),
            (Self::Bound(a), Self::Bound(b)) => a.similar(b),
            (Self::Field(a), Self::Field(b)) => {
                a.record.similar(&b.record)
                    && a.record_type
                        .as_ref()
                        .is_none_or(|a| b.record_type.as_ref().is_none_or(|b| a.similar(b)))
                    && a.key == b.key
            }
            (
                Self::Label {
                    name: a,
                    df: da,
                    tp: ta,
                },
                Self::Label {
                    name: b,
                    df: db,
                    tp: tb,
                },
            ) => {
                *a == *b
                    && ((da.is_none() && db.is_none())
                        || da
                            .as_ref()
                            .is_some_and(|a| db.as_ref().is_some_and(|b| a.similar(b))))
                    && ((ta.is_none() && tb.is_none())
                        || ta
                            .as_ref()
                            .is_some_and(|a| tb.as_ref().is_some_and(|b| a.similar(b))))
            }
            (Self::Opaque(a), Self::Opaque(b)) => {
                a.terms.len() == b.terms.len()
                    && a.terms
                        .iter()
                        .zip(b.terms.iter())
                        .all(|(a, b)| a.similar(b))
            }
            _ => false,
        }
    }
}
impl Variable {
    fn similar(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name { name: a, .. }, Self::Name { name: b, .. }) => *a == *b,
            (Self::Ref { declaration: a, .. }, Self::Ref { declaration: b, .. }) => *a == *b,
            _ => false,
        }
    }
}
impl ApplicationTerm {
    fn similar(&self, other: &Self) -> bool {
        self.head.similar(&other.head)
            && self.arguments.len() == other.arguments.len()
            && self
                .arguments
                .iter()
                .zip(other.arguments.iter())
                .all(|(a, b)| a.similar(b))
    }
}

impl BindingTerm {
    fn similar(&self, other: &Self) -> bool {
        self.head.similar(&other.head)
            && self.arguments.len() == other.arguments.len()
            && self
                .arguments
                .iter()
                .zip(other.arguments.iter())
                .all(|(a, b)| a.similar(b))
    }
}

impl Argument {
    fn similar(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Simple(a), Self::Simple(b)) => a.similar(b),
            (Self::Sequence(MaybeSequence::One(a)), Self::Sequence(MaybeSequence::One(b))) => {
                a.similar(b)
            }
            (Self::Sequence(MaybeSequence::Seq(a)), Self::Sequence(MaybeSequence::Seq(b))) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a.similar(b))
            }
            _ => false,
        }
    }
}
impl BoundArgument {
    fn similar(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Simple(a), Self::Simple(b)) => a.similar(b),
            (Self::Sequence(MaybeSequence::One(a)), Self::Sequence(MaybeSequence::One(b))) => {
                a.similar(b)
            }
            (Self::Sequence(MaybeSequence::Seq(a)), Self::Sequence(MaybeSequence::Seq(b))) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a.similar(b))
            }
            (Self::Bound(a), Self::Bound(b)) => a.var.similar(&b.var),
            (Self::BoundSeq(MaybeSequence::One(a)), Self::BoundSeq(MaybeSequence::One(b))) => {
                a.var.similar(&b.var)
            }
            (Self::BoundSeq(MaybeSequence::Seq(a)), Self::BoundSeq(MaybeSequence::Seq(b))) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a.var.similar(&b.var))
            }
            _ => false,
        }
    }
}
