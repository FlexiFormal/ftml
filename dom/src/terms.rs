use crate::{ClonableView, FtmlViews};
use ftml_core::extraction::ArgumentPosition;
use ftml_ontology::terms::VarOrSym;
use ftml_uris::DocumentElementUri;
use leptos::either::Either::{self, Left, Right};
use leptos::prelude::*;

#[derive(Clone)]
pub(crate) struct ReactiveTerm {
    //pub uri: Option<DocumentElementUri>,
    pub app: RwSignal<ReactiveApplication>,
    //owner: Owner,
}

pub enum ReactiveApplication {
    Open(OpenApp),
    Closed(ClosedApp),
}

#[warn(clippy::type_complexity)]
pub struct OpenApp {
    pub head: VarOrSym,
    //owner: Owner,
    pub(crate) arguments: Vec<Option<Either<ClonableView, Vec<Option<ClonableView>>>>>,
}

pub struct ClosedApp {
    pub head: VarOrSym,
    //owner: Owner,
    pub arguments: Vec<Either<ClonableView, Vec<ClonableView>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TopTerm {
    pub uri: DocumentElementUri,
}

impl ReactiveTerm {
    pub(crate) fn add_argument<Views: FtmlViews + ?Sized>(
        &self,
        position: ArgumentPosition,
        children: ClonableView,
    ) -> impl IntoView {
        children.set_state();
        let ch = children.clone();
        self.app.try_update_untracked(move |app| {
            tracing::trace!("Adding argument to {} at position {position:?}", app.head());
            app.set(position, ch);
        });
        children.into_view::<Views>()
    }
}

impl ReactiveApplication {
    #[inline]
    pub const fn head(&self) -> &VarOrSym {
        match self {
            Self::Open(a) => &a.head,
            Self::Closed(a) => &a.head,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub const fn len(&self) -> usize {
        match self {
            Self::Open(a) => a.arguments.len(),
            Self::Closed(a) => a.arguments.len(),
        }
    }
    pub(crate) fn close() {
        tracing::trace!(
            "Closing",
            //expect_context::<crate::OwnerId>().0
        );
        let Some(sig) =
            with_context::<Option<ReactiveTerm>, _>(|s| s.as_ref().map(|s| s.app)).flatten()
        else {
            return;
        };
        sig.update(move |app| match app {
            Self::Open(OpenApp {
                head,
                //owner,
                arguments,
            }) => {
                let head = head.clone();
                let arguments = std::mem::take(arguments);
                tracing::trace!("Closing {head:?} as {:?}", arguments);
                *app = Self::Closed(ClosedApp {
                    head,
                    //owner: owner.clone(),
                    arguments: arguments
                        .into_iter()
                        .filter_map(|e| {
                            e.map(|o| o.map_right(|r| r.into_iter().flatten().collect()))
                        })
                        .collect(),
                });
            }
            Self::Closed(_) => {
                tracing::debug!("Tracked term is already closed");
            }
        });
        /*} else {
            tracing::warn!("Tracked term does not exist");
        }*/
    }
    pub(crate) fn track<V: IntoView>(
        head: VarOrSym,
        uri: Option<DocumentElementUri>,
        children: impl FnOnce(ReadSignal<Self>) -> V,
    ) -> impl IntoView {
        tracing::debug!("Tracking {head:?}");
        let sig = RwSignal::new(Self::Open(OpenApp {
            //owner: Owner::current().expect("not in a reactive context"),
            head,
            arguments: Vec::new(),
        }));
        if let Some(uri) = uri {
            provide_context(Some(TopTerm { uri }));
        }
        provide_context(Some(ReactiveTerm {
            app: sig,
            //owner: Owner::current().expect("Not in a reactive context"),
        }));
        children(sig.read_only())
    }

    pub(crate) fn set(&mut self, position: ArgumentPosition, vw: ClonableView) {
        if let Self::Open(app) = self {
            let index = position.index() as usize;
            while app.arguments.len() <= index {
                app.arguments.push(None);
            }
            let arg = &mut app.arguments[index];
            match (arg, position) {
                (r @ None, ArgumentPosition::Simple(_, _)) => *r = Some(Left(vw)),
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
