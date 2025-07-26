pub mod theming;

use ftml_backend::FtmlBackend;
use ftml_dom::utils::local_cache::{LocalCache, SendBackend, WithLocalCache};
use leptos::IntoView;

pub trait LocalCacheExt {
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
            tracing::error!("{:?}", e);
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
            tracing::error!("{:?}", e);
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
