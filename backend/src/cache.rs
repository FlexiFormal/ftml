use crate::{BackendError, FtmlBackend};
use dashmap::Entry;
use ftml_ontology::utils::Css;
use ftml_uris::{DocumentUri, Uri};
use futures_util::TryFutureExt;
use kanal::AsyncSender;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, thiserror::Error, serde::Deserialize, serde::Serialize)]
pub enum CacheError<E: std::fmt::Debug> {
    #[error("channel closer")]
    Closed,
    #[error("channel sender closed")]
    SendClosed,
    #[error("channel receiver closed")]
    ReceiveClosed,
    #[error("{0}")]
    Connection(E),
}

impl<E: std::fmt::Debug> From<CacheError<BackendError<E>>> for BackendError<CacheError<E>> {
    fn from(value: CacheError<BackendError<E>>) -> Self {
        match value {
            CacheError::Connection(c) => match c {
                BackendError::Connection(c) => Self::Connection(CacheError::Connection(c)),
                BackendError::HtmlNotFound => Self::HtmlNotFound,
                BackendError::NoDefinition => Self::NoDefinition,
                BackendError::NoFragment => Self::NoFragment,
                BackendError::InvalidUriComponent(u) => Self::InvalidUriComponent(u),
                BackendError::NotFound(n) => Self::NotFound(n),
                BackendError::ToDo(s) => Self::ToDo(s),
            },
            CacheError::Closed => Self::Connection(CacheError::Closed),
            CacheError::ReceiveClosed => Self::Connection(CacheError::ReceiveClosed),
            CacheError::SendClosed => Self::Connection(CacheError::SendClosed),
        }
    }
}

pub struct CachedBackend<B: FtmlBackend>
where
    B::Error: Clone + Send + Sync + std::fmt::Debug,
{
    inner: B,
    #[allow(clippy::type_complexity)]
    fragment_cache: Cache<(Uri, Option<DocumentUri>), (String, Vec<Css>), BackendError<B::Error>>,
}
impl<B: FtmlBackend> FtmlBackend for CachedBackend<B>
where
    B::Error: Clone + Send + Sync + std::fmt::Debug,
{
    type Error = CacheError<B::Error>;
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<DocumentUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send {
        self.fragment_cache
            .get((uri, context), |(uri, context)| {
                self.inner.get_fragment(uri, context)
            })
            .map_err(Into::into)
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
    map: dashmap::DashMap<K, Arc<RwLock<MaybeValue<V, E>>>>,
}
impl<K, V, E> Cache<K, V, E>
where
    K: std::hash::Hash + Clone + Eq,
    V: Clone + Send + Sync,
    E: std::fmt::Debug + Clone + Send + Sync,
{
    fn get<Fut: Future<Output = Result<V, E>> + Send>(
        &self,
        k: K,
        f: impl FnOnce(K) -> Fut,
    ) -> impl Future<Output = Result<V, CacheError<E>>> + Send {
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
        if sender.receiver_count() > 1 {
            sender.send(r.clone()).await?;
        }
        drop(sender);
        r.map_err(CacheError::Connection)
    }

    #[inline]
    async fn recv(k: kanal::AsyncReceiver<Result<V, E>>) -> Result<V, CacheError<E>> {
        match k.recv().await {
            Ok(r) => r.map_err(CacheError::Connection),
            Err(e) => Err(e.into()),
        }
    }
}

impl<E: std::fmt::Debug> From<kanal::SendError> for CacheError<E> {
    #[inline]
    fn from(value: kanal::SendError) -> Self {
        match value {
            kanal::SendError::Closed => Self::Closed,
            kanal::SendError::ReceiveClosed => Self::ReceiveClosed,
        }
    }
}

impl<E: std::fmt::Debug> From<kanal::ReceiveError> for CacheError<E> {
    #[inline]
    fn from(value: kanal::ReceiveError) -> Self {
        match value {
            kanal::ReceiveError::Closed => Self::Closed,
            kanal::ReceiveError::SendClosed => Self::SendClosed,
        }
    }
}
