pub mod block;
pub mod collapsible;
pub mod theming;

use crate::components::terms::OnClickData;
use ftml_dom::{DocumentState, notations::TermExt, utils::local_cache::LocalCache};
use ftml_ontology::terms::{Term, VarOrSym};
use ftml_uris::{DocumentElementUri, LeafUri};
use leptos::{
    IntoView,
    prelude::{AnyView, IntoAny, Owner, RwSignal, expect_context},
    tachys::reactive_graph::OwnedView,
};

#[leptos::prelude::slot]
pub struct Header {
    children: leptos::prelude::Children,
}

pub fn error_toast(msg: impl IntoView + std::fmt::Display + 'static) {
    use leptos::view;
    use thaw::{
        MessageBar, MessageBarBody, MessageBarIntent, ToastOptions, ToastPosition, ToasterInjection,
    };
    tracing::error!("{msg}");
    let toaster = ToasterInjection::expect_context();
    toaster.dispatch_toast(
        move || {
            view! {
              <MessageBar intent=MessageBarIntent::Error>
                <MessageBarBody>{msg}</MessageBarBody>
              </MessageBar>
            }
        },
        ToastOptions::default().with_position(ToastPosition::Top),
    );
}

type Map<A, B> = rustc_hash::FxHashMap<A, B>;

#[derive(Clone)]
pub struct ReactiveStore(std::sync::Arc<std::sync::Mutex<ReactiveStoreI>>);
pub(crate) struct ReactiveStoreI {
    pub(crate) notations: Map<LeafUri, RwSignal<Option<DocumentElementUri>>>,
    pub(crate) on_clicks: Map<VarOrSym, OnClickData>,
    owner: Owner,
    term_owner: Owner,
}
impl ReactiveStore {
    pub(crate) fn with_value<R>(&self, f: impl FnOnce(&ReactiveStoreI) -> R) -> R {
        let lock = self.0.lock().expect("error locking");
        f(&lock)
    }
    pub(crate) fn update_value<R>(&self, f: impl FnOnce(&mut ReactiveStoreI) -> R) -> R {
        let mut lock = self.0.lock().expect("error locking");
        f(&mut lock)
    }
    #[inline]
    pub(crate) fn new() -> Self {
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        owner.with(|| DocumentState::no_document(|| {}));
        let term_owner = owner.child();
        Self(std::sync::Arc::new(std::sync::Mutex::new(ReactiveStoreI {
            notations: Map::default(),
            on_clicks: Map::default(),
            owner,
            term_owner,
        })))
    }
    #[inline]
    /// ### Panics
    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        let lock = self.0.lock().expect("error locking");
        let owner = lock.owner.clone();
        drop(lock);
        owner.with(f)
    }

    #[must_use]
    /// ### Panics
    pub fn render_term(t: Term) -> impl IntoView {
        let slf = Self::get();
        let lock = slf.0.lock().expect("error locking");
        let owner = lock.term_owner.clone();
        drop(lock);
        owner.with(move || t.into_view_safe::<crate::Views>(crate::backend()))
    }

    #[inline]
    #[must_use]
    pub(crate) fn get() -> Self {
        expect_context()
    }
    /// #### Panics
    pub fn on_click(vos: &VarOrSym) -> OnClickData {
        use leptos::prelude::*;
        use thaw::{Dialog, DialogSurface};
        let slf = Self::get();
        let (owner, (data, on_clicked, uri, allow_formals)) = {
            let mut slf = slf.0.lock().expect("error locking");
            if let Some(d) = slf.on_clicks.get(vos) {
                return *d;
            }
            let owner = slf.owner.clone();
            let r = owner.with(OnClickData::new);
            slf.on_clicks.insert(vos.clone(), r.0);
            drop(slf);
            (owner, r)
        };
        let vos = vos.clone();
        let child = owner.child();
        owner.with(move || {
            let _ = {
                view! {<Dialog open=on_clicked><DialogSurface>{
                    let r = child.with(|| {
                        provide_context(slf);
                        crate::components::terms::popover::do_onclick(vos,uri,allow_formals)
                    });
                    OwnedView::new_with_owner(r.into_view(), child)
                }</DialogSurface></Dialog>}
            };
            data
        })
    }
}

pub trait LocalCacheExt: 'static {
    fn with<R, E, Fut>(
        f: impl FnOnce(&'static Self) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
    ) -> AnyView
    where
        R: Send + Sync + 'static + Clone,
        E: std::fmt::Debug + Send + Sync + 'static,
        Fut: Future<Output = Result<R, ftml_backend::BackendError<E>>> + Send + 'static;

    fn with_or_err<R, E, Fut>(
        f: impl FnOnce(&'static Self) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
        error: impl FnOnce(ftml_backend::BackendError<E>) -> AnyView + Clone + Send + 'static,
    ) -> AnyView
    where
        R: Send + Sync + 'static + Clone,
        E: std::fmt::Debug + Send + Sync + 'static,
        Fut: Future<Output = Result<R, ftml_backend::BackendError<E>>> + 'static + Send;

    fn with_or_toast<R, E, Fut>(
        f: impl FnOnce(&'static Self) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
        error: impl FnOnce() -> AnyView + Send + Clone + 'static,
    ) -> AnyView
    where
        R: Send + Sync + 'static + Clone,
        E: std::fmt::Debug + Send + Sync + 'static,
        Fut: Future<Output = Result<R, ftml_backend::BackendError<E>>> + Send + 'static;
}

impl LocalCacheExt for LocalCache {
    fn with<R, E, Fut>(
        f: impl FnOnce(&'static Self) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
    ) -> AnyView
    where
        R: Send + Sync + 'static + Clone,
        E: std::fmt::Debug + Send + Sync + 'static,
        Fut: Future<Output = Result<R, ftml_backend::BackendError<E>>> + Send + 'static,
    {
        Self::with_or_err(f, view, |e| {
            tracing::error!("{e:?}");
            ().into_any()
        })
    }

    fn with_or_toast<R, E, Fut>(
        f: impl FnOnce(&'static Self) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
        error: impl FnOnce() -> AnyView + Send + Clone + 'static,
    ) -> AnyView
    where
        R: Send + Sync + 'static + Clone,
        E: std::fmt::Debug + Send + Sync + 'static,
        Fut: Future<Output = Result<R, ftml_backend::BackendError<E>>> + Send + 'static,
    {
        use leptos::prelude::*;
        use thaw::{
            MessageBar, MessageBarBody, MessageBarIntent, ToastOptions, ToastPosition,
            ToasterInjection,
        };
        let toaster = ToasterInjection::expect_context();
        Self::with_or_err(f, view, move |e| {
            tracing::error!("{e:?}");
            toaster.dispatch_toast(
                move || {
                    view! {
                      <MessageBar intent=MessageBarIntent::Error>
                        <MessageBarBody>{format!("{e:?}")}</MessageBarBody>
                      </MessageBar>
                    }
                },
                ToastOptions::default().with_position(ToastPosition::Top),
            );
            error()
        })
    }

    fn with_or_err<R, E, Fut>(
        f: impl FnOnce(&'static Self) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
        error: impl FnOnce(ftml_backend::BackendError<E>) -> AnyView + Clone + Send + 'static,
    ) -> AnyView
    where
        R: Send + Sync + 'static + Clone,
        E: std::fmt::Debug + Send + Sync + 'static,
        Fut: Future<Output = Result<R, ftml_backend::BackendError<E>>> + 'static + Send,
    {
        wait_and_then(move || f(Self::get()), view, error)
    }
}

pub fn wait_and_then<R, E: Send + Sync + 'static, Fut>(
    f: impl FnOnce() -> Fut + Send + Sync + 'static + Clone,
    view: impl FnOnce(R) -> AnyView + Clone + Send + 'static,
    error: impl FnOnce(E) -> AnyView + Clone + Send + 'static,
) -> AnyView
where
    R: Send + Sync + 'static + Clone,
    Fut: Future<Output = Result<R, E>> + Send + 'static,
{
    use leptos::{
        either::Either::{Left, Right},
        prelude::*,
    };
    use thaw::Spinner;
    view! {
        <Suspense fallback = || view!(<Spinner/>)>{move || {
            let v = view.clone();
            let err = error.clone();
            let fut = (f.clone())();
            Suspend::new(async move {
                match fut.await {
                    Ok(r) => Left(v(r)),
                    Err(e) => Right(err(e))
                }
            })
        }}</Suspense>
    }
    .into_any()
}
