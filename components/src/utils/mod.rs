pub mod block;
pub mod collapsible;
pub mod theming;

use crate::components::terms::OnClickData;
use ftml_backend::FtmlBackend;
use ftml_dom::{
    DocumentState,
    notations::TermExt,
    utils::local_cache::{LocalCache, SendBackend, WithLocalCache},
};
use ftml_ontology::terms::{Term, VarOrSym};
use ftml_uris::{DocumentElementUri, LeafUri};
use leptos::{
    IntoView,
    prelude::{Owner, RwSignal, StoredValue, WithValue, expect_context},
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
pub struct ReactiveStore {
    pub(crate) notations: Map<LeafUri, RwSignal<Option<DocumentElementUri>>>,
    pub(crate) on_clicks: Map<VarOrSym, OnClickData>,
    owner: Owner,
    term_owner: Owner,
}
impl ReactiveStore {
    #[inline]
    pub(crate) fn new() -> Self {
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        owner.with(|| DocumentState::no_document(|| {}));
        let term_owner = owner.child();
        Self {
            notations: Map::default(),
            on_clicks: Map::default(),
            owner,
            term_owner,
        }
    }
    #[inline]
    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        self.owner.with(f)
    }

    #[must_use]
    pub fn render_term<Be: SendBackend>(t: Term) -> impl IntoView {
        Self::get().with_value(|slf| {
            slf.term_owner
                .with(move || t.into_view_safe::<crate::Views<Be>, Be>())
        })
    }

    #[inline]
    #[must_use]
    pub(crate) fn get() -> StoredValue<Self> {
        expect_context()
    }
    /// #### Panics
    pub fn on_click<Be: SendBackend>(vos: &VarOrSym) -> OnClickData {
        use leptos::prelude::*;
        use thaw::{Dialog, DialogSurface};
        let slf = Self::get();
        let (owner, (data, on_clicked, uri, allow_formals)) = {
            let mut slf = slf.write_value();
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
                        crate::components::terms::do_onclick::<Be>(&vos,uri,allow_formals)
                    });
                    OwnedView::new_with_owner(r.into_view(), child)
                }</DialogSurface></Dialog>}
            };
            data
        })
    }
}

pub trait LocalCacheExt {
    fn with<B: SendBackend, R, Fut, V: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static;

    fn with_or_err<B: SendBackend, R, Fut, V: IntoView + 'static, V2: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
        error: impl FnOnce(ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>) -> V2
        + Clone
        + Send
        + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static;

    fn with_or_toast<B: SendBackend, R, Fut, V: IntoView + 'static, V2: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
        error: impl FnOnce() -> V2 + Send + Clone + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static;
}

impl LocalCacheExt for LocalCache {
    fn with<B: SendBackend, R, Fut, V: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static,
    {
        Self::with_or_err::<B, _, _, _, _>(f, view, |e| {
            tracing::error!("{e}");
        })
    }

    fn with_or_toast<B: SendBackend, R, Fut, V: IntoView + 'static, V2: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
        error: impl FnOnce() -> V2 + Send + Clone + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static,
    {
        use leptos::prelude::*;
        use thaw::{
            MessageBar, MessageBarBody, MessageBarIntent, ToastOptions, ToastPosition,
            ToasterInjection,
        };
        let toaster = ToasterInjection::expect_context();
        Self::with_or_err::<B, _, _, _, _>(f, view, move |e| {
            tracing::error!("{e}");
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

    fn with_or_err<B: SendBackend, R, Fut, V: IntoView + 'static, V2: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
        error: impl FnOnce(ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>) -> V2
        + Clone
        + Send
        + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            >
            + 'static
            + Send,
    {
        wait_and_then(move || f(WithLocalCache::default()), view, error)
    }
}

pub fn wait_and_then<
    R,
    E: Send + Sync + 'static,
    Fut,
    V: IntoView + 'static,
    V2: IntoView + 'static,
>(
    f: impl FnOnce() -> Fut + Send + Sync + 'static + Clone,
    view: impl FnOnce(R) -> V + Clone + Send + 'static,
    error: impl FnOnce(E) -> V2 + Clone + Send + 'static,
) -> impl IntoView
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
}
