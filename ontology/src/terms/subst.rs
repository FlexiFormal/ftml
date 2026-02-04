use std::borrow::Cow;

use ftml_uris::{DocumentUri, IsDomainUri, IsNarrativeUri, ModuleUri};

use crate::{
    narrative::{documents::Document, elements::DocumentElementRef},
    terms::{
        ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, IsTerm, MaybeSequence,
        OpaqueTerm, RecordFieldTerm, Term, Variable,
    },
    utils::RefTree,
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
        self.subst(&[(s.as_ref(), t)], &mut Vec::new())
    }
}
impl<S: AsRef<str>> std::ops::Div<(S, &Self)> for Term {
    type Output = Self;
    fn div(self, (s, t): (S, &Self)) -> Self::Output {
        match self.subst(&[(s.as_ref(), t)], &mut Vec::new()) {
            Cow::Borrowed(_) => self,
            Cow::Owned(t) => t,
        }
    }
}
impl<'t, S: AsRef<str>, T: AsRef<Term>> std::ops::Div<&[(S, T)]> for &'t Term {
    type Output = Cow<'t, Term>;
    fn div(self, rhs: &[(S, T)]) -> Self::Output {
        if rhs.is_empty() {
            Cow::Borrowed(self)
        } else {
            self.subst(rhs, &mut Vec::new())
        }
    }
}
impl<S: AsRef<str>, T: AsRef<Self>> std::ops::Div<&[(S, T)]> for Term {
    type Output = Self;
    fn div(self, rhs: &[(S, T)]) -> Self::Output {
        if rhs.is_empty() {
            return self;
        }
        match self.subst(rhs, &mut Vec::new()) {
            Cow::Borrowed(_) => self,
            Cow::Owned(t) => t,
        }
    }
}

impl Argument {
    fn subst<'s, S: AsRef<str> + 's, T: AsRef<Term>>(
        &self,
        substs: &'s [(S, T)],
        shadowed: &mut Vec<&'s str>,
    ) -> Cow<'_, Self> {
        match self {
            Self::Simple(t) => match t.subst(substs, shadowed) {
                Cow::Borrowed(_) => Cow::Borrowed(self),
                Cow::Owned(t) => Cow::Owned(Self::Simple(t)),
            },
            Self::Sequence(MaybeSequence::One(t)) => match t.subst(substs, shadowed) {
                Cow::Borrowed(_) => Cow::Borrowed(self),
                Cow::Owned(t) => Cow::Owned(Self::Sequence(MaybeSequence::One(t))),
            },
            Self::Sequence(MaybeSequence::Seq(ts)) => {
                let mut changed = false;
                let ret = ts
                    .iter()
                    .map(|t| {
                        let t = t.subst(substs, shadowed);
                        changed = changed || matches!(&t, Cow::Owned(_));
                        t
                    })
                    .collect::<Vec<_>>();
                if changed {
                    Cow::Owned(Self::Sequence(MaybeSequence::Seq(
                        ret.into_iter().map(Cow::into_owned).collect(),
                    )))
                } else {
                    Cow::Borrowed(self)
                }
            }
        }
    }
}

impl ComponentVar {
    fn subst<'s, S: AsRef<str> + 's, T: AsRef<Term>>(
        &self,
        substs: &'s [(S, T)],
        shadowed: &mut Vec<&'s str>,
    ) -> (Cow<'_, Self>, bool) {
        let Self { var, tp, df } = self;
        let mut is_shadowed = false;
        if let Some(s) = substs.iter().find_map(|(n, _)| {
            if n.as_ref() == var.name() {
                Some(n)
            } else {
                None
            }
        }) {
            shadowed.push(s.as_ref());
            is_shadowed = true;
            if shadowed.len() == substs.len() {
                return (Cow::Borrowed(self), true);
            }
        }
        let mut changed = false;
        let tp = tp.as_ref().map(|t| {
            let s = t.subst(substs, shadowed);
            changed = matches!(s, Cow::Owned(_));
            s
        });
        let df = df.as_ref().map(|t| {
            let s = t.subst(substs, shadowed);
            changed = matches!(s, Cow::Owned(_));
            s
        });
        if changed {
            (
                Cow::Owned(Self {
                    var: var.clone(),
                    tp: tp.map(Cow::into_owned),
                    df: df.map(Cow::into_owned),
                }),
                is_shadowed,
            )
        } else {
            (Cow::Borrowed(self), is_shadowed)
        }
    }
}

impl BoundArgument {
    fn subst<'s, S: AsRef<str> + 's, T: AsRef<Term>>(
        &self,
        substs: &'s [(S, T)],
        shadowed: &mut Vec<&'s str>,
    ) -> (Cow<'_, Self>, usize) {
        match self {
            Self::Simple(t) => match t.subst(substs, shadowed) {
                Cow::Borrowed(_) => (Cow::Borrowed(self), 0),
                Cow::Owned(t) => (Cow::Owned(Self::Simple(t)), 0),
            },
            Self::Sequence(MaybeSequence::One(t)) => match t.subst(substs, shadowed) {
                Cow::Borrowed(_) => (Cow::Borrowed(self), 0),
                Cow::Owned(t) => (Cow::Owned(Self::Sequence(MaybeSequence::One(t))), 0),
            },
            Self::Bound(cv) => {
                let (r, b) = cv.subst(substs, shadowed);
                (
                    match r {
                        Cow::Borrowed(_) => Cow::Borrowed(self),
                        Cow::Owned(cv) => Cow::Owned(Self::Bound(cv)),
                    },
                    b.into(),
                )
            }
            Self::BoundSeq(MaybeSequence::One(cv)) => {
                let (r, b) = cv.subst(substs, shadowed);
                (
                    match r {
                        Cow::Borrowed(_) => Cow::Borrowed(self),
                        Cow::Owned(cv) => Cow::Owned(Self::BoundSeq(MaybeSequence::One(cv))),
                    },
                    b.into(),
                )
            }
            Self::Sequence(MaybeSequence::Seq(ts)) => {
                let mut changed = false;
                let ret = ts
                    .iter()
                    .map(|t| {
                        let t = t.subst(substs, shadowed);
                        changed = changed || matches!(&t, Cow::Owned(_));
                        t
                    })
                    .collect::<Vec<_>>();
                if changed {
                    (
                        Cow::Owned(Self::Sequence(MaybeSequence::Seq(
                            ret.into_iter().map(Cow::into_owned).collect(),
                        ))),
                        0,
                    )
                } else {
                    (Cow::Borrowed(self), 0)
                }
            }
            Self::BoundSeq(MaybeSequence::Seq(vs)) => {
                let mut changed = false;
                let mut has_shadowed = 0;
                let ret = vs
                    .iter()
                    .map(|t| {
                        let (t, b) = t.subst(substs, shadowed);
                        if b {
                            has_shadowed += 1;
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
                    )
                } else {
                    (Cow::Borrowed(self), has_shadowed)
                }
            }
        }
    }
}

impl ApplicationTerm {
    fn subst<'s, S: AsRef<str> + 's, T: AsRef<Term>>(
        &self,
        substs: &'s [(S, T)],
        shadowed: &mut Vec<&'s str>,
    ) -> Cow<'_, Self> {
        let head = self.head.subst(substs, shadowed);
        let mut changed = matches!(&head, Cow::Owned(_));
        if changed {
            return Cow::Owned(Self::new(
                head.into_owned(),
                self.arguments
                    .iter()
                    .map(|a| a.subst(substs, shadowed).into_owned())
                    .collect(),
                self.presentation.clone(),
            ));
        }
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
            Cow::Owned(Self::new(
                head.into_owned(),
                arguments.into_iter().map(Cow::into_owned).collect(),
                self.presentation.clone(),
            ))
        } else {
            Cow::Borrowed(self)
        }
    }
}

impl BindingTerm {
    fn subst<'s, S: AsRef<str> + 's, T: AsRef<Term>>(
        &self,
        substs: &'s [(S, T)],
        shadowed: &mut Vec<&'s str>,
    ) -> Cow<'_, Self> {
        let head = self.head.subst(substs, shadowed);
        let mut has_shadowed = 0;
        macro_rules! subst {
            ($e:expr) => {{
                let (r, s) = $e.subst(substs, shadowed);
                has_shadowed += s;
                r
            }};
        }
        macro_rules! clear {
            () => {
                for _ in 0..has_shadowed {
                    shadowed.pop();
                }
            };
        }
        let mut changed = matches!(&head, Cow::Owned(_));
        if changed {
            let arguments = self
                .arguments
                .iter()
                .map(|a| subst!(a).into_owned())
                .collect();
            clear!();
            return Cow::Owned(Self::new(
                head.into_owned(),
                arguments,
                self.presentation.clone(),
            ));
        }
        let arguments = self
            .arguments
            .iter()
            .map(|a| {
                let r = subst!(a);
                changed = changed || matches!(&r, Cow::Owned(_));
                r
            })
            .collect::<Vec<_>>();
        clear!();
        if changed {
            Cow::Owned(Self::new(
                head.into_owned(),
                arguments.into_iter().map(Cow::into_owned).collect(),
                self.presentation.clone(),
            ))
        } else {
            Cow::Borrowed(self)
        }
    }
}

impl Term {
    fn subst<'s, S: AsRef<str> + 's, T: AsRef<Self>>(
        &self,
        substs: &'s [(S, T)],
        shadowed: &mut Vec<&'s str>,
    ) -> Cow<'_, Self> {
        match self {
            Self::Var { variable, .. } if !shadowed.contains(&variable.name()) => substs
                .iter()
                .find_map(|(n, t)| {
                    if n.as_ref() == variable.name() {
                        Some(t.as_ref().clone())
                    } else {
                        None
                    }
                })
                .map_or(Cow::Borrowed(self), Cow::Owned),
            Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => Cow::Borrowed(self),
            Self::Application(app) => match app.subst(substs, shadowed) {
                Cow::Borrowed(_) => Cow::Borrowed(self),
                Cow::Owned(app) => Cow::Owned(Self::Application(app)),
            },
            Self::Bound(app) => match app.subst(substs, shadowed) {
                Cow::Borrowed(_) => Cow::Borrowed(self),
                Cow::Owned(app) => Cow::Owned(Self::Bound(app)),
            },
            Self::Field(f) => match f.record.subst(substs, shadowed) {
                Cow::Borrowed(_) => Cow::Borrowed(self),
                Cow::Owned(rec) => Cow::Owned(Self::Field(RecordFieldTerm::new(
                    rec,
                    f.key.clone(),
                    f.record_type.clone(),
                    f.presentation.clone(),
                ))),
            },
            Self::Label { name, df, tp } => {
                let df = df.as_ref().map(|t| t.subst(substs, shadowed));
                let tp = tp.as_ref().map(|t| t.subst(substs, shadowed));
                if df.as_ref().is_some_and(|t| matches!(t, Cow::Owned(_)))
                    || tp.as_ref().is_some_and(|t| matches!(t, Cow::Owned(_)))
                {
                    Cow::Owned(Self::Label {
                        name: name.clone(),
                        tp: tp.map(|t| Box::new(t.into_owned())),
                        df: df.map(|t| Box::new(t.into_owned())),
                    })
                } else {
                    Cow::Borrowed(self)
                }
            }
            Self::Opaque(o) => {
                let mut changed = false;
                let nts = o
                    .terms
                    .iter()
                    .map(|t| {
                        let r = t.subst(substs, shadowed);
                        if matches!(&r, Cow::Owned(_)) {
                            changed = true;
                        }
                        r
                    })
                    .collect::<Vec<_>>();
                if changed {
                    Cow::Owned(Self::Opaque(OpaqueTerm::new(
                        o.node.clone(),
                        nts.into_iter().map(Cow::into_owned).collect(),
                    )))
                } else {
                    Cow::Borrowed(self)
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FreeOrBound {
    Free,
    Bound,
    Both,
}

impl Term {
    pub fn full_context(
        &self,
        get_doc: &mut dyn FnMut(&DocumentUri) -> Option<Document>,
    ) -> rustc_hash::FxHashSet<ModuleUri> {
        let mut mods = rustc_hash::FxHashSet::default();
        let mut docs = rustc_hash::FxHashSet::default();
        self.full_context_i(&mut mods, get_doc, &mut docs);
        mods
    }
    fn full_context_i(
        &self,
        mods: &mut rustc_hash::FxHashSet<ModuleUri>,
        get_doc: &mut dyn FnMut(&DocumentUri) -> Option<Document>,
        all_docs: &mut rustc_hash::FxHashSet<Document>,
    ) {
        match self {
            Self::Symbol { uri, .. } => {
                if !mods.contains(uri.module_uri()) {
                    mods.insert(uri.module_uri().clone());
                }
            }
            Self::Var {
                variable: Variable::Ref { declaration, .. },
                ..
            } => {
                if all_docs.get(declaration.document_uri()).is_none()
                    && let Some(d) = get_doc(declaration.document_uri())
                {
                    for e in d.dfs() {
                        match e {
                            DocumentElementRef::Module { module: uri, .. }
                            | DocumentElementRef::UseModule { uri, .. } => {
                                if !mods.contains(uri) {
                                    mods.insert(uri.clone());
                                }
                            }
                            _ => (),
                        }
                    }
                    all_docs.insert(d);
                } else {
                    return;
                }
            }
            o => {
                for t in o.subterms() {
                    t.full_context_i(mods, get_doc, all_docs);
                }
            }
        }
    }

    #[must_use]
    pub fn free_variables(&self) -> smallvec::SmallVec<&Variable, 4> {
        let mut vars = smallvec::SmallVec::new();
        self.free_vars_i(&mut vars, &mut smallvec::SmallVec::new());
        vars
    }

    #[must_use]
    pub fn all_variables(&self) -> smallvec::SmallVec<(&Variable, FreeOrBound), 4> {
        let mut vars = smallvec::SmallVec::new();
        self.all_vars_i(&mut vars, &mut smallvec::SmallVec::new());
        vars
    }

    fn free_vars_i<'t>(
        &'t self,
        vars: &mut smallvec::SmallVec<&'t Variable, 4>,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
    ) {
        match self {
            Self::Var { variable, .. }
                if !ctx.contains(&variable.name())
                    && !vars.iter().any(|v| v.name() == variable.name()) =>
            {
                vars.push(variable);
            }
            Self::Opaque(o) => {
                for t in &o.terms {
                    t.free_vars_i(vars, ctx);
                }
            }
            Self::Field(f) => f.record.free_vars_i(vars, ctx),
            Self::Var { .. } | Self::Symbol { .. } | Self::Label { .. } | Self::Number(_) => (),
            Self::Application(a) => {
                a.head.free_vars_i(vars, ctx);
                for a in &a.arguments {
                    match a {
                        Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                            t.free_vars_i(vars, ctx);
                        }
                        Argument::Sequence(MaybeSequence::Seq(ts)) => {
                            for t in ts {
                                t.free_vars_i(vars, ctx);
                            }
                        }
                    }
                }
            }
            Self::Bound(b) => {
                b.head.free_vars_i(vars, ctx);
                let mut added = 0;
                for a in &b.arguments {
                    match a {
                        BoundArgument::Simple(t)
                        | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                            t.free_vars_i(vars, ctx);
                        }
                        BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                            for t in ts {
                                t.free_vars_i(vars, ctx);
                            }
                        }
                        BoundArgument::Bound(v)
                        | BoundArgument::BoundSeq(MaybeSequence::One(v)) => {
                            if let Some(tp) = &v.tp {
                                tp.free_vars_i(vars, ctx);
                            }
                            if let Some(df) = &v.df {
                                df.free_vars_i(vars, ctx);
                            }
                            added += 1;
                            ctx.push(v.var.name());
                        }
                        BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                            for v in vs {
                                if let Some(tp) = &v.tp {
                                    tp.free_vars_i(vars, ctx);
                                }
                                if let Some(df) = &v.df {
                                    df.free_vars_i(vars, ctx);
                                }
                                added += 1;
                                ctx.push(v.var.name());
                            }
                        }
                    }
                }
                for _ in 0..added {
                    let _ = ctx.pop();
                }
            }
        }
    }

    fn all_vars_i<'t>(
        &'t self,
        vars: &mut smallvec::SmallVec<(&'t Variable, FreeOrBound), 4>,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
    ) {
        match self {
            Self::Var { variable, .. } => {
                let free = !ctx.contains(&variable.name());
                if let Some(v) = vars.iter_mut().find(|(v, _)| v.name() == variable.name()) {
                    if (free && v.1 == FreeOrBound::Bound) || (!free && v.1 == FreeOrBound::Free) {
                        v.1 = FreeOrBound::Both;
                    }
                } else {
                    vars.push((
                        variable,
                        if free {
                            FreeOrBound::Free
                        } else {
                            FreeOrBound::Bound
                        },
                    ));
                }
            }
            Self::Opaque(o) => {
                for t in &o.terms {
                    t.all_vars_i(vars, ctx);
                }
            }
            Self::Field(f) => f.record.all_vars_i(vars, ctx),
            Self::Symbol { .. } | Self::Label { .. } | Self::Number(_) => (),
            Self::Application(a) => {
                a.head.all_vars_i(vars, ctx);
                for a in &a.arguments {
                    match a {
                        Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                            t.all_vars_i(vars, ctx);
                        }
                        Argument::Sequence(MaybeSequence::Seq(ts)) => {
                            for t in ts {
                                t.all_vars_i(vars, ctx);
                            }
                        }
                    }
                }
            }
            Self::Bound(b) => {
                b.head.all_vars_i(vars, ctx);
                let mut added = 0;
                for a in &b.arguments {
                    match a {
                        BoundArgument::Simple(t)
                        | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                            t.all_vars_i(vars, ctx);
                        }
                        BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                            for t in ts {
                                t.all_vars_i(vars, ctx);
                            }
                        }
                        BoundArgument::Bound(var)
                        | BoundArgument::BoundSeq(MaybeSequence::One(var)) => {
                            added += 1;
                            ctx.push(var.var.name());
                            if let Some(v) =
                                vars.iter_mut().find(|(v, _)| v.name() == var.var.name())
                            {
                                if v.1 == FreeOrBound::Free {
                                    v.1 = FreeOrBound::Both;
                                }
                            } else {
                                vars.push((&var.var, FreeOrBound::Bound));
                            }
                        }
                        BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                            for var in vs {
                                added += 1;
                                ctx.push(var.var.name());
                                if let Some(v) =
                                    vars.iter_mut().find(|(v, _)| v.name() == var.var.name())
                                {
                                    if v.1 == FreeOrBound::Free {
                                        v.1 = FreeOrBound::Both;
                                    }
                                } else {
                                    vars.push((&var.var, FreeOrBound::Bound));
                                }
                            }
                        }
                    }
                }
                for _ in 0..added {
                    let _ = ctx.pop();
                }
            }
        }
    }
}
