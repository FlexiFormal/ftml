use std::marker::PhantomData;

use ftml_backend::{BackendError, FtmlBackend, GlobalBackend, ParagraphOrProblemKind};
use ftml_ontology::{
    domain::modules::Module,
    narrative::{documents::Document, elements::Notation},
    utils::Css,
};
use ftml_uris::{DocumentElementUri, DocumentUri, LeafUri, NarrativeUri, SymbolUri};

pub trait SendBackend:
    GlobalBackend<Error: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone> + Send
{
}
impl<G: GlobalBackend + Send> SendBackend for G where
    G::Error: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone
{
}

type Map<A, B> = dashmap::DashMap<A, B, rustc_hash::FxBuildHasher>;
type Set<A> = dashmap::DashSet<A, rustc_hash::FxBuildHasher>;

pub struct LocalCache {
    pub(crate) notations: Map<LeafUri, Vec<(DocumentElementUri, Notation)>>,
    pub(crate) documents: Set<Document>,
    pub(crate) modules: Set<Module>,
    pub(crate) paragraphs: Set<(DocumentElementUri, ParagraphOrProblemKind)>,
}

pub(crate) static LOCAL_CACHE: std::sync::LazyLock<LocalCache> =
    std::sync::LazyLock::new(|| LocalCache {
        notations: Map::default(),
        documents: Set::default(),
        modules: Set::default(),
        paragraphs: Set::default(),
    });

pub struct WithLocalCache<B: SendBackend>(PhantomData<B>);
impl<B: SendBackend> Default for WithLocalCache<B> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlobalLocal<T, E> {
    pub global: Option<Result<T, E>>,
    pub local: Option<T>,
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
    ) -> impl Future<
        Output = GlobalLocal<Vec<(DocumentElementUri, Notation)>, BackendError<B::Error>>,
    > + Send
    + use<B> {
        async move {
            let local = LOCAL_CACHE.notations.get(&uri).as_deref().cloned();
            let global = B::get().get_notations(uri).await;
            GlobalLocal {
                local,
                global: Some(global),
            }
        }
    }

    pub fn get_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> impl Future<
        Output = Result<Vec<(DocumentElementUri, ParagraphOrProblemKind)>, BackendError<B::Error>>,
    > + Send
    + use<B> {
        B::get().get_logical_paragraphs(uri, problems)
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
