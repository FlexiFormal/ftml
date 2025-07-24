#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

#[cfg(feature = "wasm")]
mod wasm;
use futures_util::FutureExt;
#[cfg(feature = "wasm")]
pub use wasm::*;

use ftml_ontology::utils::Css;
use ftml_uris::{ArchiveId, DocumentUri, Language, Uri};

pub const DEFAULT_SERVER_URL: &str = "https://mathhub.info";

pub trait FtmlBackend {
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError>>;
}

pub trait FlamsBackend {
    #[allow(clippy::too_many_arguments)]
    fn get_fragment(
        &self,
        uri: Option<Uri>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        m: Option<String>,
        l: Option<Language>,
        e: Option<String>,
        s: Option<String>,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(Uri, Vec<Css>, String), BackendError>>;
}

impl<FB> FtmlBackend for FB
where
    FB: FlamsBackend,
{
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError>> {
        <Self as FlamsBackend>::get_fragment(
            self,
            Some(uri),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            context,
        )
        .map(|r| r.map(|(_, css, s)| (s, css)))
    }
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum BackendError {
    #[cfg(feature = "server_fn")]
    #[error("server error: {0}")]
    ServerFn(#[from] server_fn::error::ServerFnErrorErr),
    #[cfg(feature = "server_fn")]
    #[error("error serializing error")]
    ErrorSerializing,
    #[cfg(feature = "server_fn")]
    #[error("error deserializing error: {0}")]
    ErrorDeserializing(String),
    #[error("invalid uri components: {0}")]
    InvalidUriComponent(#[from] ftml_uris::components::ComponentError),
    #[error("{0} not found")]
    NotFound(ftml_uris::UriKind),
    #[error("no html for document")]
    HtmlNotFound,
    #[error("element does not have a fragment")]
    NoFragment,
    #[error("element does not have a fragment")]
    NoDefinition,
    #[cfg(feature = "wasm")]
    #[error("request error: {0}")]
    Request(String),
}

#[cfg(feature = "server_fn")]
mod server_fn_impl {
    #[cfg(feature = "server_fn")]
    use server_fn::Decodes;
    use server_fn::Encodes;
    use server_fn::{
        Bytes,
        error::{FromServerFnError, ServerFnErrorEncoding, ServerFnErrorErr},
    };
    use std::fmt::Write;

    use crate::BackendError;

    impl FromServerFnError for BackendError {
        type Encoder = ServerFnErrorEncoding;
        #[inline]
        fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
            value.into()
        }
    }

    pub fn encode_server_fn(e: &ServerFnErrorErr) -> Result<Bytes, std::fmt::Error> {
        let mut buf = String::new();
        let result = match e {
            ServerFnErrorErr::Registration(e) => {
                write!(&mut buf, "Registration|{e}")
            }
            ServerFnErrorErr::Request(e) => write!(&mut buf, "Request|{e}"),
            ServerFnErrorErr::Response(e) => write!(&mut buf, "Response|{e}"),
            ServerFnErrorErr::ServerError(e) => {
                write!(&mut buf, "ServerError|{e}")
            }
            ServerFnErrorErr::MiddlewareError(e) => {
                write!(&mut buf, "MiddlewareError|{e}")
            }
            ServerFnErrorErr::Deserialization(e) => {
                write!(&mut buf, "Deserialization|{e}")
            }
            ServerFnErrorErr::Serialization(e) => {
                write!(&mut buf, "Serialization|{e}")
            }
            ServerFnErrorErr::Args(e) => write!(&mut buf, "Args|{e}"),
            ServerFnErrorErr::MissingArg(e) => {
                write!(&mut buf, "MissingArg|{e}")
            }
            ServerFnErrorErr::UnsupportedRequestMethod(req) => {
                write!(&mut buf, "UnsupportedRequestMethod|{req}")
            }
        };

        match result {
            Ok(()) => Ok(Bytes::from(buf)),
            Err(e) => Err(e),
        }
    }

    impl Encodes<BackendError> for ServerFnErrorEncoding {
        type Error = BackendError;

        fn encode(output: &BackendError) -> Result<Bytes, Self::Error> {
            let mut buf = String::new();
            let result = match output {
                BackendError::ServerFn(e) => {
                    return encode_server_fn(e).map_err(|_| BackendError::ErrorSerializing);
                }
                #[cfg(feature = "wasm")]
                BackendError::Request(e) => {
                    write!(&mut buf, "Request|{e}")
                }
                BackendError::ErrorSerializing => {
                    write!(&mut buf, "Serialization|")
                }
                BackendError::ErrorDeserializing(s) => write!(&mut buf, "Serialization|{s}"),
                BackendError::InvalidUriComponent(u) => write!(
                    &mut buf,
                    "InvalidUri|{}",
                    serde_json::to_string(u).map_err(|_| BackendError::ErrorSerializing)?
                ),
                BackendError::NotFound(u) => write!(
                    &mut buf,
                    "NotFound|{}",
                    serde_json::to_string(u).map_err(|_| BackendError::ErrorSerializing)?
                ),
                BackendError::HtmlNotFound => write!(&mut buf, "HtmlNotFound|"),
                BackendError::NoFragment => write!(&mut buf, "NoFragment|"),
                BackendError::NoDefinition => write!(&mut buf, "NoDefinition|"),
            }
            .map_err(|_| BackendError::ErrorSerializing);
            match result {
                Ok(()) => Ok(Bytes::from(buf)),
                Err(e) => Err(e),
            }
        }
    }

    pub fn decode_server_fn(ty: &str, data: String) -> Result<ServerFnErrorErr, String> {
        match ty {
            "Registration" => Ok(ServerFnErrorErr::Registration(data)),
            "Request" => Ok(ServerFnErrorErr::Request(data)),
            "Response" => Ok(ServerFnErrorErr::Response(data)),
            "ServerError" => Ok(ServerFnErrorErr::ServerError(data)),
            "MiddlewareError" => Ok(ServerFnErrorErr::MiddlewareError(data)),
            "Deserialization" => Ok(ServerFnErrorErr::Deserialization(data)),
            "Serialization" => Ok(ServerFnErrorErr::Serialization(data)),
            "Args" => Ok(ServerFnErrorErr::Args(data)),
            "MissingArg" => Ok(ServerFnErrorErr::MissingArg(data)),
            "UnsupportedRequestMethod" => Ok(ServerFnErrorErr::UnsupportedRequestMethod(data)),
            _ => Err(data),
        }
    }

    impl Decodes<BackendError> for ServerFnErrorEncoding {
        type Error = BackendError;

        fn decode(bytes: server_fn::Bytes) -> Result<BackendError, Self::Error> {
            let mut prefix = String::from_utf8(bytes.to_vec()).map_err(|err| {
                BackendError::ErrorDeserializing(format!("UTF-8 conversion error: {err}"))
            })?;
            let Some(j) = prefix.find('|') else {
                return Err(BackendError::ErrorDeserializing(format!(
                    "Invalid format: missing delimiter in {prefix:?}"
                )));
            };
            if j == 0 {
                return Err(BackendError::ErrorDeserializing(format!(
                    "Invalid format: missing delimiter in {prefix:?}"
                )));
            }
            let data = prefix.split_off(j + 1);
            let prefix = &prefix[..prefix.len() - 1];
            let data = match decode_server_fn(prefix, data) {
                Ok(e) => return Ok(e.into()),
                Err(e) => e,
            };
            match prefix {
                "InvalidUri" => Ok(BackendError::InvalidUriComponent(
                    serde_json::from_str(&data)
                        .map_err(|e| BackendError::ErrorDeserializing(e.to_string()))?,
                )),
                "NotFound" => Ok(BackendError::NotFound(
                    serde_json::from_str(&data)
                        .map_err(|e| BackendError::ErrorDeserializing(e.to_string()))?,
                )),
                "HtmlNotFound" => Ok(BackendError::HtmlNotFound),
                "NoFragment" => Ok(BackendError::NoFragment),
                "NoDefinition" => Ok(BackendError::NoDefinition),
                //"Serialization" => Err(BackendError::ErrorDeserializing(data)),
                _ => Err(BackendError::ErrorDeserializing(data)),
            }
        }
    }
}
