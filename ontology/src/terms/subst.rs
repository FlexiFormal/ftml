use std::borrow::Cow;

use ftml_uris::Id;
use smallvec::SmallVec;

use crate::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, OpaqueTerm,
    RecordFieldTerm, Term, Variable,
};

impl AsRef<Self> for Term {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'t, S: AsRef<str>> std::ops::Div<(S, &Term)> for &'t Term {
    type Output = Cow<'t, Term>;
    fn div(self, (s, t): (S, &Term)) -> Self::Output {
        self.subst(
            &mut smallvec::smallvec_inline![(s.as_ref(), Cow::Borrowed(t))],
            &mut Vec::new(),
        )
        .map_or_else(|| Cow::Borrowed(self), Cow::Owned)
    }
}
impl<S: AsRef<str>> std::ops::Div<(S, &Self)> for Term {
    type Output = Self;
    #[allow(clippy::option_if_let_else)]
    fn div(self, (s, t): (S, &Self)) -> Self::Output {
        if let Some(t) = self.subst(
            &mut smallvec::smallvec_inline![(s.as_ref(), Cow::Borrowed(t))],
            &mut Vec::new(),
        ) {
            t
        } else {
            self
        }
    }
}

impl<'t, S: AsRef<str>, T: AsRef<Term>> std::ops::Div<&[(S, T)]> for &'t Term {
    type Output = Cow<'t, Term>;
    fn div(self, rhs: &[(S, T)]) -> Self::Output {
        if rhs.is_empty() {
            return Cow::Borrowed(self);
        }
        let mut substs = rhs
            .iter()
            .map(|(s, t)| (s.as_ref(), Cow::Borrowed(t.as_ref())))
            .collect();
        self.subst(&mut substs, &mut Vec::new())
            .map_or(Cow::Borrowed(self), Cow::Owned)
    }
}
impl<S: AsRef<str>, T: AsRef<Self>> std::ops::Div<&[(S, T)]> for Term {
    type Output = Self;
    fn div(self, rhs: &[(S, T)]) -> Self::Output {
        if rhs.is_empty() {
            return self;
        }
        let mut substs = rhs
            .iter()
            .map(|(s, t)| (s.as_ref(), Cow::Borrowed(t.as_ref())))
            .collect();
        let r = self.subst(&mut substs, &mut Vec::new());
        drop(substs);
        r.unwrap_or(self)
    }
}

impl Term {
    fn subst<'s, 't: 's>(
        &'t self,
        substs: &mut SmallVec<(&'s str, Cow<'t, Self>), 1>,
        shadowed: &mut Vec<&'t str>,
    ) -> Option<Self> {
        match self {
            Self::Var { variable, .. } if !shadowed.contains(&variable.name()) => {
                substs.iter().rev().find_map(|(n, t)| {
                    if *n == variable.name() {
                        Some(t.as_ref().clone())
                    } else {
                        None
                    }
                })
            }
            Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => None,
            Self::Application(app) => app.subst(substs, shadowed).map(Self::Application),
            Self::Bound(app) => app.subst(substs, shadowed).map(Self::Bound),
            Self::Field(f) => f.record.subst(substs, shadowed).map(|rec| {
                Self::Field(RecordFieldTerm::new(
                    rec,
                    f.key.clone(),
                    f.record_type.clone(),
                    f.presentation.clone(),
                ))
            }),
            Self::Label { name, df, tp } => {
                let ndf = df.as_ref().map(|t| t.subst(substs, shadowed));
                let ntp = tp.as_ref().map(|t| t.subst(substs, shadowed));
                if ndf.as_ref().is_some_and(Option::is_some)
                    || ntp.as_ref().is_some_and(Option::is_some)
                {
                    Some(Self::Label {
                        name: name.clone(),
                        tp: ntp
                            .flatten()
                            .map_or_else(|| tp.clone(), |t| Some(Box::new(t))),
                        df: ndf
                            .flatten()
                            .map_or_else(|| df.clone(), |t| Some(Box::new(t))),
                    })
                } else {
                    None
                }
            }
            Self::Opaque(o) => {
                let mut changed = false;
                let nts = o
                    .terms
                    .iter()
                    .map(|t| {
                        t.subst(substs, shadowed).map_or(Cow::Borrowed(t), |t| {
                            changed = true;
                            Cow::Owned(t)
                        })
                    })
                    .collect::<Vec<_>>();
                if changed {
                    Some(Self::Opaque(OpaqueTerm::new(
                        o.node.clone(),
                        nts.into_iter().map(Cow::into_owned).collect(),
                    )))
                } else {
                    None
                }
            }
        }
    }
}

impl ApplicationTerm {
    fn subst<'s, 't: 's>(
        &'t self,
        substs: &mut SmallVec<(&'s str, Cow<'t, Term>), 1>,
        shadowed: &mut Vec<&'t str>,
    ) -> Option<Self> {
        if let Some(head) = self.head.subst(substs, shadowed) {
            return Some(Self::new(
                head,
                self.arguments
                    .iter()
                    .map(|a| a.subst(substs, shadowed).into_owned())
                    .collect(),
                self.presentation.clone(),
            ));
        }
        let mut changed = false;
        let arguments = self
            .arguments
            .iter()
            .map(|a| {
                let r = a.subst(substs, shadowed);
                changed = changed || matches!(&r, Cow::Owned(_));
                r
            })
            .collect::<Vec<_>>();
        if changed {
            Some(Self::new(
                self.head.clone(),
                arguments.into_iter().map(Cow::into_owned).collect(),
                self.presentation.clone(),
            ))
        } else {
            None
        }
    }
}

impl Argument {
    fn subst<'s, 't: 's>(
        &'t self,
        substs: &mut SmallVec<(&'s str, Cow<'t, Term>), 1>,
        shadowed: &mut Vec<&'t str>,
    ) -> Cow<'t, Self> {
        match self {
            Self::Simple(t) => t
                .subst(substs, shadowed)
                .map_or(Cow::Borrowed(self), |t| Cow::Owned(Self::Simple(t))),
            Self::Sequence(s) => subst_maybe_seq(s, substs, shadowed)
                .map_or(Cow::Borrowed(self), |n| Cow::Owned(Self::Sequence(n))),
        }
    }
}

impl BindingTerm {
    fn subst<'s, 't: 's>(
        &'t self,
        substs: &mut SmallVec<(&'s str, Cow<'t, Term>), 1>,
        shadowed: &mut Vec<&'t str>,
    ) -> Option<Self> {
        let head = self.head.subst(substs, shadowed);
        let mut has_shadowed = 0;
        let mut has_renamed = 0;
        macro_rules! subst {
            ($e:expr,$r:expr) => {{
                let (r, s, c) = $e.subst(substs, &self.arguments[$r + 1..], shadowed);
                has_shadowed += s;
                has_renamed += c;
                r
            }};
        }
        macro_rules! clear {
            () => {
                for _ in 0..has_shadowed {
                    shadowed.pop();
                }
                for _ in 0..has_renamed {
                    substs.pop();
                }
            };
        }
        if let Some(head) = head {
            let arguments = self
                .arguments
                .iter()
                .enumerate()
                .map(|(i, a)| subst!(a, i).into_owned())
                .collect();
            clear!();
            return Some(Self::new(head, arguments, self.presentation.clone()));
        }
        let mut changed = false;
        let arguments = self
            .arguments
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let r = subst!(a, i);
                changed = changed || matches!(&r, Cow::Owned(_));
                r
            })
            .collect::<Vec<_>>();
        clear!();
        if changed {
            Some(Self::new(
                self.head.clone(),
                arguments.into_iter().map(Cow::into_owned).collect(),
                self.presentation.clone(),
            ))
        } else {
            None
        }
    }
}

impl BoundArgument {
    fn subst<'s, 't: 's>(
        &'t self,
        substs: &mut SmallVec<(&'s str, Cow<'t, Term>), 1>,
        in_terms: &'t [Self],
        shadowed: &mut Vec<&'t str>,
    ) -> (Cow<'t, Self>, usize, usize) {
        match self {
            Self::Simple(t) => (
                t.subst(substs, shadowed)
                    .map_or(Cow::Borrowed(self), |t| Cow::Owned(Self::Simple(t))),
                0,
                0,
            ),
            Self::Sequence(s) => (
                subst_maybe_seq(s, substs, shadowed)
                    .map_or(Cow::Borrowed(self), |n| Cow::Owned(Self::Sequence(n))),
                0,
                0,
            ),
            Self::Bound(cv) => {
                let (r, b, c) = cv.subst(substs, in_terms, shadowed);
                (
                    match r {
                        Cow::Borrowed(_) => Cow::Borrowed(self),
                        Cow::Owned(cv) => Cow::Owned(Self::Bound(cv)),
                    },
                    b.into(),
                    c.into(),
                )
            }
            Self::BoundSeq(MaybeSequence::One(cv)) => {
                let (r, b, c) = cv.subst(substs, in_terms, shadowed);
                (
                    match r {
                        Cow::Borrowed(_) => Cow::Borrowed(self),
                        Cow::Owned(cv) => Cow::Owned(Self::BoundSeq(MaybeSequence::One(cv))),
                    },
                    b.into(),
                    c.into(),
                )
            }
            Self::BoundSeq(MaybeSequence::Seq(vs)) => {
                let mut changed = false;
                let mut has_shadowed = 0;
                let mut has_renamed = 0;
                let ret = vs
                    .iter()
                    .map(|t| {
                        let (t, b, c) = t.subst(substs, in_terms, shadowed);
                        if b {
                            has_shadowed += 1;
                        }
                        if c {
                            has_renamed += 1;
                        }
                        changed = changed || matches!(&t, Cow::Owned(_));
                        t
                    })
                    .collect::<Vec<_>>();
                if changed {
                    (
                        Cow::Owned(Self::BoundSeq(MaybeSequence::Seq(
                            ret.into_iter().map(Cow::into_owned).collect(),
                        ))),
                        has_shadowed,
                        has_renamed,
                    )
                } else {
                    (Cow::Borrowed(self), has_shadowed, has_renamed)
                }
            }
        }
    }
}

impl ComponentVar {
    fn subst<'s, 't: 's>(
        &'t self,
        substs: &mut SmallVec<(&'s str, Cow<'t, Term>), 1>,
        in_terms: &[BoundArgument],
        shadowed: &mut Vec<&'t str>,
    ) -> (Cow<'t, Self>, bool, bool) {
        fn needs_rename(
            name: &str,
            in_terms: &[BoundArgument],
            args: &[(&str, Cow<'_, Term>)],
        ) -> Option<Id> {
            let potential_captures = args
                .iter()
                .filter(|(_, t)| t.as_ref().has_free_such_that(|v| v.name() == name))
                .collect::<SmallVec<_, 1>>();
            if potential_captures.is_empty() {
                return None;
            }
            if in_terms.iter().any(|t| {
                t.has_free_such_that(|v| potential_captures.iter().any(|(n, _)| v.name() == *n))
            }) {
                let newname = format!("{name}'");
                Some(
                    needs_rename(&newname, in_terms, args)
                        .unwrap_or_else(|| newname.parse().expect("primes should be valid")),
                )
            } else {
                None
            }
        }
        let Self { var, tp, df } = self;
        let mut is_shadowed = false;
        if let Some(newname) = needs_rename(var.name(), in_terms, substs) {
            let tp = tp
                .as_ref()
                .map(|t| t.subst(substs, shadowed).unwrap_or_else(|| t.clone()));
            let df = df
                .as_ref()
                .map(|t| t.subst(substs, shadowed).unwrap_or_else(|| t.clone()));

            substs.push((
                var.name(),
                Cow::Owned(Term::Var {
                    variable: Variable::Name {
                        name: newname.clone(),
                        notated: None,
                    },
                    presentation: Some(crate::terms::VarOrSym::Var(var.clone())),
                }),
            ));
            return (
                Cow::Owned(Self {
                    var: Variable::Name {
                        name: newname,
                        notated: None,
                    },
                    tp,
                    df,
                }),
                false,
                true,
            );
        }
        if let Some(s) = substs.iter().find_map(|(n, _)| {
            if *n == var.name() {
                Some(var.name())
            } else {
                None
            }
        }) {
            shadowed.push(s);
            is_shadowed = true;
            if shadowed.len() == substs.len() {
                return (Cow::Borrowed(self), true, false);
            }
        }
        let mut changed = false;
        let tp = tp.as_ref().map(|t| {
            t.subst(substs, shadowed).map_or(Cow::Borrowed(t), |t| {
                changed = true;
                Cow::Owned(t)
            })
        });
        let df = df.as_ref().map(|t| {
            t.subst(substs, shadowed).map_or(Cow::Borrowed(t), |t| {
                changed = true;
                Cow::Owned(t)
            })
        });
        if changed {
            (
                Cow::Owned(Self {
                    var: var.clone(),
                    tp: tp.map(Cow::into_owned),
                    df: df.map(Cow::into_owned),
                }),
                is_shadowed,
                false,
            )
        } else {
            (Cow::Borrowed(self), is_shadowed, false)
        }
    }
}

fn subst_maybe_seq<'s, 't: 's>(
    ms: &'t MaybeSequence<Term>,
    substs: &mut SmallVec<(&'s str, Cow<'t, Term>), 1>,
    shadowed: &mut Vec<&'t str>,
) -> Option<MaybeSequence<Term>> {
    match ms {
        MaybeSequence::One(Term::Var { variable, .. }) if !shadowed.contains(&variable.name()) => {
            substs
                .iter()
                .find_map(|(n, t)| {
                    if *n == variable.name() {
                        Some(t.as_ref())
                    } else {
                        None
                    }
                })
                .map(|v| {
                    if let Term::Application(app) = v
                        && let Term::Symbol { uri, .. } = &app.head
                        && *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION
                        && let [Argument::Sequence(MaybeSequence::Seq(ts))] = &*app.arguments
                    {
                        MaybeSequence::Seq(ts.clone())
                    } else {
                        MaybeSequence::One(v.clone())
                    }
                })
        }
        MaybeSequence::One(t) => t.subst(substs, shadowed).map(MaybeSequence::One),
        MaybeSequence::Seq(ts) => {
            let mut changed = false;
            let ret = ts
                .iter()
                .map(|t| {
                    t.subst(substs, shadowed).map_or(Cow::Borrowed(t), |t| {
                        changed = true;
                        Cow::Owned(t)
                    })
                })
                .collect::<Vec<_>>();
            if changed {
                Some(MaybeSequence::Seq(
                    ret.into_iter().map(Cow::into_owned).collect(),
                ))
            } else {
                None
            }
        }
    }
}
