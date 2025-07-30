pub mod theming;

use ftml_backend::FtmlBackend;
use ftml_core::extraction::VarOrSym;
use ftml_dom::utils::local_cache::{LocalCache, SendBackend, WithLocalCache};
use ftml_uris::{DocumentElementUri, LeafUri};
use leptos::{
    IntoView,
    prelude::{Owner, RwSignal, StoredValue, expect_context},
};

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
    pub(crate) on_clicks: Map<VarOrSym, RwSignal<bool>>,
    owner: Owner,
}
impl ReactiveStore {
    #[inline]
    pub(crate) fn new() -> Self {
        let owner = Owner::new();
        Self {
            notations: Map::default(),
            on_clicks: Map::default(),
            owner,
        }
    }
    #[inline]
    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        self.owner.with(f)
    }
    #[inline]
    #[must_use]
    pub fn get() -> StoredValue<Self> {
        expect_context()
    }
    pub fn on_click<Be: SendBackend>(&mut self, vos: &VarOrSym) -> RwSignal<bool> {
        use leptos::prelude::*;
        use thaw::{Dialog, DialogSurface};
        if let Some(s) = self.on_clicks.get(vos) {
            return *s;
        }
        let vos = vos.clone();
        self.owner.clone().with(move || {
            let signal = RwSignal::new(false);
            self.on_clicks.insert(vos.clone(), signal);
            let _ = view! {<Dialog open=signal><DialogSurface>{
                crate::components::terms::do_onclick::<Be>(&vos)
            }</DialogSurface></Dialog>};
            signal
        })
    }
}

pub trait LocalCacheExt {
    fn resource<B: SendBackend, R, Fut>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
    ) -> leptos::prelude::Resource<
        Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
    >
    where
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static;
    fn with<B: SendBackend, R, Fut, V: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
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
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
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
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static;
}

impl LocalCacheExt for LocalCache {
    fn resource<B: SendBackend, R, Fut>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
    ) -> leptos::prelude::Resource<
        Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
    >
    where
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static,
    {
        use leptos::prelude::*;
        Resource::new(|| (), move |()| (f.clone())(WithLocalCache::default()))
    }
    fn with<B: SendBackend, R, Fut, V: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static,
    {
        Self::with_or_err::<B, _, _, _, _>(f, view, |e| {
            tracing::error!("{}", e);
        })
    }

    fn with_or_toast<B: SendBackend, R, Fut, V: IntoView + 'static, V2: IntoView + 'static>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
        view: impl FnOnce(R) -> V + Clone + Send + 'static,
        error: impl FnOnce() -> V2 + Send + Clone + 'static,
    ) -> impl IntoView
    where
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
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
        R: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + Send
            + 'static,
    {
        use leptos::{
            either::Either::{Left, Right},
            prelude::*,
        };
        use thaw::Spinner;
        let r = Resource::new(|| (), move |()| (f.clone())(WithLocalCache::default()));
        view! {
            <Suspense fallback = || view!(<Spinner/>)>{move ||
                r.get().map(|r| match r {
                    Ok(r) => Left((view.clone())(r)),
                    Err(e) => Right((error.clone())(e))
                })
            }</Suspense>
        }
    }
}
