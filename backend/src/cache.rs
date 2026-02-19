use crate::{BackendError, FtmlBackend, ParagraphOrProblemKind};
use dashmap::Entry;
use ftml_ontology::{
    domain::modules::{Module, ModuleLike},
    narrative::{
        documents::{Document, TocElem},
        elements::{Notation, SectionLevel},
    },
    utils::Css,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, Uri,
};
use futures_util::TryFutureExt;
use kanal::AsyncSender;
use parking_lot::RwLock;
use std::{hint::unreachable_unchecked, sync::Arc};

#[derive(Debug, Clone, thiserror::Error, serde::Deserialize, serde::Serialize)]
pub enum CacheError<E: std::fmt::Debug> {
    #[error("channel sender closed")]
    SendClosed,
    #[error("channel receiver closed")]
    ReceiveClosed,
    #[error("{0}")]
    Connection(E),
}

impl<E: std::fmt::Display + std::fmt::Debug> From<CacheError<BackendError<E>>>
    for BackendError<CacheError<E>>
{
    fn from(value: CacheError<BackendError<E>>) -> Self {
        match value {
            CacheError::Connection(c) => match c {
                BackendError::Connection(c) => Self::Connection(CacheError::Connection(c)),
                BackendError::HtmlNotFound => Self::HtmlNotFound,
                BackendError::NoDefinition => Self::NoDefinition,
                BackendError::NoFragment => Self::NoFragment,
                BackendError::InvalidUriComponent(u) => Self::InvalidUriComponent(u),
                BackendError::NotFound(n) => Self::NotFound(n),
                BackendError::InvalidArgument(s) => Self::InvalidArgument(s),
                BackendError::ToDo(s) => Self::ToDo(s),
            },
            CacheError::ReceiveClosed => Self::Connection(CacheError::ReceiveClosed),
            CacheError::SendClosed => Self::Connection(CacheError::SendClosed),
        }
    }
}

pub struct CachedBackend<B: FtmlBackend>
where
    B::Error: Clone + Send + Sync,
{
    inner: B,
    #[allow(clippy::type_complexity)]
    fragment_cache:
        Cache<(Uri, Option<NarrativeUri>), (Box<str>, Box<[Css]>, bool), BackendError<B::Error>>,
    #[allow(clippy::type_complexity)]
    doc_html_cache: Cache<
        (DocumentUri, Option<NarrativeUri>),
        (Box<str>, Box<[Css]>, bool),
        BackendError<B::Error>,
    >,
    notations_cache: Cache<LeafUri, Vec<(DocumentElementUri, Notation)>, BackendError<B::Error>>,
    paragraphs_cache:
        Cache<SymbolUri, Vec<(DocumentElementUri, ParagraphOrProblemKind)>, BackendError<B::Error>>,
    modules_cache: Cache<ModuleUri, Module, BackendError<B::Error>>,
    documents_cache: Cache<DocumentUri, Document, BackendError<B::Error>>,
    toc_cache:
        Cache<DocumentUri, (Box<[Css]>, SectionLevel, Box<[TocElem]>), BackendError<B::Error>>,
}

impl<B: FtmlBackend> CachedBackend<B>
where
    B::Error: Clone + Send + Sync,
{
    #[inline]
    pub fn new(inner: B) -> Self {
        Self {
            inner,
            fragment_cache: Cache {
                map: dashmap::DashMap::default(),
            },
            doc_html_cache: Cache {
                map: dashmap::DashMap::default(),
            },
            notations_cache: Cache {
                map: dashmap::DashMap::default(),
            },
            paragraphs_cache: Cache {
                map: dashmap::DashMap::default(),
            },
            modules_cache: Cache {
                map: dashmap::DashMap::default(),
            },
            documents_cache: Cache {
                map: dashmap::DashMap::default(),
            },
            toc_cache: Cache {
                map: dashmap::DashMap::default(),
            },
        }
    }
}

impl<B: FtmlBackend> FtmlBackend for CachedBackend<B>
where
    B::Error: Clone + Send + Sync + std::fmt::Debug,
{
    type Error = CacheError<B::Error>;

    #[inline]
    fn check_term(
        &self,
        global_context: &[ModuleUri],
        term: &ftml_ontology::terms::Term,
        in_path: &ftml_ontology::terms::termpaths::TermPath,
    ) -> impl Future<Output = Result<crate::BackendCheckResult, BackendError<Self::Error>>> + Send + use<B>
    {
        self.inner
            .check_term(global_context, term, in_path)
            .map_err(|e| BackendError::ToDo(e.to_string()))
    }

    #[inline]
    fn document_link_url(&self, uri: &DocumentUri) -> String {
        self.inner.document_link_url(uri)
    }

    #[inline]
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String> {
        self.inner.resource_link_url(uri, kind)
    }

    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> {
        self.fragment_cache
            .get((uri, context), |(uri, context)| {
                self.inner.get_fragment(uri, context)
            })
            .map_err(Into::into)
    }

    fn get_solutions(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            ftml_ontology::narrative::elements::problems::Solutions,
            BackendError<Self::Error>,
        >,
    > + Send {
        // TODO returns wrong error type
        let fut = self.inner.get_solutions(uri);
        async move { fut.await.map_err(|e| BackendError::ToDo(e.to_string())) }
    }

    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> + Send
    {
        self.doc_html_cache
            .get((uri, context), |(uri, context)| {
                self.inner.get_document_html(uri, context)
            })
            .map_err(Into::into)
    }

    fn get_toc(
        &self,
        uri: DocumentUri,
    ) -> impl Future<
        Output = Result<(Box<[Css]>, SectionLevel, Box<[TocElem]>), BackendError<Self::Error>>,
    > + Send {
        self.toc_cache
            .get(uri, |uri| self.inner.get_toc(uri))
            .map_err(Into::into)
    }

    fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<Output = Result<ModuleLike, BackendError<Self::Error>>> {
        if uri.is_top() {
            either::Either::Left(
                self.modules_cache
                    .get(uri, |uri| {
                        self.inner.get_module(uri).map_ok(|m| {
                            let ModuleLike::Module(m) = m else {
                                // SAFETY: A top-level module uri can only resolve to a top-level module
                                unsafe { unreachable_unchecked() }
                            };
                            m
                        })
                    })
                    .map_ok(ModuleLike::Module)
                    .map_err(Into::into),
            )
        } else {
            let Some(SymbolUri { name, module }) = uri.into_symbol() else {
                // SAFETY: uri is not a top-level module uri, so it is compatible with a symbol URI
                unsafe { unreachable_unchecked() }
            };
            either::Either::Right(
                self.modules_cache
                    .get(module, |uri| {
                        self.inner.get_module(uri).map_ok(|m| {
                            let ModuleLike::Module(m) = m else {
                                // SAFETY: A top-level module uri can only resolve to a top-level module
                                unsafe { unreachable_unchecked() }
                            };
                            m
                        })
                    })
                    .map_err(Into::into)
                    .and_then(move |m| {
                        std::future::ready(
                            m.as_module_like(&name)
                                .ok_or(BackendError::NotFound(ftml_uris::UriKind::Symbol)),
                        )
                    }),
            )
        }
    }

    fn get_document(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<Document, BackendError<Self::Error>>> {
        self.documents_cache
            .get(uri, |uri| self.inner.get_document(uri))
            .map_err(Into::into)
    }

    fn get_notations(
        &self,
        uri: LeafUri,
    ) -> impl Future<Output = Result<Vec<(DocumentElementUri, Notation)>, BackendError<Self::Error>>>
    {
        self.notations_cache
            .get(uri, |uri| self.inner.get_notations(uri))
            .map_err(Into::into)
    }

    fn get_logical_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> impl Future<
        Output = Result<
            Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
            BackendError<Self::Error>,
        >,
    > {
        self.paragraphs_cache
            .get(uri, move |uri| {
                self.inner.get_logical_paragraphs(uri, problems)
            })
            .map_err(Into::into)
    }
    fn get_notation(
        &self,
        symbol: LeafUri,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Notation, BackendError<Self::Error>>> {
        self.notations_cache
            .with(
                symbol,
                |v| self.inner.get_notations(v),
                move |v| v.iter().find(|(u, _)| *u == uri).map(|(_, n)| n.clone()),
            )
            .map_ok_or_else(
                |e| Err(e.into()),
                |v| v.ok_or(BackendError::NotFound(ftml_uris::UriKind::DocumentElement)),
            )
    }
}

#[derive(Clone)]
enum MaybeValue<T: Clone, E: Clone> {
    Done(Result<T, E>),
    Pending(kanal::AsyncReceiver<Result<T, E>>),
}

struct Cache<K, V, E: std::fmt::Debug + Clone + Send + Sync>
where
    K: std::hash::Hash + Clone + Eq,
    V: Clone + Send + Sync,
{
    map: dashmap::DashMap<K, Arc<RwLock<MaybeValue<V, E>>>, rustc_hash::FxBuildHasher>,
}
impl<K, V, E> Cache<K, V, E>
where
    K: std::hash::Hash + Clone + Eq,
    V: Clone + Send + Sync,
    E: std::fmt::Debug + std::fmt::Display + Clone + Send + Sync,
{
    fn get<Fut: Future<Output = Result<V, E>>>(
        &self,
        k: K,
        f: impl FnOnce(K) -> Fut,
    ) -> impl Future<Output = Result<V, CacheError<E>>> {
        use either::Either::{Left, Right};
        match self.map.entry(k.clone()) {
            Entry::Occupied(lock) => {
                let lock = lock.get();
                match &*lock.read() {
                    MaybeValue::Done(a) => Left(Left(std::future::ready(
                        a.clone().map_err(CacheError::Connection),
                    ))),
                    MaybeValue::Pending(k) => Left(Right(Self::recv(k.clone()))),
                }
            }
            Entry::Vacant(v) => {
                let (sender, receiver) = kanal::bounded_async(1);
                let receiver = Arc::new(RwLock::new(MaybeValue::Pending(receiver)));
                v.insert(receiver.clone());
                Right(Self::call(f(k), sender, receiver))
            }
        }
    }

    fn with<Fut: Future<Output = Result<V, E>>, R: Send>(
        &self,
        k: K,
        f: impl FnOnce(K) -> Fut,
        then: impl FnOnce(&V) -> R + Send,
    ) -> impl Future<Output = Result<R, CacheError<E>>> {
        use either::Either::{Left, Right};

        match self.map.entry(k.clone()) {
            Entry::Occupied(lock) => {
                let lock = lock.get();
                match &*lock.read() {
                    MaybeValue::Done(a) => Left(Left(std::future::ready(
                        a.as_ref()
                            .map(then)
                            .map_err(|e| CacheError::Connection(e.clone())),
                    ))),
                    MaybeValue::Pending(k) if !k.is_terminated() && !k.is_disconnected() => {
                        Left(Right(Self::recv_and_then(k.clone(), then)))
                    }
                    MaybeValue::Pending(_) => {
                        let (sender, receiver) = kanal::bounded_async(1);
                        *lock.write() = MaybeValue::Pending(receiver);
                        Right(Self::call_and_then(f(k), sender, lock.clone(), then))
                    }
                }
            }
            Entry::Vacant(v) => {
                let (sender, receiver) = kanal::bounded_async(1);
                let receiver = Arc::new(RwLock::new(MaybeValue::Pending(receiver)));
                v.insert(receiver.clone());
                Right(Self::call_and_then(f(k), sender, receiver, then))
            }
        }
    }

    async fn call_and_then<R>(
        f: impl Future<Output = Result<V, E>>,
        sender: AsyncSender<Result<V, E>>,
        receiver: Arc<RwLock<MaybeValue<V, E>>>,
        then: impl FnOnce(&V) -> R,
    ) -> Result<R, CacheError<E>> {
        let r = f.await;
        let ret = r
            .as_ref()
            .map(then)
            .map_err(|e| CacheError::Connection(e.clone()));
        {
            let mut lock = receiver.write();
            *lock = MaybeValue::Done(r.clone());
        }
        while sender.receiver_count() > 0 && !sender.is_disconnected() && !sender.is_closed() {
            let _ = sender.send(r.clone()).await;
        }
        drop(sender);
        ret
    }

    async fn call(
        f: impl Future<Output = Result<V, E>>,
        sender: AsyncSender<Result<V, E>>,
        receiver: Arc<RwLock<MaybeValue<V, E>>>,
    ) -> Result<V, CacheError<E>> {
        let r = f.await;
        {
            let mut lock = receiver.write();
            *lock = MaybeValue::Done(r.clone());
        }
        while sender.receiver_count() > 0 && !sender.is_disconnected() && !sender.is_closed() {
            let _ = sender.send(r.clone()).await;
        }
        drop(sender);
        r.map_err(CacheError::Connection)
    }

    #[inline]
    async fn recv_and_then<R>(
        k: kanal::AsyncReceiver<Result<V, E>>,
        then: impl FnOnce(&V) -> R,
    ) -> Result<R, CacheError<E>> {
        match k.recv().await {
            Ok(r) => {
                drop(k);
                r.map(|e| then(&e)).map_err(CacheError::Connection)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[inline]
    async fn recv(k: kanal::AsyncReceiver<Result<V, E>>) -> Result<V, CacheError<E>> {
        match k.recv().await {
            Ok(r) => {
                drop(k);
                r.map_err(CacheError::Connection)
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> From<kanal::SendError> for CacheError<E> {
    #[inline]
    fn from(value: kanal::SendError) -> Self {
        match value {
            kanal::SendError::Closed => Self::SendClosed,
            kanal::SendError::ReceiveClosed => Self::ReceiveClosed,
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> From<kanal::ReceiveError> for CacheError<E> {
    #[inline]
    fn from(value: kanal::ReceiveError) -> Self {
        match value {
            kanal::ReceiveError::Closed => Self::ReceiveClosed,
            kanal::ReceiveError::SendClosed => Self::SendClosed,
        }
    }
}

#[cfg(feature = "deepsize")]
impl<B: FtmlBackend> CachedBackend<B>
where
    B::Error: Clone + Send + Sync,
{
    #[allow(clippy::significant_drop_tightening)]
    pub fn cache_size(&self) -> CacheSize {
        use deepsize::DeepSizeOf;
        let mut num_notations = 0;
        let mut notations_bytes = 0;
        for n in &self.notations_cache.map {
            notations_bytes += std::mem::size_of::<LeafUri>();
            let value = n.value().read();
            let value = &*value;
            notations_bytes += std::mem::size_of_val(value);
            if let MaybeValue::Done(Ok(v)) = value {
                for v in v {
                    num_notations += 1;
                    notations_bytes +=
                        std::mem::size_of::<DocumentElementUri>() + v.1.deep_size_of();
                }
            }
        }
        let mut num_documents = 0;
        let mut documents_bytes = 0;
        for d in &self.documents_cache.map {
            documents_bytes += std::mem::size_of::<DocumentUri>();
            num_documents += 1;
            let value = d.value().read();
            let value = &*value;
            documents_bytes += std::mem::size_of_val(value);
            if let MaybeValue::Done(Ok(v)) = value {
                documents_bytes += v.deep_size_of();
            }
        }
        let mut num_modules = 0;
        let mut modules_bytes = 0;
        for d in &self.modules_cache.map {
            num_modules += 1;
            modules_bytes += std::mem::size_of::<ModuleUri>();
            let value = d.value().read();
            let value = &*value;
            modules_bytes += std::mem::size_of_val(value);
            if let MaybeValue::Done(Ok(v)) = value {
                modules_bytes += v.deep_size_of();
            }
        }
        let mut num_fragments = 0;
        let mut fragments_bytes = 0;
        for d in &self.fragment_cache.map {
            num_fragments += 1;
            fragments_bytes += std::mem::size_of::<(Uri, Option<NarrativeUri>)>();
            let value = d.value().read();
            let value = &*value;
            fragments_bytes += std::mem::size_of_val(value);
            if let MaybeValue::Done(Ok((s, c, _))) = value {
                fragments_bytes += s.len();
                for c in c {
                    fragments_bytes += std::mem::size_of_val(c);
                    match c {
                        Css::Class { name, css } => fragments_bytes += name.len() + css.len(),
                        Css::Inline(i) => fragments_bytes += i.len(),
                        Css::Link(l) => fragments_bytes += l.len(),
                    }
                }
            }
        }
        let mut num_paragraphs = 0;
        let mut paragraphs_bytes = 0;
        for n in &self.paragraphs_cache.map {
            num_paragraphs += 1;
            paragraphs_bytes += std::mem::size_of::<SymbolUri>();
            let value = n.value().read();
            let value = &*value;
            fragments_bytes += std::mem::size_of_val(value);
            if let MaybeValue::Done(Ok(v)) = value {
                fragments_bytes +=
                    v.len() * std::mem::size_of::<(DocumentElementUri, ParagraphOrProblemKind)>();
            }
        }
        CacheSize {
            num_notations,
            notations_bytes,
            num_documents,
            documents_bytes,
            num_modules,
            modules_bytes,
            num_fragments,
            fragments_bytes,
            num_paragraphs,
            paragraphs_bytes,
        }
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
    pub num_fragments: usize,
    pub fragments_bytes: usize,
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
            + self.fragments_bytes
            + self.paragraphs_bytes
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
            num_fragments,
            fragments_bytes,
            num_paragraphs,
            paragraphs_bytes,
        } = self;
        write!(
            f,
            "\n\
             notations:  {num_notations} ({})\n\
             documents:  {num_documents} ({})\n\
             modules:    {num_modules} ({})\n\
             fragments:  {num_fragments} ({})\n\
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
            bytesize::ByteSize::b(*fragments_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*paragraphs_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(total as u64).display().iec_short(),
        )
    }
}
