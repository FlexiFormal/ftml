use ftml_ontology::utils::Css;
use ftml_uris::{DocumentUri, FtmlUri, Uri};
use send_wrapper::SendWrapper;

use crate::BackendError;

pub struct RemoteBackend {
    pub fragment_url: String,
}

pub struct RemoteFlamsBackend {
    pub url: String,
}

impl RemoteBackend {
    async fn call<R: serde::de::DeserializeOwned>(url: String) -> Result<R, BackendError> {
        FutWrap(SendWrapper::new(async move {
            let req = reqwasm::http::Request::get(&url)
                .send()
                .await
                .map_err(|e| BackendError::Request(e.to_string()))?;
            req.json::<R>()
                .await
                .map_err(|e| BackendError::Request(e.to_string()))
        }))
        .await
    }
}
impl super::FtmlBackend for RemoteBackend {
    #[allow(clippy::similar_names)]
    fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), crate::BackendError>> {
        let url = context.map_or_else(
            || format!("{}?uri={}", self.fragment_url, uri.url_encoded()),
            |ctx| {
                format!(
                    "{}?uri={}&context={}",
                    self.fragment_url,
                    uri.url_encoded(),
                    ctx.url_encoded()
                )
            },
        );
        Self::call(url)
    }
}
impl super::FlamsBackend for RemoteFlamsBackend {
    ftml_uris::compfun! {!!
        #[allow(clippy::similar_names)]
        async fn get_fragment(&self,uri:Uri,context:Option<DocumentUri>) -> Result<(Uri, Vec<Css>,String), BackendError> {
            let url = context.map_or_else(|| format!(
                "{}/content/fragment{}",
                self.url,
                uri.as_query(),
            ), |ctx| format!(
                "{}/content/fragment{}&context={}",
                self.url,
                uri.as_query(),
                ctx.url_encoded()
            ));
            RemoteBackend::call(url).await
        }
    }
    /*
    fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: ftml_uris::DocumentUri,
    ) -> impl Future<Output = Result<(String, Vec<Css>), crate::BackendError>> {
        RemoteBackend::call(format!(
            "{}/content/fragment?uri={}&context={}",
            self.url,
            urlencoding::encode(&uri.to_string()),
            urlencoding::encode(&context.to_string())
        ))
        // urlencoding::encode(uri)
    } */
}

struct FutWrap<F: Future>(send_wrapper::SendWrapper<F>);
impl<F: Future> std::ops::Deref for FutWrap<F> {
    type Target = F;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<F: Future> std::ops::DerefMut for FutWrap<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<F: Future> Future for FutWrap<F> {
    type Output = F::Output;
    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let inner: std::pin::Pin<&mut F> = unsafe { self.map_unchecked_mut(|s| &mut *s.0) };
        inner.poll(cx)
    }
}
