use either::Either;
use ftml_backend::{BackendError, FtmlBackend, GlobalBackend, ParagraphOrProblemKind};
use ftml_ontology::{
    domain::{
        SharedDeclaration,
        declarations::{
            morphisms::Morphism,
            structures::{MathStructure, StructureExtension},
            symbols::Symbol,
        },
        modules::Module,
    },
    narrative::{
        SharedDocumentElement,
        documents::Document,
        elements::{DocumentTerm, Notation, VariableDeclaration},
    },
    utils::Css,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, Uri, UriKind,
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
impl<T, E> GlobalLocal<Vec<T>, E> {
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.local
            .unwrap_or_default()
            .into_iter()
            .chain(if let Some(Ok(v)) = self.global {
                either::Left(v.into_iter())
            } else {
                either::Right(std::iter::empty())
            })
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
        if let Uri::DocumentElement(uri) = &uri
            && let Some(s) = LOCAL_CACHE.paragraphs.get(uri)
        {
            either::Either::Left(std::future::ready(Ok((s.clone(), Vec::new()))))
        } else {
            either::Either::Right(B::get().get_fragment(uri, context))
        }
    }

    #[inline]
    pub fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<B::Error>>> + Send + use<B>
    {
        if let Some(v) = LOCAL_CACHE.fors.get(&uri)
            && let Some((uri, _)) = v
                .iter()
                .find(|(_, k)| matches!(k, ParagraphOrProblemKind::Definition))
            && let Some(s) = LOCAL_CACHE.paragraphs.get(uri)
        {
            return either::Either::Left(std::future::ready(Ok((s.clone(), Vec::new()))));
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
        let uri = uri.simple_module();
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

    pub fn get_morphism(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<Either<Morphism, SharedDeclaration<Morphism>>, BackendError<B::Error>>,
    > + Send
    + use<B> {
        let uri = uri.simple_module();
        if let Some(m) = LOCAL_CACHE.modules.get(&uri.module) {
            let r = m
                .get_as::<Morphism>(&uri.name)
                .map_or(Err(BackendError::NotFound(UriKind::Symbol)), |d| {
                    Ok(either::Either::Right(d))
                });
            return either::Either::Left(std::future::ready(r));
        }
        either::Either::Right(B::get().get_morphism(uri))
    }

    #[allow(clippy::type_complexity)]
    pub fn get_structure(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
            BackendError<B::Error>,
        >,
    > + Send
    + use<B> {
        let uri = uri.simple_module();
        if let Some(m) = LOCAL_CACHE.modules.get(&uri.module) {
            let r = m.get_as::<MathStructure>(&uri.name).map_or_else(
                || {
                    m.get_as::<StructureExtension>(&uri.name)
                        .map_or(Err(BackendError::NotFound(UriKind::Symbol)), |d| {
                            Ok(either::Either::Right(d))
                        })
                },
                |d| Ok(either::Either::Left(d)),
            );
            return either::Either::Left(std::future::ready(r));
        }
        either::Either::Right(B::get().get_structure(uri))
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
        Output = GlobalLocal<
            Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
            BackendError<B::Error>,
        >,
    > + Send
    + use<B> {
        async move {
            let local = LOCAL_CACHE.fors.get(&uri).as_deref().cloned();
            let global = B::get().get_logical_paragraphs(uri, problems).await;
            GlobalLocal {
                local,
                global: Some(global),
            }
        }
    }

    pub fn get_notation(
        &self,
        symbol: Option<LeafUri>,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Notation, BackendError<B::Error>>> + Send + use<B> {
        use either::Either::{Left, Right};
        let local = symbol.as_ref().map_or_else(
            || {
                LOCAL_CACHE.notations.iter().find_map(|e| {
                    e.value()
                        .iter()
                        .find_map(|(u, n)| if *u == uri { Some(n.clone()) } else { None })
                })
            },
            |symbol| {
                LOCAL_CACHE.notations.get(symbol).and_then(|v| {
                    v.iter()
                        .find_map(|(u, n)| if *u == uri { Some(n.clone()) } else { None })
                })
            },
        );
        local.map_or_else(
            || {
                symbol.map_or_else(
                    || {
                        Left(std::future::ready(Err(BackendError::NotFound(
                            ftml_uris::UriKind::DocumentElement,
                        ))))
                    },
                    |symbol| Right(B::get().get_notation(symbol, uri)),
                )
            },
            |n| Left(std::future::ready(Ok(n))),
        )
    }
}

#[cfg(feature = "deepsize")]
pub struct CacheSize {
    pub num_notations: usize,
    pub notations_bytes: usize,
    pub num_documents: usize,
    pub documents_bytes: usize,
    pub num_modules: usize,
    pub modules_bytes: usize,
    pub num_fors: usize,
    pub fors_bytes: usize,
    pub num_paragraphs: usize,
    pub paragraphs_bytes: usize,
}
#[cfg(feature = "deepsize")]
impl CacheSize {
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.notations_bytes
            + self.documents_bytes
            + self.modules_bytes
            + self.fors_bytes
            + self.paragraphs_bytes
    }
}

#[cfg(feature = "deepsize")]
pub fn cache_size() -> CacheSize {
    use deepsize::DeepSizeOf;
    let mut num_notations = 0;
    let mut notations_bytes = 0;
    for n in &LOCAL_CACHE.notations {
        notations_bytes += std::mem::size_of::<LeafUri>();
        let value = n.value();
        notations_bytes += std::mem::size_of_val(value);
        for v in value {
            num_notations += 1;
            notations_bytes += std::mem::size_of::<DocumentElementUri>() + v.1.deep_size_of();
        }
    }
    let mut num_documents = 0;
    let mut documents_bytes = 0;
    for d in LOCAL_CACHE.documents.iter() {
        num_documents += 1;
        documents_bytes += d.deep_size_of();
    }
    let mut num_modules = 0;
    let mut modules_bytes = 0;
    for d in LOCAL_CACHE.modules.iter() {
        num_modules += 1;
        modules_bytes += d.deep_size_of();
    }
    let mut num_fors = 0;
    let mut fors_bytes = 0;
    for n in &LOCAL_CACHE.fors {
        fors_bytes += std::mem::size_of::<SymbolUri>();
        let value = n.value();
        fors_bytes += std::mem::size_of_val(value);
        num_fors = value.len();
        fors_bytes +=
            value.len() * std::mem::size_of::<(DocumentElementUri, ParagraphOrProblemKind)>();
    }
    let mut num_paragraphs = 0;
    let mut paragraphs_bytes = 0;
    for n in &LOCAL_CACHE.paragraphs {
        num_paragraphs += 1;
        paragraphs_bytes += std::mem::size_of::<(DocumentElementUri, String)>();
        paragraphs_bytes += n.value().len();
    }
    CacheSize {
        num_notations,
        notations_bytes,
        num_documents,
        documents_bytes,
        num_modules,
        modules_bytes,
        num_fors,
        fors_bytes,
        num_paragraphs,
        paragraphs_bytes,
    }
}

#[cfg(feature = "deepsize")]
impl std::fmt::Display for CacheSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.total_bytes();
        let Self {
            num_notations,
            notations_bytes,
            num_documents,
            documents_bytes,
            num_modules,
            modules_bytes,
            num_fors,
            fors_bytes,
            num_paragraphs,
            paragraphs_bytes,
        } = self;
        write!(
            f,
            "\n\
             notations:  {num_notations} ({})\n\
             documents:  {num_documents} ({})\n\
             modules:    {num_modules} ({})\n\
             fors:       {num_fors} ({})\n\
             paragraphs  {num_paragraphs} ({})\n\
             ----------------------------------\n\
             total: {}
            ",
            bytesize::ByteSize::b(*notations_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*documents_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*modules_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*fors_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*paragraphs_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(total as u64).display().iec_short(),
        )
    }
}
