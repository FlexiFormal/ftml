use std::marker::PhantomData;

use ftml_backend::{BackendError, FtmlBackend, GlobalBackend};
use ftml_ontology::{narrative::elements::Notation, utils::Css};
use ftml_uris::{DocumentElementUri, DocumentUri, LeafUri, NarrativeUri, SymbolUri};

pub trait SendBackend:
    GlobalBackend<Error: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone>
{
}
impl<G: GlobalBackend> SendBackend for G where
    G::Error: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone
{
}

pub struct LocalCache {
    notations:
        dashmap::DashMap<LeafUri, Vec<(DocumentElementUri, Notation)>, rustc_hash::FxBuildHasher>,
}

static LOCAL_CACHE: std::sync::LazyLock<LocalCache> = std::sync::LazyLock::new(|| LocalCache {
    notations: dashmap::DashMap::default(),
});

pub struct WithLocalCache<B: SendBackend>(PhantomData<B>);
impl<B: SendBackend> Default for WithLocalCache<B> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<B: SendBackend> WithLocalCache<B> {
    #[inline]
    pub fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<B::Error>>> + Send + use<B>
    {
        B::get().get_fragment(uri, context)
    }

    #[inline]
    pub fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<B::Error>>> + Send + use<B>
    {
        self.get_fragment(uri.into(), context)
    }

    #[inline]
    pub fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<B::Error>>> + Send + use<B>
    {
        self.get_fragment(uri.into(), context)
    }

    pub fn get_notations(
        &self,
        uri: LeafUri,
    ) -> impl Future<Output = Result<Vec<(DocumentElementUri, Notation)>, BackendError<B::Error>>>
    + Send
    + use<B> {
        B::get().get_notations(uri)
    }

    pub fn get_notation(
        &self,
        symbol: LeafUri,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Notation, BackendError<B::Error>>> + Send + use<B> {
        use either::Either::{Left, Right};
        if let Some(v) = LOCAL_CACHE.notations.get(&symbol) {
            for (u, n) in v.iter() {
                if *u == uri {
                    return Left(std::future::ready(Ok(n.clone())));
                }
            }
        }
        Right(B::get().get_notation(symbol, uri))
    }
}
