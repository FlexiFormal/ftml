#![allow(clippy::cast_possible_truncation)]

use smallvec::SmallVec;

use crate::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathIndex {
    Head,
    Record,
    RecordType,
    LabelType,
    LabelDefiniens,
    VarType,
    VarDefiniens,
    Argument(u8),
    SequenceIndex(u8),
}
impl PathIndex {
    const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Head,
            1 => Self::Record,
            2 => Self::RecordType,
            3 => Self::LabelType,
            4 => Self::LabelDefiniens,
            5 => Self::VarType,
            6 => Self::VarDefiniens,
            i if !i.is_multiple_of(2) => Self::Argument((i - 7).saturating_div(2)),
            i => Self::SequenceIndex((i - 8).saturating_div(2)),
        }
    }
    const fn into_u8(self) -> u8 {
        match self {
            Self::Head => 0,
            Self::Record => 1,
            Self::RecordType => 2,
            Self::LabelType => 3,
            Self::LabelDefiniens => 4,
            Self::VarType => 5,
            Self::VarDefiniens => 6,
            Self::Argument(i) => i.saturating_mul(2).saturating_add(7),
            Self::SequenceIndex(i) => i.saturating_mul(2).saturating_add(8),
        }
    }
}
impl From<u8> for PathIndex {
    #[inline]
    fn from(value: u8) -> Self {
        Self::from_u8(value)
    }
}
impl From<PathIndex> for u8 {
    #[inline]
    fn from(value: PathIndex) -> Self {
        value.into_u8()
    }
}

#[derive(Clone, Debug, Default)]
pub struct TermPath(SmallVec<u8, 16>);
impl TermPath {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<PathIndex> {
        self.0.get(index).map(|u| PathIndex::from_u8(*u))
    }
    #[inline]
    pub fn set(&mut self, index: usize, value: PathIndex) {
        self.0[index] = value.into_u8();
    }
    #[inline]
    pub fn push(&mut self, value: PathIndex) {
        self.0.push(value.into_u8());
    }
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn pop(&mut self) {
        self.0.pop();
    }
    #[inline]
    #[must_use]
    pub const fn as_mut_ref(&mut self) -> TermPathMutRef<'_> {
        TermPathMutRef(&mut self.0, 0)
    }
}
pub struct TermPathMutRef<'s>(&'s mut SmallVec<u8, 16>, usize);
impl TermPathMutRef<'_> {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<PathIndex> {
        self.0.get(self.1 + index).map(|u| PathIndex::from_u8(*u))
    }
    #[inline]
    pub fn set(&mut self, index: usize, value: PathIndex) {
        self.0[self.1 + index] = value.into_u8();
    }
    #[inline]
    #[must_use]
    pub fn next(&self) -> Option<PathIndex> {
        self.get(0)
    }
    pub fn remove(&mut self, index: usize) {
        self.0.remove(self.1 + index);
    }
    pub const fn inc(&mut self) {
        self.1 += 1;
    }
    pub fn insert(&mut self, index: usize, value: PathIndex) {
        self.0.insert(self.1 + index, value.into_u8());
    }
    pub const fn copy(&mut self) -> TermPathMutRef<'_> {
        TermPathMutRef(self.0, self.1)
    }
}

impl Term {
    #[must_use]
    pub fn path_of_subterm2(&self, sub: &Self) -> Option<TermPath> {
        let mut ret = TermPath::default();
        if self.path_of_subterm_i2(sub, &mut ret, None) {
            Some(ret)
        } else {
            None
        }
    }

    #[must_use]
    pub fn path_of_subterm_with_ctx2(&self, sub: &Self) -> Option<(Vec<&ComponentVar>, TermPath)> {
        let mut ret = TermPath::default();
        let mut vars = Vec::new();
        if self.path_of_subterm_i2(sub, &mut ret, Some(&mut vars)) {
            Some((vars, ret))
        } else {
            None
        }
    }
    #[must_use]
    pub fn subterm_at_path2(&self, path: &TermPath) -> Option<(Vec<&ComponentVar>, &Self)> {
        let mut vars = Vec::new();
        self.subterm_at_path_i2(path, &mut vars, 0)
            .map(|t| (vars, t))
    }

    fn path_of_subterm_i2<'s>(
        &'s self,
        sub: &Self,
        ret: &mut TermPath,
        mut vars: Option<&mut Vec<&'s ComponentVar>>,
    ) -> bool {
        if self.similar(sub) {
            return true;
        }
        match self {
            Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => false,
            Self::Application(app) => app.path_of_subterm_i2(sub, ret, vars),
            Self::Bound(b) => {
                let (b, num) = b.path_of_subterm_i2(sub, ret, vars.as_deref_mut());
                if !b && let Some(vars) = vars {
                    for _ in 0..num {
                        vars.pop();
                    }
                }
                b
            }
            Self::Field(f) => {
                ret.push(PathIndex::Record);
                if f.record.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                    return true;
                }
                ret.pop();
                f.record_type.as_ref().is_some_and(|tp| {
                    ret.push(PathIndex::RecordType);
                    if tp.path_of_subterm_i2(sub, ret, vars) {
                        true
                    } else {
                        ret.pop();
                        false
                    }
                })
            }
            Self::Label { df, tp, .. } => {
                if let Some(t) = tp.as_ref() {
                    ret.push(PathIndex::LabelType);
                    if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                        return true;
                    }
                    ret.pop();
                }
                if let Some(t) = df.as_ref() {
                    ret.push(PathIndex::LabelDefiniens);
                    if t.path_of_subterm_i2(sub, ret, vars) {
                        return true;
                    }
                    ret.pop();
                }
                false
            }
            Self::Opaque(o) => {
                for (i, t) in o.terms.iter().enumerate() {
                    ret.push(PathIndex::Argument(i as u8));
                    if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                        return true;
                    }
                    ret.pop();
                }
                false
            }
        }
    }

    fn subterm_at_path_i2<'s>(
        &'s self,
        path: &TermPath,
        vars: &mut Vec<&'s ComponentVar>,
        index: usize,
    ) -> Option<&'s Self> {
        let Some(next) = path.get(index) else {
            return Some(self);
        };
        match self {
            //Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => None,
            Self::Application(app) => app.subterm_at_path_i2(path, vars, next, index),
            Self::Bound(b) => b.subterm_at_path_i2(path, vars, next, index),
            Self::Field(f) => match next {
                PathIndex::Record => f.record.subterm_at_path_i2(path, vars, index + 1),
                PathIndex::RecordType if let Some(tp) = f.record_type.as_ref() => {
                    tp.subterm_at_path_i2(path, vars, index + 1)
                }
                _ => None,
            },
            Self::Label { df, tp, .. } => match next {
                PathIndex::LabelType if let Some(tp) = tp.as_ref() => {
                    tp.subterm_at_path_i2(path, vars, index + 1)
                }
                PathIndex::LabelDefiniens if let Some(df) = df.as_ref() => {
                    df.subterm_at_path_i2(path, vars, index + 1)
                }
                _ => None,
            },
            Self::Opaque(o) if let PathIndex::Argument(i) = next => o
                .terms
                .get(i as usize)?
                .subterm_at_path_i2(path, vars, index + 1),
            _ => None,
        }
    }
}

impl ApplicationTerm {
    fn path_of_subterm_i2<'s>(
        &'s self,
        sub: &Term,
        ret: &mut TermPath,
        mut vars: Option<&mut Vec<&'s ComponentVar>>,
    ) -> bool {
        ret.push(PathIndex::Head);
        if self.head.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
            return true;
        }
        ret.pop();
        for (i, a) in self.arguments.iter().enumerate() {
            ret.push(PathIndex::Argument(i as u8));
            match a {
                Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                    if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                        return true;
                    }
                }
                Argument::Sequence(MaybeSequence::Seq(ts)) => {
                    for (j, t) in ts.iter().enumerate() {
                        ret.push(PathIndex::SequenceIndex(j as u8));
                        if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                            return true;
                        }
                        ret.pop();
                    }
                }
            }
            ret.pop();
        }
        false
    }
    fn subterm_at_path_i2<'s>(
        &'s self,
        path: &TermPath,
        vars: &mut Vec<&'s ComponentVar>,
        next: PathIndex,
        index: usize,
    ) -> Option<&'s Term> {
        match next {
            PathIndex::Head => self.head.subterm_at_path_i2(path, vars, index + 1),
            PathIndex::Argument(i) if let Some(arg) = self.arguments.get(i as usize) => match arg {
                Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                    t.subterm_at_path_i2(path, vars, index + 1)
                }
                Argument::Sequence(MaybeSequence::Seq(ts)) => {
                    let index = index + 1;
                    let Some(PathIndex::SequenceIndex(i)) = path.get(index) else {
                        return None;
                    };
                    ts.get(i as usize)?
                        .subterm_at_path_i2(path, vars, index + 1)
                }
            },
            _ => None,
        }
    }
}

impl BindingTerm {
    fn path_of_subterm_i2<'s>(
        &'s self,
        sub: &Term,
        ret: &mut TermPath,
        mut vars: Option<&mut Vec<&'s ComponentVar>>,
    ) -> (bool, usize) {
        ret.push(PathIndex::Head);
        if self.head.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
            return (true, 0);
        }
        ret.pop();
        let mut added_vars = 0;
        for (i, a) in self.arguments.iter().enumerate() {
            ret.push(PathIndex::Argument(i as u8));
            match a {
                BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                    if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                        return (true, added_vars);
                    }
                }
                BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                    for (j, t) in ts.iter().enumerate() {
                        ret.push(PathIndex::SequenceIndex(j as u8));
                        if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                            return (true, added_vars);
                        }
                        ret.pop();
                    }
                }
                BoundArgument::Bound(cv @ ComponentVar { tp, df, .. })
                | BoundArgument::BoundSeq(MaybeSequence::One(cv @ ComponentVar { tp, df, .. })) => {
                    if let Some(t) = tp.as_ref() {
                        ret.push(PathIndex::VarType);
                        if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                            return (true, added_vars);
                        }
                        ret.pop();
                    }
                    if let Some(t) = df.as_ref() {
                        ret.push(PathIndex::VarDefiniens);
                        if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                            return (true, added_vars);
                        }
                        ret.pop();
                    }
                    if let Some(vars) = vars.as_mut() {
                        vars.push(cv);
                        added_vars += 1;
                    }
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                    for (i, cv @ ComponentVar { tp, df, .. }) in vs.iter().enumerate() {
                        if tp.is_none() && df.is_none() {
                            continue;
                        }
                        ret.push(PathIndex::SequenceIndex(i as u8));
                        if let Some(t) = tp.as_ref() {
                            ret.push(PathIndex::VarType);
                            if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                                return (true, added_vars);
                            }
                            ret.pop();
                        }
                        if let Some(t) = df.as_ref() {
                            ret.push(PathIndex::VarDefiniens);
                            if t.path_of_subterm_i2(sub, ret, vars.as_deref_mut()) {
                                return (true, added_vars);
                            }
                            ret.pop();
                        }
                        ret.pop();
                        if let Some(vars) = vars.as_mut() {
                            vars.push(cv);
                            added_vars += 1;
                        }
                    }
                }
            }
            ret.pop();
        }
        (false, added_vars)
    }

    fn subterm_at_path_i2<'s>(
        &'s self,
        path: &TermPath,
        vars: &mut Vec<&'s ComponentVar>,
        next: PathIndex,
        index: usize,
    ) -> Option<&'s Term> {
        match next {
            PathIndex::Head => self.head.subterm_at_path_i2(path, vars, index + 1),
            PathIndex::Argument(i) if let Some(arg) = self.arguments.get(i as usize) => {
                for a in self.arguments.get(..i as usize).unwrap_or_default() {
                    match a {
                        BoundArgument::Bound(v)
                        | BoundArgument::BoundSeq(MaybeSequence::One(v)) => {
                            vars.push(v);
                        }
                        BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => vars.extend(vs.iter()),
                        _ => (),
                    }
                }
                match arg {
                    BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                        t.subterm_at_path_i2(path, vars, index + 1)
                    }
                    BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                        let index = index + 1;
                        let Some(PathIndex::SequenceIndex(i)) = path.get(index) else {
                            return None;
                        };
                        ts.get(i as usize)?
                            .subterm_at_path_i2(path, vars, index + 1)
                    }
                    BoundArgument::Bound(ComponentVar { tp, df, .. })
                    | BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                        tp, df, ..
                    })) => {
                        let index = index + 1;
                        let next = path.get(index)?;
                        match next {
                            PathIndex::VarType if let Some(tp) = tp => {
                                tp.subterm_at_path_i2(path, vars, index + 1)
                            }
                            PathIndex::VarDefiniens if let Some(df) = df => {
                                df.subterm_at_path_i2(path, vars, index + 1)
                            }
                            _ => None,
                        }
                    }
                    BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                        let index = index + 1;
                        let Some(PathIndex::SequenceIndex(i)) = path.get(index) else {
                            return None;
                        };
                        vars.extend(vs.get(..i as usize)?);
                        let ComponentVar { tp, df, .. } = vs.get(i as usize)?;
                        let index = index + 1;
                        let next = path.get(index)?;
                        match next {
                            PathIndex::VarType if let Some(tp) = tp => {
                                tp.subterm_at_path_i2(path, vars, index + 1)
                            }
                            PathIndex::VarDefiniens if let Some(df) = df => {
                                df.subterm_at_path_i2(path, vars, index + 1)
                            }
                            _ => None,
                        }
                    }
                }
            }
            _ => None,
        }
    }
}
