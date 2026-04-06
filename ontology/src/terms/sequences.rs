use ftml_uris::Id;

use crate::terms::{ApplicationTerm, Argument, MaybeSequence, Term, Variable};

// SAFETY: MAP_DUMMY is valid Id
pub static MAP_DUMMY: std::sync::LazyLock<Id> =
    std::sync::LazyLock::new(|| unsafe { "MAP_DUMMY".parse().unwrap_unchecked() });

pub enum Sequence<'t> {
    Var(&'t Variable),
    SequenceExpression(&'t [Term]),
    Map(Box<Self>, &'t Term),
    Concatenation(Vec<Self>),
}

impl Sequence<'_> {
    pub fn to_term(&self) -> Term {
        match self {
            Self::Var(v) => Term::Var {
                variable: (*v).clone(),
                presentation: None,
            },
            Self::SequenceExpression(ts) => Term::into_seq(ts.iter().cloned()),
            Self::Map(s, f) => Term::Application(ApplicationTerm::new(
                ftml_uris::metatheory::SEQUENCE_MAP.clone().into(),
                Box::new([
                    Argument::Simple(s.to_term()),
                    Argument::Simple((*f).clone()),
                ]),
                None,
            )),
            Self::Concatenation(ts) => Term::Application(ApplicationTerm::new(
                ftml_uris::metatheory::SEQUENCE_CONC.clone().into(),
                Box::new([Argument::Sequence(MaybeSequence::Seq(
                    ts.iter().map(Self::to_term).collect(),
                ))]),
                None,
            )),
        }
    }
    #[must_use]
    pub fn is_concrete(&self) -> bool {
        match self {
            Self::Var(_) => false,
            Self::SequenceExpression(_) => true,
            Self::Map(s, _) => s.is_concrete(),
            Self::Concatenation(v) => v.iter().all(Self::is_concrete),
        }
    }

    pub fn is_concrete_or(&self, or: &mut impl FnMut(&Variable) -> bool) -> bool {
        match self {
            Self::Var(v) => or(v),
            Self::SequenceExpression(_) => true,
            Self::Map(s, _) => s.is_concrete_or(or),
            Self::Concatenation(v) => v.iter().all(|e| e.is_concrete_or(or)),
        }
    }
    #[must_use]
    pub fn to_concrete(&self) -> Option<Vec<Term>> {
        match self {
            Self::Var(_) => None,
            Self::SequenceExpression(es) => Some(es.to_vec()),
            Self::Map(seq, f) => seq.to_concrete().map(|seq| {
                seq.into_iter()
                    .map(|a| {
                        Term::Application(ApplicationTerm::new(
                            (*f).clone(),
                            Box::new([Argument::Simple(a)]),
                            None,
                        ))
                    })
                    .collect()
            }),
            Self::Concatenation(v) => {
                let mut ret = Vec::new();
                for v in v {
                    ret.extend(v.to_concrete()?);
                }
                Some(ret)
            }
        }
    }

    #[must_use]
    pub fn to_concrete_or(
        &self,
        or: &mut impl FnMut(&Variable) -> Option<Vec<Term>>,
    ) -> Option<Vec<Term>> {
        match self {
            Self::Var(v) => or(v),
            Self::SequenceExpression(es) => Some(es.to_vec()),
            Self::Map(seq, f) => seq.to_concrete_or(or).map(|seq| {
                seq.into_iter()
                    .map(|a| {
                        Term::Application(ApplicationTerm::new(
                            (*f).clone(),
                            Box::new([Argument::Simple(a)]),
                            None,
                        ))
                    })
                    .collect()
            }),
            Self::Concatenation(v) => {
                let mut ret = Vec::new();
                for v in v {
                    ret.extend(v.to_concrete_or(or)?);
                }
                Some(ret)
            }
        }
    }
}

pub enum SequenceType<'t> {
    Var(&'t Variable),
    SequenceExpression(&'t [Term]),
    Map(Sequence<'t>, &'t Term),
    SeqType(&'t Term, Option<&'t [Term]>),
}

impl Term {
    #[must_use]
    pub const fn is_sequence_variable(&self) -> bool {
        matches!(
            self,
            Self::Var {
                variable: Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
                ..
            }
        )
    }
    #[must_use]
    pub fn is_sequence(&self) -> bool {
        self.is_sequence_i(false)
    }
    fn is_sequence_i(&self, force: bool) -> bool {
        if self.is_sequence_variable() {
            return true;
        }
        if force
            && let Self::Var {
                variable: v @ Variable::Ref { .. },
                ..
            } = self
        {
            return true;
        }
        let Self::Application(app) = self else {
            return false;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return false;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION
            && let [Argument::Sequence(MaybeSequence::Seq(_))] = &*app.arguments
        {
            true
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_MAP {
            if let [_, Argument::Simple(_)] = &*app.arguments {
                true
            } else {
                false
            }
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_CONC {
            if let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
                && seq.iter().all(Self::is_sequence)
            {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    #[must_use]
    pub fn as_sequence(&self) -> Option<Sequence<'_>> {
        self.as_sequence_i(false)
    }
    fn as_sequence_i(&self, force: bool) -> Option<Sequence<'_>> {
        if force
            && let Self::Var {
                variable: v @ Variable::Ref { .. },
                ..
            } = self
        {
            return Some(Sequence::Var(v));
        }
        if let Self::Var {
            variable:
                v @ Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
            ..
        } = self
        {
            return Some(Sequence::Var(v));
        }
        let Self::Application(app) = self else {
            return None;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return None;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION
            && let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
        {
            Some(Sequence::SequenceExpression(seq))
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_MAP {
            if let [
                Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                Argument::Simple(f),
            ] = &*app.arguments
                && let Some(seq) = seq.as_sequence_i(true)
            {
                Some(Sequence::Map(Box::new(seq), f))
            } else if let [
                Argument::Sequence(MaybeSequence::Seq(seq)),
                Argument::Simple(f),
            ] = &*app.arguments
            {
                Some(Sequence::Map(
                    Box::new(Sequence::SequenceExpression(seq)),
                    f,
                ))
            } else {
                None
            }
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_CONC {
            if let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
                && seq.iter().all(|s| s.is_sequence_i(force))
            {
                Some(Sequence::Concatenation(
                    seq.iter().filter_map(Self::as_sequence).collect(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn into_seq(seqs: impl Iterator<Item = Self>) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::SEQUENCE_EXPRESSION.clone(),
                presentation: None,
            },
            Box::new([Argument::Sequence(MaybeSequence::Seq(seqs.collect()))]),
            None,
        ))
    }

    #[must_use]
    pub fn as_sequence_type(&self) -> Option<SequenceType<'_>> {
        if let Self::Application(app) = self
            && let Self::Symbol { uri, .. } = &app.head
        {
            if *uri == *ftml_uris::metatheory::SEQUENCE_TYPE
                && let [Argument::Simple(t)] = &*app.arguments
            {
                Some(SequenceType::SeqType(t, None))
            } else if *uri == *ftml_uris::metatheory::RANGED_SEQUENCE_TYPE
                && let [
                    Argument::Simple(t),
                    Argument::Sequence(MaybeSequence::Seq(range)),
                ] = &*app.arguments
            {
                Some(SequenceType::SeqType(t, Some(&**range)))
            } else if *uri == *ftml_uris::metatheory::SEQUENCE_MAP
                && let [
                    Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                    Argument::Simple(f),
                ] = &*app.arguments
            {
                Some(SequenceType::Map(seq.as_sequence()?, f))
            } else {
                None
            }
        } else if let Self::Var {
            variable:
                v @ Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
            ..
        } = self
        {
            // TODO check that variable is inhabitable?
            Some(SequenceType::Var(v))
        } else {
            None
        }
    }

    #[must_use]
    pub fn into_seq_type(self) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::SEQUENCE_TYPE.clone(),
                presentation: None,
            },
            Box::new([Argument::Simple(self)]),
            None,
        ))
    }

    #[must_use]
    pub fn into_ranged_seq_type(self, range: impl IntoIterator<Item = Self>) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::RANGED_SEQUENCE_TYPE.clone(),
                presentation: None,
            },
            Box::new([
                Argument::Simple(self),
                Argument::Sequence(MaybeSequence::Seq(range.into_iter().collect())),
            ]),
            None,
        ))
    }
}
