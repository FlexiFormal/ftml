use dashmap::Entry;
use triomphe::Arc;

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("internal channel error")]
pub struct ChannelError;

#[derive(Debug)]
pub struct AsyncCache<
    K: std::hash::Hash + Clone + Eq,
    T: Clone + Send,
    E: Clone + From<ChannelError> + Send,
> {
    map: dashmap::DashMap<K, Awaitable<T, E>, rustc_hash::FxBuildHasher>,
}
impl<K: std::hash::Hash + Clone + Eq, T: Clone + Send, E: Clone + From<ChannelError> + Send> Default
    for AsyncCache<K, T, E>
{
    fn default() -> Self {
        Self {
            map: dashmap::DashMap::default(),
        }
    }
}

impl<
    K: std::hash::Hash + Clone + Eq,
    T: Clone + Send + Sync,
    E: Clone + From<ChannelError> + Send + Sync,
> AsyncCache<K, T, E>
{
    #[inline]
    pub fn clear(&self) {
        self.map.clear();
    }

    pub fn retain(&self, mut keep: impl FnMut(&K, &Result<T, E>) -> bool) {
        self.map.retain(|k, e| match &*e.inner.read() {
            MaybeValue::Pending(_) => true,
            MaybeValue::Done(v) => keep(k, v),
        });
    }

    pub fn get<Fut: Future<Output = Result<T, E>> + Send + Sync, F: FnOnce(K) -> Fut>(
        &self,
        k: K,
        f: F,
    ) -> impl Future<Output = Result<T, E>> + Send + use<Fut, T, E, K, F> {
        match self.map.entry(k) {
            Entry::Occupied(a) => either::Left(a.get().clone().get()),
            Entry::Vacant(v) => {
                let (a, ret) = Awaitable::new(f(v.key().clone()));
                v.insert(a);
                either::Right(ret.get())
            }
        }
    }

    /// blocks
    pub fn has<Q: std::hash::Hash + Eq + ?Sized>(&self, k: &Q) -> Option<Result<T, E>>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get(k).map(|r| r.inner.read().clone().get_sync())
    }

    /// Assumes f blocks
    /// # Errors
    pub fn get_sync(&self, k: K, f: impl FnOnce(K) -> Result<T, E>) -> Result<T, E> {
        match self.map.entry(k) {
            Entry::Occupied(a) => a.get().clone().get_sync(),
            Entry::Vacant(v) => {
                let (sender, receiver) = flume::bounded(1);
                let inner = Arc::new(parking_lot::RwLock::new(MaybeValue::Pending(receiver)));
                let key = v.key().clone();
                {
                    v.insert(Awaitable {
                        inner: inner.clone(),
                    });
                }
                let res = f(key);
                {
                    let mut lock = inner.write();
                    *lock = MaybeValue::Done(res.clone());
                }
                while sender.receiver_count() > 0 {
                    let _ = sender.send(res.clone());
                }
                res
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Awaitable<T: Clone + Send, E: Clone + From<ChannelError> + Send> {
    inner: Arc<parking_lot::RwLock<MaybeValue<T, E>>>,
}

#[derive(Clone, Debug)]
enum MaybeValue<T: Clone + Send, E: Clone + From<ChannelError> + Send> {
    Done(Result<T, E>),
    Pending(flume::Receiver<Result<T, E>>), //(kanal::AsyncReceiver<Result<T, E>>),
}
impl<T: Clone + Send, E: Clone + From<ChannelError> + Send> MaybeValue<T, E> {
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

impl<T: Clone + Send, E: Clone + From<ChannelError> + Send> Awaitable<T, E> {
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
        let inner = Arc::new(parking_lot::RwLock::new(MaybeValue::Pending(receiver)));
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
    T: Clone + Send,
    E: Clone + From<ChannelError> + Send,
    F: Future<Output = Result<T, E>> + Send,
> {
    inner: Arc<parking_lot::RwLock<MaybeValue<T, E>>>,
    sender: flume::Sender<Result<T, E>>, //kanal::AsyncSender<Result<T, E>>,
    future: F,
}

impl<
    T: Clone + Send + Sync,
    E: Clone + From<ChannelError> + Send + Sync,
    F: Future<Output = Result<T, E>> + Send + Sync,
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
        pollster::FutureExt::block_on(self.get())
    }
}

#[cfg(feature = "deepsize")]
impl<T: Clone + Send + deepsize::DeepSizeOf, E: Clone + From<ChannelError> + Send>
    deepsize::DeepSizeOf for MaybeValue<T, E>
{
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Done(Ok(v)) => v.deep_size_of_children(context),
            _ => 0,
        }
    }
}

#[cfg(feature = "deepsize")]
impl<
    K: std::hash::Hash + Clone + Eq,
    T: Clone + Send + deepsize::DeepSizeOf,
    E: Clone + From<ChannelError> + Send,
> deepsize::DeepSizeOf for AsyncCache<K, T, E>
{
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.map
            .iter()
            .map(|e| {
                std::mem::size_of::<K>() + std::mem::size_of_val(e.value()) + {
                    let value = e.value();
                    let value = value.inner.read();
                    // urgh, what's the overhead of a dashmap bucket...? Let's just add 8 bytes
                    // for good measure...
                    8 + std::mem::size_of_val(&*value) + value.deep_size_of_children(context)
                }
            })
            .sum()
    }
}
