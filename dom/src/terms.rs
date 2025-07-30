use crate::{ClonableView, FtmlViews, extractor::DomExtractor};
use ftml_core::extraction::{ArgumentPosition, FtmlExtractor, VarOrSym};
use ftml_ontology::terms::{ArgumentMode, Term};
use leptos::either::Either::{self, Left, Right};
use leptos::prelude::*;

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

#[warn(clippy::type_complexity)]
pub struct OpenApp {
    pub head: VarOrSym,
    pub(crate) arguments: Vec<Option<Either<ClonableView, Vec<Option<ClonableView>>>>>,
}

pub struct ClosedApp {
    pub head: VarOrSym,
    pub term: Term,
    pub arguments: Vec<Either<ClonableView, Vec<ClonableView>>>,
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
                    let arguments = std::mem::take(arguments);
                    tracing::trace!("Closing {head:?} as {:?}", t.debug_short());
                    *app = Self::Closed(ClosedApp {
                        head,
                        term: t,
                        arguments: arguments
                            .into_iter()
                            .filter_map(|e| {
                                e.map(|o| o.map_right(|r| r.into_iter().flatten().collect()))
                            })
                            .collect(),
                    });
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

    pub(crate) fn add_argument<Views: FtmlViews + ?Sized>(
        slf: RwSignal<Self>,
        position: ArgumentPosition,
        children: ClonableView,
    ) -> impl IntoView {
        /*
        let t = with_context::<RwSignal<DomExtractor>, _>(|s| {
            s.with_untracked(|s| s.term_at(position).cloned())
        })
        .flatten();
        if let Some(t) = t {
            let ch = children.clone();
            slf.update_untracked(move |app| app.set(position, t, ch));
        }
         */
        let ch = children.clone();
        slf.update_untracked(move |app| app.set(position, ch));
        children.into_view::<Views>()
    }

    pub(crate) fn set(&mut self, position: ArgumentPosition, vw: ClonableView) {
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
                ) => *r = Some(Left(vw)),
                (r @ None, ArgumentPosition::Sequence { sequence_index, .. }) => {
                    let mut v: Vec<Option<ClonableView>> = (0..(sequence_index.get() - 1) as usize)
                        .map(|_| None)
                        .collect();
                    v.push(Some(vw));
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
                    v[idx] = Some(vw);
                }
                _ => (),
            }
        }
    }
}
