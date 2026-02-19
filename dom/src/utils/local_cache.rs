use either::Either;
use ftml_backend::{BackendError, FtmlBackend, GlobalBackend};
use ftml_ontology::{
    domain::{
        SharedDeclaration,
        declarations::{
            morphisms::Morphism,
            structures::{MathStructure, StructureExtension},
            symbols::Symbol,
        },
        modules::{Module, ModuleLike},
    },
    narrative::{
        SharedDocumentElement,
        documents::{Document, TocElem},
        elements::{
            DocumentTerm, Notation, ParagraphOrProblemKind, SectionLevel, VariableDeclaration,
            problems::Solutions,
        },
    },
    utils::Css,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, Uri, UriKind,
};
use std::{hint::unreachable_unchecked, marker::PhantomData};

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
    pub(crate) paragraphs: Map<DocumentElementUri, Box<str>>,
    pub(crate) dochtmls: Map<DocumentUri, Box<str>>,
    pub(crate) solutions: Map<DocumentElementUri, Solutions>,
}
impl LocalCache {
    pub fn resource<B: SendBackend, R, Fut>(
        f: impl FnOnce(WithLocalCache<B>) -> Fut + Send + Sync + 'static + Clone,
    ) -> leptos::prelude::RwSignal<
        Option<Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>>,
    >
    where
        R: Send + Sync + 'static + Clone,
        Fut: Future<
                Output = Result<R, ftml_backend::BackendError<<B::Backend as FtmlBackend>::Error>>,
            > + 'static,
    {
        use leptos::prelude::*;
        let result = RwSignal::new(None);
        leptos::task::spawn_local(async move {
            let r = f(WithLocalCache::default()).await;
            result.set(Some(r));
        });
        result
    }
}

pub(crate) static LOCAL_CACHE: std::sync::LazyLock<LocalCache> =
    std::sync::LazyLock::new(|| LocalCache {
        notations: Map::default(),
        documents: Set::default(),
        modules: Set::default(),
        fors: Map::default(),
        paragraphs: Map::default(),
        dochtmls: Map::default(),
        solutions: Map::default(),
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
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.local
            .as_deref()
            .unwrap_or_default()
            .iter()
            .chain(if let Some(Ok(v)) = &self.global {
                either::Left(v.iter())
            } else {
                either::Right(std::iter::empty())
            })
    }
}

impl LocalCache {
    pub fn add_module(m: Module) {
        LOCAL_CACHE.modules.insert(m);
    }
    pub fn remove_module(uri: &ModuleUri) {
        LOCAL_CACHE.modules.remove(uri);
    }
    pub fn add_definition(
        uri: DocumentElementUri,
        html: Box<str>,
        fors: impl Iterator<Item = SymbolUri>,
    ) {
        for s in fors {
            let mut e = LOCAL_CACHE.fors.entry(s).or_default();
            e.value_mut()
                .push((uri.clone(), ParagraphOrProblemKind::Definition));
        }
        LOCAL_CACHE.paragraphs.insert(uri, html);
    }
    pub fn remove_definition(uri: &DocumentElementUri) {
        for mut e in LOCAL_CACHE.fors.iter_mut() {
            e.value_mut().retain(|(u, _)| *u != *uri);
        }
        LOCAL_CACHE.paragraphs.remove(uri);
    }
}

impl<B: SendBackend> WithLocalCache<B> {
    pub fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<B::Error>>> + Send + use<B>
    {
        if let Uri::DocumentElement(uri) = &uri
            && let Some(s) = LOCAL_CACHE.paragraphs.get(uri)
        {
            either::Either::Left(std::future::ready(Ok((s.clone(), Box::new([]) as _, true))))
        } else {
            either::Either::Right(B::get().get_fragment(uri, context))
        }
    }

    pub fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<B::Error>>> + Send + use<B>
    {
        if let Some(v) = LOCAL_CACHE.fors.get(&uri)
            && let Some((uri, _)) = v
                .iter()
                .find(|(_, k)| matches!(k, ParagraphOrProblemKind::Definition))
            && let Some(s) = LOCAL_CACHE.paragraphs.get(uri)
        {
            return either::Either::Left(std::future::ready(Ok((
                s.clone(),
                Box::new([]) as _,
                true,
            ))));
        }
        either::Either::Right(self.get_fragment(uri.into(), context))
    }

    pub fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<Output = Result<ModuleLike, BackendError<B::Error>>> + Send + use<B> {
        use futures_util::TryFutureExt;
        if uri.is_top() {
            if let Some(m) = LOCAL_CACHE.modules.get(&uri) {
                return either::Left(either::Left(std::future::ready(Ok(ModuleLike::Module(
                    m.clone(),
                )))));
            }
            either::Left(either::Right(B::get().get_module(uri)))
        } else {
            let Some(SymbolUri { name, module }) = uri.into_symbol() else {
                // SAFETY: uri is not a top-level module uri, so it is compatible with a symbol URI
                unsafe { unreachable_unchecked() }
            };
            let fut = LOCAL_CACHE.modules.get(&module).map_or_else(
                || {
                    either::Right(B::get().get_module(module).map_ok(|m| {
                        let ModuleLike::Module(m) = m else {
                            // SAFETY: A top-level module uri can only resolve to a top-level module
                            unsafe { unreachable_unchecked() }
                        };
                        m
                    }))
                },
                |m| either::Left(std::future::ready(Ok(m.clone()))),
            );
            either::Right(fut.and_then(move |m| {
                std::future::ready(
                    m.as_module_like(&name)
                        .ok_or(BackendError::NotFound(ftml_uris::UriKind::Symbol)),
                )
            }))
        }
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

    #[inline]
    pub fn get_toc(
        &self,
        uri: DocumentUri,
    ) -> impl Future<
        Output = Result<(Box<[Css]>, SectionLevel, Box<[TocElem]>), BackendError<B::Error>>,
    > + Send
    + use<B> {
        B::get().get_toc(uri)
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
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<B::Error>>> + Send + use<B>
    {
        LOCAL_CACHE.dochtmls.get(&uri).map_or_else(
            || either::Either::Right(B::get().get_document_html(uri, context)),
            |s| either::Either::Left(std::future::ready(Ok((s.clone(), Box::new([]) as _, true)))),
        )
    }

    #[allow(clippy::unused_self)]
    pub fn get_solutions(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Solutions, BackendError<B::Error>>> + Send + use<B> {
        LOCAL_CACHE.solutions.get(&uri).map_or_else(
            || either::Either::Right(B::get().get_solutions(uri)),
            |s| either::Either::Left(std::future::ready(Ok(s.clone()))),
        )
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
