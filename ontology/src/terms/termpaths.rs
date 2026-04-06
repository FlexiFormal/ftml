#![allow(
    clippy::option_if_let_else,
    clippy::useless_let_if_seq,
    clippy::cast_possible_truncation
)]

use crate::{
    terms::{
        ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
    },
    utils::SVec,
};
/*
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
}
impl From<u8> for PathIndex {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Head,
            1 => Self::Record,
            2 => Self::RecordType,
            3 => Self::LabelType,
            4 => Self::LabelDefiniens,
            5 => Self::VarType,
            6 => Self::VarDefiniens,
            i => Self::Argument(i - 7),
        }
    }
}
impl From<PathIndex> for u8 {
    fn from(value: PathIndex) -> Self {
        match value {
            PathIndex::Head => 0,
            PathIndex::Record => 1,
            PathIndex::RecordType => 2,
            PathIndex::LabelType => 3,
            PathIndex::LabelDefiniens => 4,
            PathIndex::VarType => 5,
            PathIndex::VarDefiniens => 6,
            PathIndex::Argument(i) => i.saturating_add(7),
        }
    }
}
 */

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct TermPath(SVec<u8, 16>);
impl TermPath {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn inner(&self) -> &[u8] {
        self.0.0.as_slice()
    }
    #[must_use]
    pub const fn inner_mut(&mut self) -> &mut smallvec::SmallVec<u8, 16> {
        &mut self.0.0
    }
}
impl From<Vec<u8>> for TermPath {
    fn from(value: Vec<u8>) -> Self {
        Self(SVec(value.into()))
    }
}
impl From<TermPath> for Vec<u8> {
    fn from(value: TermPath) -> Self {
        value.0.0.into_vec()
    }
}
impl From<smallvec::SmallVec<u8, 16>> for TermPath {
    fn from(value: smallvec::SmallVec<u8, 16>) -> Self {
        Self(SVec(value))
    }
}
impl From<TermPath> for smallvec::SmallVec<u8, 16> {
    fn from(value: TermPath) -> Self {
        value.0.0
    }
}

impl Term {
    #[must_use]
    pub fn path_of_subterm(&self, sub: &Self) -> Option<TermPath> {
        let mut ret = smallvec::SmallVec::<u8, 16>::new();
        if self.path_of_subterm_i(sub, &mut ret) {
            Some(TermPath(SVec(ret)))
        } else {
            None
        }
    }

    #[must_use]
    pub fn path_of_subterm_with_ctx(&self, sub: &Self) -> Option<(Vec<&ComponentVar>, TermPath)> {
        let mut ret = smallvec::SmallVec::<u8, 16>::new();
        let mut vars = Vec::new();
        if self.path_of_subterm_with_ctx_i(sub, &mut ret, &mut vars) {
            Some((vars, TermPath(SVec(ret))))
        } else {
            None
        }
    }

    #[must_use]
    pub fn subterm_at_path(&self, path: &TermPath) -> Option<(Vec<&ComponentVar>, &Self)> {
        let mut vars = Vec::new();
        self.subterm_at_path_i(path.0.as_slice(), &mut vars)
            .map(|t| (vars, t))
    }

    fn subterm_at_path_i<'s>(
        &'s self,
        path: &[u8],
        vars: &mut Vec<&'s ComponentVar>,
    ) -> Option<&'s Self> {
        if path.is_empty() {
            return Some(self);
        }
        let first = path[0] as usize;
        let path = &path[1..];
        match self {
            Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => None,
            Self::Application(app) => app.subterm_at_path_i(first, path, vars),
            Self::Bound(b) => b.subterm_at_path_i(first, path, vars),
            Self::Field(f) => {
                if first == 0 {
                    return f.record.subterm_at_path_i(path, vars);
                }
                if first == 1
                    && let Some(tp) = f.record_type.as_ref()
                {
                    tp.subterm_at_path_i(path, vars)
                } else {
                    None
                }
            }
            Self::Label { df, tp, .. } => {
                if first == 0
                    && let Some(t) = tp.as_ref()
                {
                    return t.subterm_at_path_i(path, vars);
                }
                if first == 0
                    && tp.is_none()
                    && let Some(t) = df.as_ref()
                {
                    return t.subterm_at_path_i(path, vars);
                }
                if first == 1
                    && tp.is_some()
                    && let Some(t) = df.as_ref()
                {
                    t.subterm_at_path_i(path, vars)
                } else {
                    None
                }
            }
            Self::Opaque(o) => o.terms.get(first)?.subterm_at_path_i(path, vars),
        }
    }

    fn path_of_subterm_with_ctx_i<'s>(
        &'s self,
        sub: &Self,
        ret: &mut smallvec::SmallVec<u8, 16>,
        vars: &mut Vec<&'s ComponentVar>,
    ) -> bool {
        if self.similar(sub) {
            return true;
        }
        match self {
            Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => false,
            Self::Application(app) => app.path_of_subterm_with_ctx_i(sub, ret, vars),
            Self::Bound(b) => {
                let (b, a) = b.path_of_subterm_with_ctx_i(sub, ret, vars);
                if !b {
                    for i in 0..a {
                        vars.pop();
                    }
                }
                b
            }
            Self::Field(f) => {
                ret.push(0);
                if f.record.path_of_subterm_with_ctx_i(sub, ret, vars) {
                    return true;
                }
                ret.pop();
                if let Some(tp) = f.record_type.as_ref() {
                    ret.push(1);
                    if tp.path_of_subterm_with_ctx_i(sub, ret, vars) {
                        true
                    } else {
                        ret.pop();
                        false
                    }
                } else {
                    false
                }
            }
            Self::Label { df, tp, .. } => {
                let mut next = 0;
                if let Some(t) = tp.as_ref() {
                    ret.push(0);
                    if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                        return true;
                    }
                    ret.pop();
                    next = 1;
                }
                if let Some(t) = df.as_ref() {
                    ret.push(next);
                    if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                        return true;
                    }
                    ret.pop();
                }
                false
            }
            Self::Opaque(o) => {
                for (i, t) in o.terms.iter().enumerate() {
                    ret.push(i as u8);
                    if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                        return true;
                    }
                    ret.pop();
                }
                false
            }
        }
    }

    fn path_of_subterm_i(&self, sub: &Self, ret: &mut smallvec::SmallVec<u8, 16>) -> bool {
        if self.similar(sub) {
            return true;
        }
        match self {
            Self::Symbol { .. } | Self::Var { .. } | Self::Number(_) => false,
            Self::Application(app) => app.path_of_subterm_i(sub, ret),
            Self::Bound(b) => b.path_of_subterm_i(sub, ret),
            Self::Field(f) => {
                ret.push(0);
                if f.record.path_of_subterm_i(sub, ret) {
                    return true;
                }
                ret.pop();
                if let Some(tp) = f.record_type.as_ref() {
                    ret.push(1);
                    if tp.path_of_subterm_i(sub, ret) {
                        true
                    } else {
                        ret.pop();
                        false
                    }
                } else {
                    false
                }
            }
            Self::Label { df, tp, .. } => {
                let mut next = 0;
                if let Some(t) = tp.as_ref() {
                    ret.push(0);
                    if t.path_of_subterm_i(sub, ret) {
                        return true;
                    }
                    ret.pop();
                    next = 1;
                }
                if let Some(t) = df.as_ref() {
                    ret.push(next);
                    if t.path_of_subterm_i(sub, ret) {
                        return true;
                    }
                    ret.pop();
                }
                false
            }
            Self::Opaque(o) => {
                for (i, t) in o.terms.iter().enumerate() {
                    ret.push(i as u8);
                    if t.path_of_subterm_i(sub, ret) {
                        return true;
                    }
                    ret.pop();
                }
                false
            }
        }
    }
}

impl BindingTerm {
    fn subterm_at_path_i<'s>(
        &'s self,
        index: usize,
        path: &[u8],
        vars: &mut Vec<&'s ComponentVar>,
    ) -> Option<&'s Term> {
        if index == 0 {
            return self.head.subterm_at_path_i(path, vars);
        }
        for a in self.arguments.get(..index - 1).unwrap_or_default() {
            match a {
                BoundArgument::Bound(v) | BoundArgument::BoundSeq(MaybeSequence::One(v)) => {
                    vars.push(v);
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => vars.extend(vs.iter()),
                _ => (),
            }
        }
        match self.arguments.get(index - 1)? {
            BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                t.subterm_at_path_i(path, vars)
            }
            BoundArgument::Sequence(MaybeSequence::Seq(ts)) if path.is_empty() => None,
            BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                let index = path[0];
                let path = &path[1..];
                ts.get(index as usize)?.subterm_at_path_i(path, vars)
            }
            BoundArgument::Bound(ComponentVar { tp, df, .. })
            | BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar { tp, df, .. })) => {
                if path.is_empty() {
                    return None;
                }
                let index = path[0];
                let path = &path[1..];
                if index == 0
                    && let Some(t) = tp.as_ref()
                {
                    return t.subterm_at_path_i(path, vars);
                }
                if index == 0
                    && tp.is_none()
                    && let Some(t) = df.as_ref()
                {
                    return t.subterm_at_path_i(path, vars);
                }
                if index == 1
                    && tp.is_some()
                    && let Some(t) = df.as_ref()
                {
                    t.subterm_at_path_i(path, vars)
                } else {
                    None
                }
            }
            BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                if path.is_empty() {
                    return None;
                }
                let index = path[0];
                let path = &path[1..];
                let mut iter = vs
                    .iter()
                    .flat_map(|ComponentVar { tp, df, .. }| [tp, df])
                    .flatten();
                iter.nth(index as usize)?.subterm_at_path_i(path, vars)
            }
        }
    }

    fn path_of_subterm_with_ctx_i<'s>(
        &'s self,
        sub: &Term,
        ret: &mut smallvec::SmallVec<u8, 16>,
        vars: &mut Vec<&'s ComponentVar>,
    ) -> (bool, usize) {
        ret.push(0);
        if self.head.path_of_subterm_with_ctx_i(sub, ret, vars) {
            return (true, 0);
        }
        ret.pop();
        let mut added_vars = 0;
        for (i, a) in self.arguments.iter().enumerate() {
            ret.push((i + 1) as u8);
            match a {
                BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                    if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                        return (true, added_vars);
                    }
                }
                BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                    for (j, t) in ts.iter().enumerate() {
                        ret.push(j as u8);
                        if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                            return (true, added_vars);
                        }
                        ret.pop();
                    }
                }
                BoundArgument::Bound(cv @ ComponentVar { tp, df, .. })
                | BoundArgument::BoundSeq(MaybeSequence::One(cv @ ComponentVar { tp, df, .. })) => {
                    let mut next = 0;
                    if let Some(t) = tp.as_ref() {
                        ret.push(0);
                        if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                            return (true, added_vars);
                        }
                        ret.pop();
                        next = 1;
                    }
                    if let Some(t) = df.as_ref() {
                        ret.push(next);
                        if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                            return (true, added_vars);
                        }
                        ret.pop();
                    }
                    vars.push(cv);
                    added_vars += 1;
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                    let mut i = 0;
                    for cv in vs {
                        if let Some(t) = cv.tp.as_ref() {
                            ret.push(i);
                            if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                                return (true, added_vars);
                            }
                            ret.pop();
                            i += 1;
                        }
                        if let Some(t) = cv.df.as_ref() {
                            ret.push(i);
                            if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                                return (true, added_vars);
                            }
                            ret.pop();
                            i += 1;
                        }
                        vars.push(cv);
                        added_vars += 1;
                    }
                }
            }
            ret.pop();
        }
        (false, added_vars)
    }

    fn path_of_subterm_i(&self, sub: &Term, ret: &mut smallvec::SmallVec<u8, 16>) -> bool {
        ret.push(0);
        if self.head.path_of_subterm_i(sub, ret) {
            return true;
        }
        ret.pop();
        for (i, a) in self.arguments.iter().enumerate() {
            ret.push((i + 1) as u8);
            match a {
                BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                    if t.path_of_subterm_i(sub, ret) {
                        return true;
                    }
                }
                BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                    for (j, t) in ts.iter().enumerate() {
                        ret.push(j as u8);
                        if t.path_of_subterm_i(sub, ret) {
                            return true;
                        }
                        ret.pop();
                    }
                }
                BoundArgument::Bound(ComponentVar { tp, df, .. })
                | BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar { tp, df, .. })) => {
                    let mut next = 0;
                    if let Some(t) = tp.as_ref() {
                        ret.push(0);
                        if t.path_of_subterm_i(sub, ret) {
                            return true;
                        }
                        ret.pop();
                        next = 1;
                    }
                    if let Some(t) = df.as_ref() {
                        ret.push(next);
                        if t.path_of_subterm_i(sub, ret) {
                            return true;
                        }
                        ret.pop();
                    }
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                    let iter = vs
                        .iter()
                        .flat_map(|ComponentVar { tp, df, .. }| [tp, df])
                        .flatten();
                    for (i, t) in iter.enumerate() {
                        ret.push(i as u8);
                        if t.path_of_subterm_i(sub, ret) {
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
}

impl ApplicationTerm {
    fn subterm_at_path_i<'s>(
        &'s self,
        index: usize,
        path: &[u8],
        vars: &mut Vec<&'s ComponentVar>,
    ) -> Option<&'s Term> {
        if index == 0 {
            return self.head.subterm_at_path_i(path, vars);
        }
        match self.arguments.get(index - 1)? {
            Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                t.subterm_at_path_i(path, vars)
            }
            Argument::Sequence(MaybeSequence::Seq(ts)) if path.is_empty() => None,
            Argument::Sequence(MaybeSequence::Seq(ts)) => {
                let index = path[0];
                let path = &path[1..];
                ts.get(index as usize)?.subterm_at_path_i(path, vars)
            }
        }
    }
    fn path_of_subterm_with_ctx_i<'s>(
        &'s self,
        sub: &Term,
        ret: &mut smallvec::SmallVec<u8, 16>,
        vars: &mut Vec<&'s ComponentVar>,
    ) -> bool {
        ret.push(0);
        if self.head.path_of_subterm_with_ctx_i(sub, ret, vars) {
            return true;
        }
        ret.pop();
        for (i, a) in self.arguments.iter().enumerate() {
            ret.push((i + 1) as u8);
            match a {
                Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                    if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
                        return true;
                    }
                }
                Argument::Sequence(MaybeSequence::Seq(ts)) => {
                    for (j, t) in ts.iter().enumerate() {
                        ret.push(j as u8);
                        if t.path_of_subterm_with_ctx_i(sub, ret, vars) {
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

    fn path_of_subterm_i(&self, sub: &Term, ret: &mut smallvec::SmallVec<u8, 16>) -> bool {
        ret.push(0);
        if self.head.path_of_subterm_i(sub, ret) {
            return true;
        }
        ret.pop();
        for (i, a) in self.arguments.iter().enumerate() {
            ret.push((i + 1) as u8);
            match a {
                Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                    if t.path_of_subterm_i(sub, ret) {
                        return true;
                    }
                }
                Argument::Sequence(MaybeSequence::Seq(ts)) => {
                    for (j, t) in ts.iter().enumerate() {
                        ret.push(j as u8);
                        if t.path_of_subterm_i(sub, ret) {
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
}
