use either::Either;
use ftml_backend::{BackendError, FtmlBackend, GlobalBackend, ParagraphOrProblemKind};
use ftml_ontology::{
    domain::{SharedDeclaration, declarations::symbols::Symbol, modules::Module},
    narrative::{
        SharedDocumentElement,
        documents::Document,
        elements::{DocumentTerm, Notation, VariableDeclaration},
    },
    utils::Css,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, UriKind,
};
use std::marker::PhantomData;

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
    pub(crate) fors: Map<SymbolUri, Vec<(DocumentElementUri, ParagraphOrProblemKind)>>,
    pub(crate) paragraphs: Map<DocumentElementUri, String>,
}

pub(crate) static LOCAL_CACHE: std::sync::LazyLock<LocalCache> =
    std::sync::LazyLock::new(|| LocalCache {
        notations: Map::default(),
        documents: Set::default(),
        modules: Set::default(),
        fors: Map::default(),
        paragraphs: Map::default(),
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
        if let Some(v) = LOCAL_CACHE.fors.get(&uri) {
            if let Some((uri, _)) = v
                .iter()
                .find(|(_, k)| matches!(k, ParagraphOrProblemKind::Definition))
            {
                if let Some(s) = LOCAL_CACHE.paragraphs.get(uri) {
                    return either::Either::Left(std::future::ready(Ok((s.clone(), Vec::new()))));
                }
            }
        }
        either::Either::Right(self.get_fragment(uri.into(), context))
    }

    pub fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<Output = Result<Module, BackendError<B::Error>>> + Send + use<B> {
        if let Some(m) = LOCAL_CACHE.modules.get(&uri) {
            return either::Either::Left(std::future::ready(Ok(m.clone())));
        }
        either::Either::Right(B::get().get_module(uri))
    }

    pub fn get_document(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<Document, BackendError<B::Error>>> + Send + use<B> {
        if let Some(m) = LOCAL_CACHE.documents.get(&uri) {
            return either::Either::Left(std::future::ready(Ok(m.clone())));
        }
        either::Either::Right(B::get().get_document(uri))
    }

    pub fn get_document_term(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            Either<DocumentTerm, SharedDocumentElement<DocumentTerm>>,
            BackendError<B::Error>,
        >,
    > + Send
    + use<B> {
        if let Some(m) = LOCAL_CACHE.documents.get(&uri.document) {
            let r = m
                .get_as::<DocumentTerm>(&uri.name)
                .map_or(Err(BackendError::NotFound(UriKind::DocumentElement)), |d| {
                    Ok(either::Either::Right(d))
                });
            return either::Either::Left(std::future::ready(r));
        }
        either::Either::Right(B::get().get_document_term(uri))
    }

    pub fn get_symbol(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<Either<Symbol, SharedDeclaration<Symbol>>, BackendError<B::Error>>,
    > + Send
    + use<B> {
        if let Some(m) = LOCAL_CACHE.modules.get(&uri.module) {
            let r = m
                .get_as::<Symbol>(&uri.name)
                .map_or(Err(BackendError::NotFound(UriKind::Symbol)), |d| {
                    Ok(either::Either::Right(d))
                });
            return either::Either::Left(std::future::ready(r));
        }
        either::Either::Right(B::get().get_symbol(uri))
    }

    pub fn get_variable(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            Either<VariableDeclaration, SharedDocumentElement<VariableDeclaration>>,
            BackendError<B::Error>,
        >,
    > + Send
    + use<B> {
        if let Some(m) = LOCAL_CACHE.documents.get(&uri.document) {
            let r = m
                .get_as::<VariableDeclaration>(&uri.name)
                .map_or(Err(BackendError::NotFound(UriKind::DocumentElement)), |d| {
                    Ok(either::Either::Right(d))
                });
            return either::Either::Left(std::future::ready(r));
        }
        either::Either::Right(B::get().get_variable(uri))
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
