use std::{marker::PhantomData, str::FromStr};

use ftml_ontology::utils::Css;
use ftml_uris::{DocumentUri, FtmlUri};

use crate::BackendError;

pub struct RemoteBackend<Url: std::fmt::Display, E: std::fmt::Debug + From<reqwasm::Error>> {
    pub fragment_url: Url,
    __phantom: PhantomData<E>,
}
impl<Url, E> RemoteBackend<Url, E>
where
    Url: std::fmt::Display,
    E: std::fmt::Debug + From<reqwasm::Error>,
{
    pub const fn new(fragment_url: Url) -> Self {
        Self {
            fragment_url,
            __phantom: PhantomData,
        }
    }
}

#[allow(clippy::future_not_send)]
async fn call_i<R, E>(url: String) -> Result<R, BackendError<E>>
where
    R: serde::de::DeserializeOwned,
    E: From<reqwasm::Error> + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let res = reqwasm::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| BackendError::Connection(E::from(e)))?;

    let status = res.status();
    if (400..=599).contains(&status) {
        let str = res
            .text()
            .await
            .map_err(|e| BackendError::Connection(E::from(e)))?;
        return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
    }

    res.json::<R>()
        .await
        .map_err(|e| BackendError::Connection(E::from(e)))
}

fn call<R, E>(url: String) -> impl Future<Output = Result<R, BackendError<E>>>
where
    R: serde::de::DeserializeOwned,
    E: From<reqwasm::Error> + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    crate::utils::FutWrap::new(call_i(url))
}

impl<Url, E> super::FtmlBackend for RemoteBackend<Url, E>
where
    Url: std::fmt::Display,
    E: std::fmt::Debug + From<reqwasm::Error> + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    type Error = E;
    #[allow(clippy::similar_names)]
    fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<E>>> {
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
        call(url)
    }
}

#[cfg(feature = "server_fn")]
pub struct RemoteFlamsBackend<Url: std::fmt::Display> {
    pub url: Url,
}

#[cfg(feature = "server_fn")]
mod server_fn {
    use crate::{BackendError, FlamsBackend, RemoteFlamsBackend};
    use ::server_fn::error::ServerFnErrorErr;
    use ftml_ontology::utils::Css;
    use ftml_uris::{DocumentUri, FtmlUri, Uri};
    use futures_util::TryFutureExt;

    impl<Url: std::fmt::Display> FlamsBackend for RemoteFlamsBackend<Url> {
        ftml_uris::compfun! {!!
            #[allow(clippy::similar_names)]
            fn get_fragment(&self,uri:Uri,context:Option<DocumentUri>) -> impl Future<Output=Result<(Uri, Vec<Css>,String), BackendError<ServerFnErrorErr>>> {
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
    impl From<reqwasm::Error> for SFnE {
        fn from(value: reqwasm::Error) -> Self {
            Self(match value {
                reqwasm::Error::JsError(j) => ServerFnErrorErr::Request(j.to_string()),
                reqwasm::Error::SerdeError(e) => ServerFnErrorErr::Deserialization(e.to_string()),
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
