use ftml_backend::FtmlBackend;
use ftml_uris::DocumentUri;

pub(crate) static BACKEND_URL: std::sync::LazyLock<std::sync::Arc<parking_lot::RwLock<Box<str>>>> =
    std::sync::LazyLock::new(|| {
        std::sync::Arc::new(parking_lot::RwLock::new(
            ftml_backend::DEFAULT_SERVER_URL
                .to_string()
                .into_boxed_str(),
        ))
    });
pub struct BackendUrlRef;
impl BackendUrlRef {
    pub fn set_url(url: &str) {
        *BACKEND_URL.write() = url.to_string().into_boxed_str();
    }
}
impl std::fmt::Display for BackendUrlRef {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        BACKEND_URL.read().fmt(f)
    }
}

#[allow(clippy::type_complexity)]
pub(crate) static REDIRECTS: std::sync::LazyLock<
    std::sync::Arc<parking_lot::RwLock<Vec<(DocumentUri, Box<str>)>>>,
> = std::sync::LazyLock::new(|| std::sync::Arc::new(parking_lot::RwLock::new(Vec::new())));

pub struct RedirectsRef;
impl ftml_backend::Redirects for RedirectsRef {
    fn for_fragment<'s>(&'s self, uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        REDIRECTS
            .read()
            .iter()
            .find_map(|(u, r)| if *u == *uri { Some(r.clone()) } else { None })
    }
}

type BE =
    ftml_backend::CachedBackend<ftml_backend::RemoteFlamsBackend<BackendUrlRef, RedirectsRef>>;

pub struct GlobalBackend;
impl ftml_backend::GlobalBackend for GlobalBackend {
    type Backend = BE;
    type Error = <BE as ftml_backend::FtmlBackend>::Error;
    fn get() -> &'static BE {
        static BACKEND: std::sync::LazyLock<BE> = std::sync::LazyLock::new(|| {
            ftml_backend::RemoteFlamsBackend::new_with_redirects(BackendUrlRef, RedirectsRef, true)
                .cached()
        });
        &BACKEND
    }
}
