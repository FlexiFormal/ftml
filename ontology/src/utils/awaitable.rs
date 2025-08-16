use dashmap::Entry;

#[derive(Debug, thiserror::Error)]
#[error("internal channel error")]
pub struct ChannelError;

pub struct AsyncCache<
    K: std::hash::Hash + Clone + Eq,
    T: Clone + Send + Sync,
    E: Clone + From<ChannelError> + Send + Sync,
> {
    map: dashmap::DashMap<K, Awaitable<T, E>>,
}

impl<
    K: std::hash::Hash + Clone + Eq,
    T: Clone + Send + Sync,
    E: Clone + From<ChannelError> + Send + Sync,
> AsyncCache<K, T, E>
{
    pub fn get<Fut: Future<Output = Result<T, E>> + Send>(
        &self,
        k: K,
        f: impl FnOnce(K) -> Fut,
    ) -> impl Future<Output = Result<T, E>> {
        match self.map.entry(k) {
            Entry::Occupied(a) => either::Left(a.get().clone().get()),
            Entry::Vacant(v) => {
                let (a, ret) = Awaitable::new(f(v.key().clone()));
                v.insert(a);
                either::Right(ret.get())
            }
        }
    }

    /// # Errors
    pub fn get_sync<Fut: Future<Output = Result<T, E>> + Send>(
        &self,
        k: K,
        f: impl FnOnce(K) -> Fut,
    ) -> Result<T, E> {
        match self.map.entry(k) {
            Entry::Occupied(a) => a.get().clone().get_sync(),
            Entry::Vacant(v) => {
                let (a, ret) = Awaitable::new(f(v.key().clone()));
                v.insert(a);
                ret.get_sync()
            }
        }
    }
}

#[derive(Clone)]
pub struct Awaitable<T: Clone + Send + Sync, E: Clone + From<ChannelError> + Send + Sync> {
    inner: std::sync::Arc<parking_lot::RwLock<MaybeValue<T, E>>>,
}

#[derive(Clone)]
enum MaybeValue<T: Clone + Send + Sync, E: Clone + From<ChannelError> + Send + Sync> {
    Done(Result<T, E>),
    Pending(flume::Receiver<Result<T, E>>), //(kanal::AsyncReceiver<Result<T, E>>),
}
impl<T: Clone + Send + Sync, E: Clone + From<ChannelError> + Send + Sync> MaybeValue<T, E> {
    async fn get(self) -> Result<T, E> {
        match self {
            Self::Done(r) => r,
            Self::Pending(kanal) => kanal
                .recv_async()
                .await
                .unwrap_or_else(|_| Err(ChannelError.into())),
        }
    }
    fn get_sync(self) -> Result<T, E> {
        match self {
            Self::Done(r) => r,
            Self::Pending(kanal) => kanal.recv().unwrap_or_else(|_| Err(ChannelError.into())),
        }
    }
}

impl<T: Clone + Send + Sync, E: Clone + From<ChannelError> + Send + Sync> Awaitable<T, E> {
    pub fn get(self) -> impl Future<Output = Result<T, E>> {
        self.inner.read().clone().get()
    }

    /// # Errors
    pub fn get_sync(self) -> Result<T, E> {
        self.inner.read().clone().get_sync()
    }

    pub fn new<F: Future<Output = Result<T, E>> + Send>(
        future: F,
    ) -> (Self, AwaitableSource<T, E, F>) {
        let (sender, receiver) = flume::bounded(1); //kanal::bounded_async(1);
        let inner = std::sync::Arc::new(parking_lot::RwLock::new(MaybeValue::Pending(receiver)));
        (
            Self {
                inner: inner.clone(),
            },
            AwaitableSource {
                inner,
                sender,
                future,
            },
        )
    }
}

pub struct AwaitableSource<
    T: Clone + Send + Sync,
    E: Clone + From<ChannelError> + Send + Sync,
    F: Future<Output = Result<T, E>> + Send,
> {
    inner: std::sync::Arc<parking_lot::RwLock<MaybeValue<T, E>>>,
    sender: flume::Sender<Result<T, E>>, //kanal::AsyncSender<Result<T, E>>,
    future: F,
}

impl<
    T: Clone + Send + Sync,
    E: Clone + From<ChannelError> + Send + Sync,
    F: Future<Output = Result<T, E>> + Send,
> AwaitableSource<T, E, F>
{
    /// # Errors
    pub async fn get(self) -> Result<T, E> {
        let Self {
            inner,
            sender,
            future,
        } = self;
        let res = future.await;
        {
            let mut lock = inner.write();
            *lock = MaybeValue::Done(res.clone());
        }
        while sender.receiver_count() > 0 {
            let _ = sender.send_async(res.clone()).await;
        }
        res
    }

    /// # Errors
    pub fn get_sync(self) -> Result<T, E> {
        futures::executor::block_on(self.get())
    }
}
