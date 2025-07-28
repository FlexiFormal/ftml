use either::Either::{self, Left, Right};
use ftml_core::extraction::{ArgumentPosition, FtmlExtractor, VarOrSym};
use ftml_ontology::terms::{ArgumentMode, Term};
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::extractor::DomExtractor;

#[derive(Clone)]
pub(crate) enum ReactiveTerm {
    //Symbol(SymbolUri),
    //Variable(Variable),
    Application(RwSignal<ReactiveApplication>),
}

pub enum ReactiveApplication {
    Open(OpenApp),
    Closed(ClosedApp),
}

pub struct OpenApp {
    pub head: VarOrSym,
    pub(crate) arguments: Vec<Option<Either<Term, Vec<Option<Term>>>>>,
}

pub struct ClosedApp {
    pub head: VarOrSym,
    pub term: Term,
    //pub arguments: Vec<Either<TermFn, Vec<TermFn>>>,
}

impl ReactiveApplication {
    #[inline]
    pub const fn head(&self) -> &VarOrSym {
        match self {
            Self::Open(a) => &a.head,
            Self::Closed(a) => &a.head,
        }
    }
    pub(crate) fn close() {
        tracing::trace!(
            "Closing; current owner: {}",
            expect_context::<crate::OwnerId>().0
        );
        let Some(sig) = with_context::<ReactiveTerm, _>(|s| {
            if let ReactiveTerm::Application(a) = s {
                Some(*a)
            } else {
                None
            }
        })
        .flatten() else {
            return;
        };

        let t = with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|s| s.last_term().cloned())
        })
        .flatten();
        if let Some(t) = t {
            sig.update(move |app| match app {
                Self::Open(OpenApp { head, arguments }) => {
                    let head = head.clone();
                    tracing::trace!("Closing {head:?} as {:?}", t.debug_short());
                    *app = Self::Closed(ClosedApp { head, term: t });
                }
                Self::Closed(_) => {
                    tracing::warn!("Tracked term is already closed");
                }
            });
        } else {
            tracing::warn!("Tracked term does not exist");
        }
    }
    pub(crate) fn track<V: IntoView>(
        head: VarOrSym,
        children: impl FnOnce(ReadSignal<Self>) -> V,
    ) -> impl IntoView {
        tracing::debug!(
            "Tracking {head:?} current owner: {}",
            expect_context::<crate::OwnerId>().0
        );
        let sig = RwSignal::new(Self::Open(OpenApp {
            head,
            arguments: Vec::new(),
        }));
        provide_context(ReactiveTerm::Application(sig));
        children(sig.read_only())
    }

    pub(crate) fn add_argument<V: IntoView>(
        slf: RwSignal<Self>,
        position: ArgumentPosition,
        children: impl FnOnce() -> V,
    ) -> impl IntoView {
        let t = with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|s| s.term_at(position).cloned())
        })
        .flatten();
        if let Some(t) = t {
            slf.update_untracked(move |app| app.set(position, t));
        }
        children()
    }

    pub(crate) fn set(&mut self, position: ArgumentPosition, term: Term) {
        if let Self::Open(app) = self {
            let index = position.index() as usize;
            while app.arguments.len() <= index + 1 {
                app.arguments.push(None);
            }
            let arg = &mut app.arguments[index];
            match (arg, position) {
                (
                    r @ None,
                    ArgumentPosition::Simple(_, ArgumentMode::Simple | ArgumentMode::Sequence),
                ) => *r = Some(Left(term)),
                (r @ None, ArgumentPosition::Sequence { sequence_index, .. }) => {
                    let mut v = (0..(sequence_index.get() - 1) as usize)
                        .map(|_| None)
                        .collect::<Vec<_>>();
                    v.push(Some(term));
                    *r = Some(Right(v));
                }
                (Some(Right(v)), ArgumentPosition::Sequence { sequence_index, .. }) => {
                    let idx = (sequence_index.get() - 1) as usize;
                    while v.len() <= idx + 1 {
                        v.push(None);
                    }
                    if v[idx].is_some() {
                        return;
                    }
                    v[idx] = Some(term);
                }
                _ => (),
            }
        }
    }
}

pub(crate) struct TermFn(SendWrapper<Box<dyn FnOnce() -> AnyView>>);
impl std::fmt::Debug for TermFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[TermFn]")
    }
}

/*
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Argument(WriteSignal<Option<Either<TermFn, Vec<Option<TermFn>>>>>);


impl ReactiveTerm {
    pub fn with_new<V: IntoView>(
        self,
        f: impl FnOnce() -> V + Clone + 'static,
    ) -> impl IntoView {
        let tm = Self::Open {
            head,
            arguments: Vec::new(),
        };
        provide_context(tm);

        let arg = with_context::<Argument, _>(|arg| arg.0);
        if let Some(arg) = arg {
            if arg.update_untracked(|a| a.is_none()) {
                let f = f.clone();
                arg.set(Some(Left(TermFn(SendWrapper::new(Box::new(move || {
                    f().into_any()
                }))))));
            }
        }
        f()
    }
}
 */
