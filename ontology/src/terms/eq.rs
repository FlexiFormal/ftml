use crate::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
    Variable,
};

pub type Alpha<'t> = smallvec::SmallVec<(&'t str, &'t Variable), 1>;

fn alpha_arg<'t>(lhs: &'t Argument, rhs: &'t Argument, alpha: &mut Alpha<'t>) -> bool {
    match (lhs, rhs) {
        (Argument::Simple(lhs), Argument::Simple(rhs))
        | (
            Argument::Sequence(MaybeSequence::One(lhs)),
            Argument::Sequence(MaybeSequence::One(rhs)),
        ) => lhs.alpha_equal_under(rhs, alpha),
        (
            Argument::Sequence(MaybeSequence::Seq(lhs)),
            Argument::Sequence(MaybeSequence::Seq(rhs)),
        ) if lhs.len() == rhs.len() => lhs
            .iter()
            .zip(rhs.iter())
            .all(|(lhs, rhs)| lhs.alpha_equal_under(rhs, alpha)),
        _ => false,
    }
}
fn alpha_barg<'t>(
    lhs: &'t BoundArgument,
    rhs: &'t BoundArgument,
    alpha: &mut Alpha<'t>,
) -> Option<usize> {
    macro_rules! ret {
        ($e:expr) => {
            if $e { Some(0) } else { None }
        };
    }
    match (lhs, rhs) {
        (BoundArgument::Simple(lhs), BoundArgument::Simple(rhs))
        | (
            BoundArgument::Sequence(MaybeSequence::One(lhs)),
            BoundArgument::Sequence(MaybeSequence::One(rhs)),
        ) => ret!(lhs.alpha_equal_under(rhs, alpha)),
        (
            BoundArgument::Sequence(MaybeSequence::Seq(lhs)),
            BoundArgument::Sequence(MaybeSequence::Seq(rhs)),
        ) if lhs.len() == rhs.len() => ret!(
            lhs.iter()
                .zip(rhs.iter())
                .all(|(lhs, rhs)| lhs.alpha_equal_under(rhs, alpha))
        ),
        (BoundArgument::Bound(lhs), BoundArgument::Bound(rhs))
        | (
            BoundArgument::BoundSeq(MaybeSequence::One(lhs)),
            BoundArgument::BoundSeq(MaybeSequence::One(rhs)),
        ) => {
            if alpha_cv(lhs, rhs, alpha) {
                Some(1)
            } else {
                None
            }
        }
        (
            BoundArgument::BoundSeq(MaybeSequence::Seq(lhs)),
            BoundArgument::BoundSeq(MaybeSequence::Seq(rhs)),
        ) if lhs.len() == rhs.len() => {
            if lhs
                .iter()
                .zip(rhs.iter())
                .all(|(a, b)| alpha_cv(a, b, alpha))
            {
                Some(lhs.len())
            } else {
                None
            }
        }
        _ => None,
    }
}
fn alpha_cv<'t>(lhs: &'t ComponentVar, rhs: &'t ComponentVar, alpha: &mut Alpha<'t>) -> bool {
    match (lhs.tp.as_ref(), rhs.tp.as_ref()) {
        (Some(lhs), Some(rhs)) => {
            if !lhs.alpha_equal_under(rhs, alpha) {
                return false;
            }
        }
        (None, None) => (),
        _ => return false,
    }
    match (lhs.df.as_ref(), rhs.df.as_ref()) {
        (Some(lhs), Some(rhs)) => {
            if !lhs.alpha_equal_under(rhs, alpha) {
                return false;
            }
        }
        (None, None) => (),
        _ => return false,
    }
    alpha.push((lhs.var.name(), &rhs.var));
    true
}

impl Term {
    #[must_use]
    pub fn alpha_equal(&self, rhs: &Self) -> bool {
        self.alpha_equal_under(rhs, &mut Alpha::default())
    }

    pub fn alpha_equal_under<'t>(&'t self, rhs: &'t Self, alpha: &mut Alpha<'t>) -> bool {
        if self == rhs {
            return true;
        }
        match (self, rhs) {
            (Self::Var { variable: v1, .. }, Self::Var { variable: v2, .. }) => {
                v1.name() == v2.name()
                    || alpha.iter().any(|(a, b)| {
                        (*a == v1.name() && b.name() == v2.name())
                            || (b.name() == v1.name() && *a == v2.name())
                    })
            }
            (Self::Application(a), Self::Application(b))
                if a.arguments.len() == b.arguments.len() =>
            {
                a.head.alpha_equal_under(&b.head, alpha)
                    && a.arguments
                        .iter()
                        .zip(b.arguments.iter())
                        .all(|(a, b)| alpha_arg(a, b, alpha))
            }
            (Self::Bound(a), Self::Bound(b)) if a.arguments.len() == b.arguments.len() => {
                let mut pop = 0;
                if !a.head.alpha_equal_under(&b.head, alpha)
                    || a.arguments
                        .iter()
                        .zip(b.arguments.iter())
                        .any(|(a, b)| alpha_barg(a, b, alpha).inspect(|i| pop += i).is_none())
                {
                    return false;
                }
                for _ in 0..pop {
                    alpha.pop();
                }
                true
            }
            (Self::Field(a), Self::Field(b)) => {
                a.record.alpha_equal_under(&b.record, alpha) && a.key == b.key
            }
            (
                Self::Label {
                    name: na,
                    df: da,
                    tp: ta,
                },
                Self::Label {
                    name: nb,
                    df: db,
                    tp: tb,
                },
            ) if *na == *nb => {
                match (da, db) {
                    (Some(a), Some(b)) => {
                        if !a.alpha_equal_under(b, alpha) {
                            return false;
                        }
                    }
                    (None, None) => (),
                    _ => return false,
                }
                match (ta, tb) {
                    (Some(a), Some(b)) => a.alpha_equal_under(b, alpha),
                    (None, None) => true,
                    _ => false,
                }
            }
            (Self::Number(a), Self::Number(b)) => a == b,
            _ => false,
        }
    }

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
