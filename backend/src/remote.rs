use std::marker::PhantomData;
use std::str::FromStr;

use ftml_ontology::utils::Css;
use ftml_uris::{DocumentUri, FtmlUri, LeafUri, NarrativeUri, Uri};

use crate::BackendError;

pub trait Redirects {
    #[inline]
    fn for_fragment<'s>(&'s self, _uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
    #[inline]
    fn for_notations<'s>(&'s self, _uri: &LeafUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
}
pub struct NoRedirects;
impl Redirects for NoRedirects {}
impl<const LEN: usize, D: std::fmt::Display> Redirects for [(DocumentUri, D); LEN] {
    fn for_fragment<'s>(&'s self, uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        self.iter()
            .find_map(|(u, d)| if *u == *uri { Some(d) } else { None })
    }
}

pub struct RemoteBackend<
    E: std::fmt::Debug,
    Url: std::fmt::Display = &'static str,
    Re: Redirects = NoRedirects,
> {
    pub fragment_url: Url,
    pub notations_url: Url,
    pub redirects: Re,
    __phantom: PhantomData<E>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("error during request: {0}")]
    Request(String),
    #[error("error deserializing response: {0}")]
    Deserialization(String),
}

impl<Url, E, Re: Redirects> RemoteBackend<E, Url, Re>
where
    Url: std::fmt::Display,
    E: std::fmt::Debug + From<RequestError>,
{
    pub const fn new_with_redirects(fragment_url: Url, notations_url: Url, redirects: Re) -> Self {
        Self {
            fragment_url,
            notations_url,
            redirects,
            __phantom: PhantomData,
        }
    }
}

impl<Url, E> RemoteBackend<E, Url>
where
    Url: std::fmt::Display,
    E: std::fmt::Debug + From<RequestError>,
{
    pub const fn new(fragment_url: Url, notations_url: Url) -> Self {
        Self {
            fragment_url,
            notations_url,
            redirects: NoRedirects,
            __phantom: PhantomData,
        }
    }
}

impl<Url, E, Re: Redirects> RemoteBackend<E, Url, Re>
where
    Url: std::fmt::Display,
    E: std::fmt::Debug + From<RequestError> + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    fn make_url<D: std::fmt::Display>(
        base: D,
        uri: &Uri,
        context: Option<&NarrativeUri>,
    ) -> String {
        context.map_or_else(
            || format!("{}?uri={}", base, uri.url_encoded()),
            |ctx| {
                format!(
                    "{}?uri={}&context={}",
                    base,
                    uri.url_encoded(),
                    ctx.url_encoded()
                )
            },
        )
    }
}

impl<Url, E, Re: Redirects> super::FtmlBackend for RemoteBackend<E, Url, Re>
where
    Url: std::fmt::Display,
    E: std::fmt::Debug + From<RequestError> + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    type Error = E;
    #[allow(clippy::similar_names)]
    fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<E>>> {
        let url = if let Uri::Document(d) = &uri {
            self.redirects.for_fragment(d).map_or_else(
                || Self::make_url(&self.fragment_url, &uri, context.as_ref()),
                |r| r.to_string(),
            )
        } else {
            Self::make_url(&self.fragment_url, &uri, context.as_ref())
        };
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_notations(
        &self,
        uri: ftml_uris::LeafUri,
    ) -> impl Future<
        Output = Result<
            Vec<(
                ftml_uris::DocumentElementUri,
                ftml_ontology::narrative::elements::Notation,
            )>,
            BackendError<Self::Error>,
        >,
    > + Send {
        let url = self.redirects.for_notations(&uri).map_or_else(
            || Self::make_url(&self.notations_url, &uri.into(), None),
            |r| r.to_string(),
        );
        call(url)
    }
}

#[cfg(feature = "server_fn")]
pub struct RemoteFlamsBackend<Url: std::fmt::Display, Re: Redirects = NoRedirects> {
    pub url: Url,
    pub redirects: Re,
}

#[cfg(feature = "server_fn")]
impl<Url, Re: Redirects> RemoteFlamsBackend<Url, Re>
where
    Url: std::fmt::Display,
{
    pub const fn new_with_redirects(url: Url, redirects: Re) -> Self {
        Self { url, redirects }
    }
}

#[cfg(feature = "server_fn")]
impl<Url: std::fmt::Display> RemoteFlamsBackend<Url> {
    pub const fn new(url: Url) -> Self {
        Self {
            url,
            redirects: NoRedirects,
        }
    }
}

#[cfg(feature = "server_fn")]
mod server_fn {
    use crate::{BackendError, FlamsBackend, Redirects, RemoteFlamsBackend};
    use ::server_fn::error::ServerFnErrorErr;
    use ftml_ontology::utils::Css;
    use ftml_uris::{FtmlUri, LeafUri, NarrativeUri, Uri, components::UriComponentTuple};
    use futures_util::TryFutureExt;

    impl<Url: std::fmt::Display, Re: Redirects> FlamsBackend for RemoteFlamsBackend<Url, Re> {
        ftml_uris::compfun! {!!
            #[allow(clippy::similar_names)]
            fn get_fragment(&self,uri:Uri,context:Option<NarrativeUri>) -> impl Future<Output=Result<(Uri, Vec<Css>,String), BackendError<ServerFnErrorErr>>> {
                fn make_url<D: std::fmt::Display>(
                    base: D,
                    uri: &UriComponentTuple,
                    context: Option<&NarrativeUri>,
                ) -> String {
                    context.map_or_else(
                        || format!("{}/content/fragment{}", base, uri.as_query()),
                        |ctx| {
                            format!(
                                "{}/content/fragment{}&context={}",
                                base,
                                uri.as_query(),
                                ctx.url_encoded()
                            )
                        },
                    )
                }

                let url = if let Some(Uri::Document(d)) = uri.uri.as_ref() {
                    self.redirects.for_fragment(d).map_or_else(
                        || make_url(&self.url, &uri, context.as_ref()),
                        |r| r.to_string(),
                    )
                } else {
                    make_url(&self.url, &uri, context.as_ref())
                };
                super::call::<_,SFnE>(url).map_err(BackendError::from_other)
            }
        }

        ftml_uris::compfun! {!!
            #[allow(clippy::similar_names)]
            fn get_notations(&self,uri:Uri) -> impl Future<Output=Result<Vec<(
                ftml_uris::DocumentElementUri,
                ftml_ontology::narrative::elements::Notation,
            )>, BackendError<ServerFnErrorErr>>> {
                fn leaf(base:impl std::fmt::Display,re:&impl Redirects,uri:&LeafUri) -> String {
                    re.for_notations(uri).map_or_else(
                        || format!(
                            "{}/content/notations?uri={}",
                            base,
                            uri.url_encoded(),
                        ),
                        |r| r.to_string()
                    )
                }
                let url = match uri.uri {
                    Some(Uri::Symbol(s)) => {
                        let uri = s.into();
                        leaf(&self.url,&self.redirects,&uri)
                    },
                    Some(Uri::DocumentElement(e)) => {
                        let uri = e.into();
                        leaf(&self.url,&self.redirects,&uri)
                    },
                    _ => format!(
                        "{}/content/notations?uri={}",
                        self.url,
                        uri.as_query(),
                    )
                };
                super::call::<_,SFnE>(url).map_err(BackendError::from_other)
            }
        }
    }

    #[derive(Debug)]
    struct SFnE(ServerFnErrorErr);
    impl From<SFnE> for ServerFnErrorErr {
        #[inline]
        fn from(value: SFnE) -> Self {
            value.0
        }
    }
    impl From<super::RequestError> for SFnE {
        fn from(value: super::RequestError) -> Self {
            Self(match value {
                super::RequestError::Request(r) => ServerFnErrorErr::Request(r),
                super::RequestError::Deserialization(r) => ServerFnErrorErr::Deserialization(r),
            })
        }
    }
    impl std::str::FromStr for SFnE {
        type Err = BackendError<Self>;
        fn from_str(string: &str) -> Result<Self, Self::Err> {
            let Some(j) = string.find('|') else {
                return Err(BackendError::Connection(Self(
                    ServerFnErrorErr::Deserialization(format!(
                        "Invalid format: missing delimiter in {string:?}"
                    )),
                )));
            };
            if j == 0 {
                return Err(BackendError::Connection(Self(
                    ServerFnErrorErr::Deserialization(format!(
                        "Invalid format: missing delimiter in {string:?}"
                    )),
                )));
            }
            let data = string[j + 1..].to_string();
            let prefix = &string[..string.len() - 1];
            crate::server_fn_impl::decode_server_fn(prefix, data)
                .map(Self)
                .map_err(|e| BackendError::Connection(Self(ServerFnErrorErr::Deserialization(e))))
        }
    }
}

#[cfg(feature = "wasm")]
fn call<R, E>(url: String) -> impl Future<Output = Result<R, BackendError<E>>>
where
    R: serde::de::DeserializeOwned,
    E: From<RequestError> + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    #[allow(clippy::future_not_send)]
    async fn call_i<R, E>(url: String) -> Result<R, BackendError<E>>
    where
        R: serde::de::DeserializeOwned,
        E: From<RequestError> + std::fmt::Debug + std::str::FromStr,
        E::Err: Into<BackendError<E>>,
    {
        let res = reqwasm::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;

        let status = res.status();
        if (400..=599).contains(&status) {
            let str = res
                .text()
                .await
                .map_err(|e| BackendError::Connection(E::from(e.into())))?;
            return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
        }

        res.json::<R>()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))
    }

    crate::utils::FutWrap::new(call_i(url))
}

#[cfg(not(feature = "wasm"))]
async fn call<R, E>(url: String) -> Result<R, BackendError<E>>
where
    R: serde::de::DeserializeOwned,
    E: From<RequestError> + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let res = ::reqwest::get(&url)
        .await
        .map_err(|e| BackendError::Connection(E::from(e.into())))?;

    let status = res.status().as_u16();
    if (400..=599).contains(&status) {
        let str = res
            .text()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
    }

    res.json::<R>()
        .await
        .map_err(|e| BackendError::Connection(E::from(e.into())))
}

#[cfg(feature = "wasm")]
impl From<reqwasm::Error> for RequestError {
    fn from(value: reqwasm::Error) -> Self {
        match value {
            reqwasm::Error::JsError(j) => Self::Request(j.to_string()),
            reqwasm::Error::SerdeError(e) => Self::Deserialization(e.to_string()),
        }
    }
}

#[cfg(feature = "reqwest")]
impl From<::reqwest::Error> for RequestError {
    fn from(value: ::reqwest::Error) -> Self {
        let value = value.without_url();
        if value.is_body() || value.is_decode() {
            Self::Deserialization(value.to_string())
        } else {
            Self::Request(value.to_string())
        }
    }
}
