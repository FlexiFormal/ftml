use ftml_uris::{DocumentUri, Id, IsDomainUri, IsNarrativeUri, ModuleUri};
use smallvec::SmallVec;

use crate::{
    narrative::{documents::Document, elements::DocumentElementRef},
    terms::{Argument, BoundArgument, IsTerm, MaybeSequence, Term, Variable},
    utils::RefTree,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FreeOrBound {
    Free,
    Bound,
    Both,
}

impl Term {
    pub fn full_context(
        &self,
        get_doc: &mut impl FnMut(&DocumentUri) -> Option<Document>,
    ) -> rustc_hash::FxHashSet<ModuleUri> {
        let mut mods = rustc_hash::FxHashSet::default();
        let mut docs = rustc_hash::FxHashSet::default();
        self.full_context_i(&mut mods, get_doc, &mut docs);
        mods
    }

    #[must_use]
    pub fn free_variables(&self) -> smallvec::SmallVec<&Variable, 4> {
        let mut vars = smallvec::SmallVec::<&Variable, _>::new();
        let _ = self.has_free_i(&mut smallvec::SmallVec::new(), &mut |var| {
            if !vars.iter().any(|v| v.name() == var.name()) {
                vars.push(var);
            }
            false
        });
        //self.free_vars_i(&mut vars, &mut smallvec::SmallVec::new());
        vars
    }

    pub fn has_free_such_that(&self, mut f: impl FnMut(&Variable) -> bool) -> bool {
        self.has_free_i(&mut smallvec::SmallVec::new(), &mut f)
    }

    #[must_use]
    pub fn all_variables(&self) -> smallvec::SmallVec<(&Variable, FreeOrBound), 4> {
        let mut vars = smallvec::SmallVec::new();
        self.all_vars_i(&mut vars, &mut smallvec::SmallVec::new());
        vars
    }

    /// #### Panics
    #[must_use]
    pub fn fresh_variable(&self, prefix: &Id, num: Option<u16>) -> (Variable, Option<u16>) {
        let prefix = prefix.as_ref();
        let mut frees = self
            .free_variables()
            .into_iter()
            .filter_map(|v| {
                if v.name().starts_with(prefix) {
                    let rest = &v.name()[prefix.len()..];
                    if rest.is_empty() && num.is_none_or(|i| i == 0) {
                        Some(0)
                    } else if let Some(rest) = rest.strip_prefix('_') {
                        rest.parse::<u16>().ok().and_then(|i| {
                            num.map_or(Some(i), |num| if num >= i { Some(i) } else { None })
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<SmallVec<_, 2>>();
        frees.sort_unstable();
        let mut current = num.unwrap_or_default();
        for f in frees {
            if current != f {
                break;
            }
            current += 1;
        }
        let name = if current == 0 {
            prefix.parse::<Id>().expect("shouldn't be possible")
        } else {
            format!("{prefix}_{current}")
                .parse::<Id>()
                .expect("shouldn't be possible")
        };
        (
            Variable::Name {
                name,
                notated: None,
            },
            if current == 0 { None } else { Some(current) },
        )
    }

    fn full_context_i(
        &self,
        mods: &mut rustc_hash::FxHashSet<ModuleUri>,
        get_doc: &mut impl FnMut(&DocumentUri) -> Option<Document>,
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
                }
            }
            o => {
                for t in o.subterms() {
                    t.full_context_i(mods, get_doc, all_docs);
                }
            }
        }
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
                    a.free_vars_i(vars, ctx);
                }
            }
            Self::Bound(b) => {
                b.head.free_vars_i(vars, ctx);
                let mut added = 0;
                for a in &b.arguments {
                    a.free_vars_i(vars, ctx, &mut added);
                }
                for _ in 0..added {
                    let _ = ctx.pop();
                }
            }
        }
    }

    fn has_free_i<'t>(
        &'t self,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
        f: &mut impl FnMut(&'t Variable) -> bool,
    ) -> bool {
        match self {
            Self::Var { variable, .. }
                if !ctx.contains(&variable.name())
                    //&& !vars.iter().any(|v| v.name() == variable.name())
                    =>
            {
                f(variable)
            }
            Self::Opaque(o) => {
                o.terms.iter().any(|t| t.has_free_i(ctx, f))
            }
            Self::Field(fld) => fld.record.has_free_i(ctx,f),
            Self::Var { .. } | Self::Symbol { .. } | Self::Label { .. } | Self::Number(_) => false,
            Self::Application(a) => {
                a.head.has_free_i(ctx,f) ||
                a.arguments.iter().any(|a| a.has_free_i(ctx,f))
            }
            Self::Bound(b) => {
                b.head.has_free_i(ctx, f) || {
                    let mut added = 0;
                    for a in &b.arguments {
                        if a.has_free_i(ctx, f,&mut added) {
                            return true
                        }
                    }
                    for _ in 0..added {
                        let _ = ctx.pop();
                    }
                    false
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
                    a.all_vars_i(vars, ctx);
                }
            }
            Self::Bound(b) => {
                b.head.all_vars_i(vars, ctx);
                let mut added = 0;
                for a in &b.arguments {
                    a.all_vars_i(vars, ctx, &mut added);
                }
                for _ in 0..added {
                    let _ = ctx.pop();
                }
            }
        }
    }
}

impl Argument {
    #[must_use]
    pub fn free_variables(&self) -> smallvec::SmallVec<&Variable, 4> {
        let mut vars = smallvec::SmallVec::<&Variable, _>::new();
        let _ = self.has_free_i(&mut smallvec::SmallVec::new(), &mut |var| {
            if !vars.iter().any(|v| v.name() == var.name()) {
                vars.push(var);
            }
            false
        });
        //self.free_vars_i(&mut vars, &mut smallvec::SmallVec::new());
        vars
    }

    pub fn has_free_such_that(&self, mut f: impl FnMut(&Variable) -> bool) -> bool {
        self.has_free_i(&mut smallvec::SmallVec::new(), &mut f)
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
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => {
                t.free_vars_i(vars, ctx);
            }
            Self::Sequence(MaybeSequence::Seq(ts)) => {
                for t in ts {
                    t.free_vars_i(vars, ctx);
                }
            }
        }
    }
    fn has_free_i<'t>(
        &'t self,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
        f: &mut impl FnMut(&'t Variable) -> bool,
    ) -> bool {
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => t.has_free_i(ctx, f),
            Self::Sequence(MaybeSequence::Seq(ts)) => ts.iter().any(|t| t.has_free_i(ctx, f)),
        }
    }

    fn all_vars_i<'t>(
        &'t self,
        vars: &mut smallvec::SmallVec<(&'t Variable, FreeOrBound), 4>,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
    ) {
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => {
                t.all_vars_i(vars, ctx);
            }
            Self::Sequence(MaybeSequence::Seq(ts)) => {
                for t in ts {
                    t.all_vars_i(vars, ctx);
                }
            }
        }
    }
}

impl BoundArgument {
    #[must_use]
    pub fn free_variables(&self) -> smallvec::SmallVec<&Variable, 4> {
        let mut vars = smallvec::SmallVec::<&Variable, _>::new();
        let _ = self.has_free_i(
            &mut smallvec::SmallVec::new(),
            &mut |var| {
                if !vars.iter().any(|v| v.name() == var.name()) {
                    vars.push(var);
                }
                false
            },
            &mut 0,
        );
        //self.free_vars_i(&mut vars, &mut smallvec::SmallVec::new());
        vars
    }

    pub fn has_free_such_that(&self, mut f: impl FnMut(&Variable) -> bool) -> bool {
        self.has_free_i(&mut smallvec::SmallVec::new(), &mut f, &mut 0)
    }

    #[must_use]
    pub fn all_variables(&self) -> smallvec::SmallVec<(&Variable, FreeOrBound), 4> {
        let mut vars = smallvec::SmallVec::new();
        self.all_vars_i(&mut vars, &mut smallvec::SmallVec::new(), &mut 0);
        vars
    }
    fn free_vars_i<'t>(
        &'t self,
        vars: &mut smallvec::SmallVec<&'t Variable, 4>,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
        added: &mut usize,
    ) {
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => {
                t.free_vars_i(vars, ctx);
            }
            Self::Sequence(MaybeSequence::Seq(ts)) => {
                for t in ts {
                    t.free_vars_i(vars, ctx);
                }
            }
            Self::Bound(v) | Self::BoundSeq(MaybeSequence::One(v)) => {
                if let Some(tp) = &v.tp {
                    tp.free_vars_i(vars, ctx);
                }
                if let Some(df) = &v.df {
                    df.free_vars_i(vars, ctx);
                }
                *added += 1;
                ctx.push(v.var.name());
            }
            Self::BoundSeq(MaybeSequence::Seq(vs)) => {
                for v in vs {
                    if let Some(tp) = &v.tp {
                        tp.free_vars_i(vars, ctx);
                    }
                    if let Some(df) = &v.df {
                        df.free_vars_i(vars, ctx);
                    }
                    *added += 1;
                    ctx.push(v.var.name());
                }
            }
        }
    }

    fn has_free_i<'t>(
        &'t self,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
        f: &mut impl FnMut(&'t Variable) -> bool,
        added: &mut usize,
    ) -> bool {
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => t.has_free_i(ctx, f),
            Self::Sequence(MaybeSequence::Seq(ts)) => ts.iter().any(|t| t.has_free_i(ctx, f)),
            Self::Bound(v) | Self::BoundSeq(MaybeSequence::One(v)) => {
                if let Some(tp) = &v.tp
                    && tp.has_free_i(ctx, f)
                {
                    return true;
                }
                if let Some(df) = &v.df
                    && df.has_free_i(ctx, f)
                {
                    return true;
                }
                *added += 1;
                ctx.push(v.var.name());
                false
            }
            Self::BoundSeq(MaybeSequence::Seq(vs)) => {
                for v in vs {
                    if let Some(tp) = &v.tp
                        && tp.has_free_i(ctx, f)
                    {
                        return true;
                    }
                    if let Some(df) = &v.df
                        && df.has_free_i(ctx, f)
                    {
                        return true;
                    }
                    *added += 1;
                    ctx.push(v.var.name());
                }
                false
            }
        }
    }

    fn all_vars_i<'t>(
        &'t self,
        vars: &mut smallvec::SmallVec<(&'t Variable, FreeOrBound), 4>,
        ctx: &mut smallvec::SmallVec<&'t str, 4>,
        added: &mut usize,
    ) {
        match self {
            Self::Simple(t) | Self::Sequence(MaybeSequence::One(t)) => {
                t.all_vars_i(vars, ctx);
            }
            Self::Sequence(MaybeSequence::Seq(ts)) => {
                for t in ts {
                    t.all_vars_i(vars, ctx);
                }
            }
            Self::Bound(var) | Self::BoundSeq(MaybeSequence::One(var)) => {
                *added += 1;
                ctx.push(var.var.name());
                if let Some(v) = vars.iter_mut().find(|(v, _)| v.name() == var.var.name()) {
                    if v.1 == FreeOrBound::Free {
                        v.1 = FreeOrBound::Both;
                    }
                } else {
                    vars.push((&var.var, FreeOrBound::Bound));
                }
            }
            Self::BoundSeq(MaybeSequence::Seq(vs)) => {
                for var in vs {
                    *added += 1;
                    ctx.push(var.var.name());
                    if let Some(v) = vars.iter_mut().find(|(v, _)| v.name() == var.var.name()) {
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
}
