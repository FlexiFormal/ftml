#[derive(Debug, Clone, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum BackendError<E: std::fmt::Debug> {
    #[error("{0}")]
    Connection(#[source] E),
    #[error("invalid uri components: {0}")]
    InvalidUriComponent(#[from] ftml_uris::components::ComponentError),
    #[error("{0} not found")]
    NotFound(ftml_uris::UriKind),
    #[error("no html for document")]
    HtmlNotFound,
    #[error("element does not have a fragment")]
    NoFragment,
    #[error("no definition for element found")]
    NoDefinition,
    #[error("not yet implemented")]
    ToDo(String),
}

impl<F: std::fmt::Display + std::fmt::Debug> BackendError<F> {
    pub fn from_other<I: std::fmt::Display + std::fmt::Debug + From<F>>(self) -> BackendError<I> {
        match self {
            Self::HtmlNotFound => BackendError::HtmlNotFound,
            Self::NoDefinition => BackendError::NoDefinition,
            Self::NoFragment => BackendError::NoFragment,
            Self::InvalidUriComponent(u) => BackendError::InvalidUriComponent(u),
            Self::NotFound(n) => BackendError::NotFound(n),
            Self::ToDo(t) => BackendError::ToDo(t),
            Self::Connection(c) => BackendError::Connection(c.into()),
        }
    }
}
impl<E: std::fmt::Display + std::fmt::Debug> BackendError<E> {
    /// ### Errors
    pub fn from_prefix_value_pair(value: String, prefix_len: usize) -> Result<Self, String> {
        fn js<V: serde::de::DeserializeOwned>(
            mut s: String,
            prefix_len: usize,
        ) -> Result<V, String> {
            let s = if s.get(prefix_len..).is_none_or(|v| !v.starts_with('|')) {
                return Err(s);
            } else {
                s.split_off(prefix_len + 1)
            };
            serde_json::from_str(&s).map_err(|_| s)
        }
        let Some(prefix) = value.get(0..prefix_len) else {
            return Err(value);
        };
        match prefix {
            "InvalidUri" => Ok(Self::InvalidUriComponent(js(value, prefix_len)?)),
            "NotFound" => Ok(Self::NotFound(js(value, prefix_len)?)),
            "HtmlNotFound" => Ok(Self::HtmlNotFound),
            "NoFragment" => Ok(Self::NoFragment),
            "NoDefinition" => Ok(Self::NoDefinition),
            "NotYetImplemented" => Ok(Self::ToDo(js(value, prefix_len)?)),
            _ => Err(value),
        }
    }
}
impl<E> std::str::FromStr for BackendError<E>
where
    E: std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<Self>,
{
    type Err = <E as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //TODO
        Ok(Self::Connection(E::from_str(s)?))
    }
}

#[cfg(feature = "server_fn")]
pub mod server_fn_impl {
    use crate::BackendError;
    use server_fn::{
        Bytes, ContentType, Decodes, Encodes, FormatType,
        error::{FromServerFnError, ServerFnErrorErr},
    };
    use std::fmt::Write;

    impl FromServerFnError for BackendError<ServerFnErrorErr> {
        type Encoder = Encoder;
        #[inline]
        fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
            Self::Connection(value)
        }
    }
    impl From<ServerFnErrorErr> for BackendError<ServerFnErrorErr> {
        #[inline]
        fn from(value: ServerFnErrorErr) -> Self {
            Self::Connection(value)
        }
    }

    pub struct Encoder;

    /// ### Errors
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

    impl FormatType for Encoder {
        const FORMAT_TYPE: server_fn::Format = server_fn::error::ServerFnErrorEncoding::FORMAT_TYPE;
    }
    impl ContentType for Encoder {
        const CONTENT_TYPE: &'static str = server_fn::error::ServerFnErrorEncoding::CONTENT_TYPE;
    }

    impl Encodes<BackendError<ServerFnErrorErr>> for Encoder {
        type Error = String;

        fn encode(output: &BackendError<ServerFnErrorErr>) -> Result<Bytes, Self::Error> {
            let mut buf = String::new();
            match output {
                BackendError::Connection(e) => {
                    return encode_server_fn(e).map_err(|_| "error serializing".to_string());
                }
                BackendError::InvalidUriComponent(u) => write!(
                    &mut buf,
                    "InvalidUri|{}",
                    serde_json::to_string(u).map_err(|e| format!("error serializing: {e}"))?
                ),
                BackendError::NotFound(u) => write!(
                    &mut buf,
                    "NotFound|{}",
                    serde_json::to_string(u).map_err(|e| format!("error serializing: {e}"))?
                ),
                BackendError::ToDo(u) => write!(
                    &mut buf,
                    "NotYetImplemented|{}",
                    serde_json::to_string(u).map_err(|e| format!("error serializing: {e}"))?
                ),
                BackendError::HtmlNotFound => {
                    buf.push_str("HtmlNotFound|");
                    Ok(())
                }
                BackendError::NoFragment => {
                    buf.push_str("NoFragment|");
                    Ok(())
                }
                BackendError::NoDefinition => {
                    buf.push_str("NoDefinition|");
                    Ok(())
                }
            }
            .map_err(|_| "Error deserializing".to_string())?;
            Ok(Bytes::from(buf))
        }
    }

    /// ### Errors
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

    /// ### Errors
    pub fn from_string(string: String) -> Result<BackendError<ServerFnErrorErr>, String> {
        let Some(j) = string.find('|') else {
            return Err(format!("Invalid format: missing delimiter in {string:?}"));
        };
        if j == 0 {
            return Err(format!("Invalid format: missing delimiter in {string:?}"));
        }
        let mut string = match BackendError::from_prefix_value_pair(string, j) {
            Ok(r) => return Ok(r),
            Err(s) => s,
        };

        let data = string.split_off(j + 1);
        let prefix = &string[..string.len() - 1];
        decode_server_fn(prefix, data).map(BackendError::Connection)
    }

    impl Decodes<BackendError<ServerFnErrorErr>> for Encoder {
        type Error = String;

        fn decode(bytes: Bytes) -> Result<BackendError<ServerFnErrorErr>, Self::Error> {
            let string = String::from_utf8(bytes.to_vec())
                .map_err(|err| format!("UTF-8 conversion error: {err}"))?;
            from_string(string)
        }
    }

    /*
        async fn run_client(path: &str, input: Input) -> Result<Output, E>
        where
            Client: crate::Client<E>,
        {
            // create and send request on client
            let req = input.into_req(path, OutputProtocol::CONTENT_TYPE)?;
            let res = Client::send(req).await?;

            let status = res.status();
            let location = res.location();
            let has_redirect_header = res.has_redirect();

            // if it returns an error status, deserialize the error using the error's decoder.
            let res = if (400..=599).contains(&status) {
                Err(E::de(res.try_into_bytes().await?))
            } else {
                // otherwise, deserialize the body as is
                let output = Output::from_res(res).await?;
                Ok(output)
            }?;

            // if redirected, call the redirect hook (if that's been set)
            if (300..=399).contains(&status) || has_redirect_header {
                call_redirect_hook(&location);
            }
            Ok(res)
        }
    */
}
