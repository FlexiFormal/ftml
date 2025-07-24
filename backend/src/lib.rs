#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

#[cfg(feature = "wasm")]
mod utils;
#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::*;
#[cfg(feature = "cached")]
mod cache;
#[cfg(feature = "cached")]
pub use cache::*;

pub mod errors;
pub use errors::*;

use ftml_ontology::utils::Css;
use ftml_uris::{ArchiveId, DocumentUri, Language, Uri};
use futures_util::FutureExt;

pub const DEFAULT_SERVER_URL: &str = "https://mathhub.info";

pub trait FtmlBackend {
    type Error: std::fmt::Debug;
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send;
}

#[cfg(feature = "server_fn")]
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
    ) -> impl Future<
        Output = Result<(Uri, Vec<Css>, String), BackendError<server_fn::error::ServerFnErrorErr>>,
    > + Send;
}

#[cfg(feature = "server_fn")]
impl<FB> FtmlBackend for FB
where
    FB: FlamsBackend,
{
    type Error = server_fn::error::ServerFnErrorErr;
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send {
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
