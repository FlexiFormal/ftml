#[derive(Debug, Clone, Copy, thiserror::Error, serde::Deserialize, serde::Serialize)]
#[error("internal cache error")]
pub struct CacheError;

#[derive(Debug)]
pub struct AsyncCache<
    Key: std::hash::Hash + Clone + Eq,
    Val: Clone + Send,
    Err: Clone + From<CacheError> + Send,
    Hash: std::hash::BuildHasher + Clone = std::hash::RandomState,
> {
    pub(crate) map: dashmap::DashMap<Key, Awaitable<Val, Err>, Hash>,
    max: Option<usize>,
}

impl<
    Key: std::hash::Hash + Clone + Eq,
    Val: Clone + Send,
    Err: Clone + From<CacheError> + Send,
    Hash: std::hash::BuildHasher + Clone + Default,
> Default for AsyncCache<Key, Val, Err, Hash>
{
    fn default() -> Self {
        Self {
            map: dashmap::DashMap::default(),
            max: None,
        }
    }
}

impl<
    Key: std::hash::Hash + Clone + Eq,
    Val: Clone + Send + Sync,
    Err: Clone + From<CacheError> + Send + Sync + 'static,
    Hash: std::hash::BuildHasher + Clone,
> AsyncCache<Key, Val, Err, Hash>
{
    #[must_use]
    pub fn new(max: usize) -> Self
    where
        Hash: Default,
    {
        Self {
            map: dashmap::DashMap::default(),
            max: Some(max),
        }
    }
    pub fn new_with_hasher(hasher: Hash) -> Self {
        Self {
            map: dashmap::DashMap::with_hasher(hasher),
            max: None,
        }
    }
    pub const fn set_max(&mut self, max: usize) {
        self.max = Some(max);
    }

    pub fn all(&self, mut f: impl FnMut(&Key, &Option<Result<Val, Err>>)) {
        for v in &self.map {
            let (k, v) = v.pair();
            v.with_value(|opt| f(k, opt));
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.map.clear();
    }

    #[inline]
    pub fn remove<Q: std::hash::Hash + Eq + ?Sized>(&self, key: &Q)
    where
        Key: std::borrow::Borrow<Q>,
    {
        self.map.remove(key);
    }

    pub fn retain(&self, mut keep: impl FnMut(&Key, &Result<Val, Err>) -> bool) {
        self.map
            .retain(|k, e| e.with_value(|opt| opt.as_ref().is_none_or(|v| keep(k, v))));
    }

    pub fn get<'t, Fut: Future<Output = Result<Val, Err>> + Send + 't, F: FnOnce(Key) -> Fut>(
        &self,
        key: Key,
        f: F,
    ) -> impl Future<Output = Result<Val, Err>> + Send + use<'t, Key, Val, Hash, Err, Fut, F> + 't
    where
        Err: 't,
        Val: 't,
    {
        use dashmap::Entry;
        // Evict before inserting so the newly inserted entry is not immediately removed.
        if self.max.is_some_and(|max| self.map.len() >= max) {
            self.retain(|_, _| false);
        }
        match self.map.entry(key) {
            Entry::Occupied(entry) => {
                let r = entry.get().clone();
                drop(entry);
                either::Left(r.get())
            }
            Entry::Vacant(entry) => {
                let (a, ret) = Awaitable::new_fut(f(entry.key().clone()));
                let r = entry.insert(a);
                drop(r);
                either::Right(ret.get())
            }
        }
    }

    /// Assumes f blocks
    /// # Errors
    pub fn get_sync(&self, key: Key, f: impl FnOnce(Key) -> Result<Val, Err>) -> Result<Val, Err> {
        use dashmap::Entry;
        // Evict before inserting so the newly inserted entry is not immediately removed.
        if self.max.is_some_and(|max| self.map.len() >= max) {
            self.retain(|_, _| false);
        }
        match self.map.entry(key) {
            Entry::Occupied(a) => {
                let awaitable = a.get().clone();
                drop(a);
                awaitable.get_sync()
            }
            Entry::Vacant(v) => {
                let key = v.key().clone();
                let (a, inner, sender) = Awaitable::new_sync();
                {
                    let r = v.insert(a);
                    drop(r);
                }
                let res = f(key);
                {
                    let (lock, cvar) = &*inner;
                    let mut guard = lock.lock().map_err(|_| CacheError)?;
                    *guard = Some(res.clone());
                    drop(guard);
                    cvar.notify_all();
                }

                // Broadcast to async waiters only when some actually exist — the common
                // case (no concurrent waiters) pays zero clones here.
                if sender.receiver_count() > 0 {
                    #[cfg(not(target_family = "wasm"))]
                    {
                        let _ = sender.broadcast_blocking(true);
                    } //.broadcast_direct(true).await;
                    #[cfg(target_family = "wasm")]
                    {
                        let _ = pollster::FutureExt::block_on(sender.broadcast_direct(true));
                    }
                }

                res
            }
        }
    }

    pub fn has<Q: std::hash::Hash + Eq + ?Sized>(
        &self,
        key: &Q,
    ) -> Option<impl Future<Output = Result<Val, Err>> + Send + use<Q, Key, Val, Err, Hash> + 'static>
    where
        Key: std::borrow::Borrow<Q>,
        Val: 'static,
    {
        self.map.get(key).map(|v| {
            let r = v.value().clone();
            drop(v);
            r.get()
        })
    }

    pub fn has_sync<Q: std::hash::Hash + Eq + ?Sized>(&self, key: &Q) -> Option<Result<Val, Err>>
    where
        Key: std::borrow::Borrow<Q>,
    {
        self.map.get(key).map(|v| {
            let r = v.value().clone();
            drop(v);
            r.get_sync()
        })
    }

    pub fn with<
        Fut: Future<Output = Result<Val, Err>> + Send + 'static,
        F: FnOnce(Key) -> Fut,
        R: Send + 'static,
        Then: FnOnce(&Val) -> R + Send + 'static,
    >(
        &self,
        key: &Key,
        f: F,
        then: Then,
    ) -> impl Future<Output = Result<R, Err>> + Send + use<Key, Val, Hash, Err, Fut, F, R, Then> + 'static
    where
        Err: 'static,
        Val: 'static,
        Key: Clone,
    {
        let val = self.map.get(key);
        if let Some(v) = val.as_ref() {
            let v = v.value();
            let inner = v.inner.0.lock();
            let Ok(guard) = inner else {
                drop(inner);
                return either::Left(either::Right(std::future::ready(std::result::Result::Err(
                    CacheError.into(),
                ))));
            };
            return if let Some(v) = &*guard {
                either::Left(either::Right(std::future::ready(v.as_ref().map_or_else(
                    |e| std::result::Result::Err(e.clone()),
                    |v| Ok(then(v)),
                ))))
            } else {
                drop(guard);
                let get = v.clone().get();
                either::Left(either::Left(async move { get.await.map(|r| then(&r)) }))
            };
        }
        drop(val);
        let get = self.get(key.clone(), f);
        either::Right(async move {
            let ret = get.await;
            ret.map(|r| then(&r))
        })
    }
}

use std::sync::{Arc, Condvar, Mutex};

type InnerAwaitable<V, E> = Arc<(Mutex<Option<Result<V, E>>>, Condvar)>;

#[derive(Clone, Debug)]
pub struct Awaitable<V: Clone + Send, E: Clone + From<CacheError> + Send> {
    // Shared state for sync waiters (Mutex + Condvar)
    pub inner: InnerAwaitable<V, E>,
    // Async waiters use a broadcast channel
    pub async_rx: async_broadcast::InactiveReceiver<bool>,
}

impl<T: Clone + Send + Sync, E: Clone + From<CacheError> + Send + Sync> Awaitable<T, E> {
    #[allow(clippy::manual_async_fn)]
    /// #### Errors
    pub fn get<'t>(self) -> impl std::future::Future<Output = Result<T, E>> + Send + 't
    where
        T: 't,
        E: 't,
    {
        async move {
            // Fast path: already done
            {
                let guard = self.inner.0.lock().map_err(|_| CacheError)?;
                if let Some(v) = guard.clone() {
                    return v;
                }
            }
            // Slow path: wait on the broadcast channel
            if !self
                .async_rx
                .activate()
                .recv_direct()
                .await
                .unwrap_or(false)
            {
                return Err(CacheError.into());
            }
            let guard = self.inner.0.lock().map_err(|_| CacheError)?;
            guard.clone().unwrap_or_else(|| Err(CacheError.into()))
        }
    }
    /// # Errors
    pub fn get_sync(self) -> Result<T, E> {
        let (lock, cvar) = &*self.inner;
        let guard = lock.lock().map_err(|_| CacheError)?;
        // Wait until the value is written. The condvar is always notified while
        // the mutex is held by the producer, so we cannot miss the transition.
        let guard = cvar
            .wait_while(guard, |v| v.is_none())
            .map_err(|_| CacheError)?;
        guard.clone().unwrap_or_else(|| Err(CacheError.into()))
    }

    pub fn new_fut<F: Future<Output = Result<T, E>> + Send>(
        future: F,
    ) -> (Self, AwaitableSource<T, E, F>) {
        let (mut async_tx, async_rx) = async_broadcast::broadcast(1);
        async_tx.set_await_active(false);
        let inner = Arc::new((Mutex::new(None), Condvar::new()));
        (
            Self {
                inner: inner.clone(),
                async_rx: async_rx.deactivate(),
            },
            AwaitableSource {
                inner,
                async_tx,
                future,
            },
        )
    }

    /// Create an `Awaitable` for use with the synchronous `get_sync` path.
    /// Returns the `Awaitable` (to store in the map) and the shared inner arc
    /// (so the caller can write the result and notify waiters).
    pub fn new_sync() -> (Self, InnerAwaitable<T, E>, async_broadcast::Sender<bool>) {
        let (mut async_tx, async_rx) = async_broadcast::broadcast(1);
        async_tx.set_await_active(false);
        let inner = Arc::new((Mutex::new(None::<Result<T, E>>), Condvar::new()));
        let awaitable = Self {
            inner: inner.clone(),
            async_rx: async_rx.deactivate(),
        };
        (awaitable, inner, async_tx)
    }
}
/*
#[derive(Clone, Debug)]
pub enum MaybeValue<T: Clone + Send, E: Clone + From<CacheError> + Send> {
    Done(Result<T, E>),
    Pending,
}
 */

pub struct AwaitableSource<
    T: Clone + Send,
    E: Clone + From<CacheError> + Send,
    F: Future<Output = Result<T, E>> + Send,
> {
    inner: InnerAwaitable<T, E>,
    async_tx: async_broadcast::Sender<bool>,
    future: F,
}

impl<
    T: Clone + Send + Sync,
    E: Clone + From<CacheError> + Send + Sync,
    F: Future<Output = Result<T, E>> + Send,
> AwaitableSource<T, E, F>
{
    /// # Errors
    pub async fn get(self) -> Result<T, E> {
        let Self {
            inner,
            async_tx,
            future,
        } = self;
        let res = future.await;
        // Write the result into the Mutex, notifying any sync waiters.
        // One unconditional clone here; sync waiters then clone out for themselves.
        {
            let (lock, cvar) = &*inner;
            let mut guard = lock.lock().map_err(|_| CacheError)?;
            *guard = Some(res.clone());
            drop(guard);
            cvar.notify_all();
        }
        // Broadcast to async waiters only when some actually exist — the common
        // case (no concurrent waiters) pays zero clones here.
        if async_tx.receiver_count() > 0 {
            let _ = async_tx.broadcast_direct(true).await;
        }
        res
    }
}

impl<T: Clone + Send, E: Clone + From<CacheError> + Send> Awaitable<T, E> {
    /// Call `f` with a reference to the current value without cloning anything.
    /// `None` means the entry is still pending; `Some(r)` means it is done.
    pub(crate) fn with_value<R>(&self, f: impl FnOnce(&Option<Result<T, E>>) -> R) -> R {
        match self.inner.0.lock() {
            Ok(guard) => f(&*guard),
            // Poisoned lock: treat as still-pending so retain keeps the entry.
            Err(_) => f(&None),
        }
    }
}

/*
#[cfg(feature = "deepsize")]
impl<T: Clone + Send + deepsize::DeepSizeOf, E: Clone + From<CacheError> + Send>
    deepsize::DeepSizeOf for MaybeValue<T, E>
{
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Done(Ok(v)) => v.deep_size_of_children(context),
            _ => 0,
        }
    }
}
 */

#[cfg(feature = "deepsize")]
impl<
    K: std::hash::Hash + Clone + Eq,
    T: Clone + Send + deepsize::DeepSizeOf,
    E: Clone + From<CacheError> + Send,
> deepsize::DeepSizeOf for AsyncCache<K, T, E>
{
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.map
            .iter()
            .map(|e| {
                std::mem::size_of::<K>() + std::mem::size_of_val(e.value()) + {
                    e.value().with_value(|opt| match opt {
                        Some(Ok(v)) =>
                        // urgh, what's the overhead of a dashmap bucket...? Let's just add 8
                        // bytes for good measure...
                        {
                            8 + v.deep_size_of_children(context)
                        }
                        _ => 0,
                    })
                }
            })
            .sum()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone, Debug, PartialEq)]
    struct TestErr(String);
    impl From<CacheError> for TestErr {
        fn from(_: CacheError) -> Self {
            Self("cache error".into())
        }
    }

    // -------------------------------------------------------------------------
    // Async tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_basic() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();
        let val = cache.get(1, |k| async move { Ok(k * 10) }).await.unwrap();
        assert_eq!(val, 10);
    }

    #[tokio::test]
    async fn test_get_cached() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let counter = counter.clone();
            let val = cache
                .get(42, move |k| async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(k * 2)
                })
                .await
                .unwrap();
            assert_eq!(val, 84);
        }

        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "factory must be called exactly once"
        );
    }

    #[tokio::test]
    async fn test_get_deduplication() {
        use tokio::sync::Barrier as TokioBarrier;

        let cache = Arc::new(AsyncCache::<u32, u32, TestErr>::default());
        let counter = Arc::new(AtomicUsize::new(0));
        // All tasks start together so they race to insert the same key.
        let barrier = Arc::new(TokioBarrier::new(5));

        let mut handles = Vec::new();
        for _ in 0..5 {
            let cache = cache.clone();
            let counter = counter.clone();
            let barrier = barrier.clone();
            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                cache
                    .get(7, move |k| {
                        async move {
                            counter.fetch_add(1, Ordering::SeqCst);
                            // Simulate a slow fetch so other tasks arrive while pending.
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            Ok(k + 1)
                        }
                    })
                    .await
            }));
        }

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.expect("task panicked"))
            .collect();

        for r in &results {
            assert_eq!(*r, Ok(8));
        }
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "factory must be called exactly once across all concurrent waiters"
        );
    }

    #[tokio::test]
    async fn test_get_error_cached() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let counter = counter.clone();
            let result = cache
                .get(99, move |_k| async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(TestErr("oops".into()))
                })
                .await;
            assert_eq!(result, Err(TestErr("oops".into())));
        }

        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "factory must be called exactly once even for errors"
        );
    }

    #[tokio::test]
    #[allow(clippy::cast_possible_truncation)]
    async fn test_get_max_eviction() {
        // Fill the cache to `max`, then request one more key.
        // The new entry must survive (Bug 1 regression test).
        const MAX: usize = 3;
        let mut cache = AsyncCache::<u32, u32, TestErr>::default();
        cache.set_max(MAX);

        // Fill to capacity.
        for i in 0..MAX as u32 {
            let v = cache.get(i, |k| async move { Ok(k) }).await.unwrap();
            assert_eq!(v, i);
        }
        assert_eq!(cache.map.len(), MAX);

        // Insert one more — this triggers eviction of old entries first.
        let new_key = MAX as u32;
        let val = cache
            .get(new_key, |k| async move { Ok(k * 10) })
            .await
            .unwrap();
        assert_eq!(
            val,
            new_key * 10,
            "newly inserted entry must return the correct value"
        );

        // The map must not have grown beyond max + 1 (one fresh entry after clearing).
        assert!(
            cache.map.len() <= MAX,
            "map length {} exceeds max {}",
            cache.map.len(),
            MAX
        );

        // The new key must be in the map and return the right value on a cache hit.
        let cached = cache.get(new_key, |_| async move { Ok(0) }).await.unwrap();
        assert_eq!(
            cached,
            new_key * 10,
            "new entry must be retrievable from cache"
        );
    }

    // -------------------------------------------------------------------------
    // Sync tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_sync_basic() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();
        let val = cache.get_sync(1, |k| Ok(k * 10)).unwrap();
        assert_eq!(val, 10);
    }

    #[tokio::test]
    async fn test_get_sync_cached() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let counter = counter.clone();
            let val = cache
                .get_sync(42, move |k| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(k * 2)
                })
                .unwrap();
            assert_eq!(val, 84);
        }

        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "factory must be called exactly once"
        );
    }

    #[tokio::test]
    #[allow(clippy::cast_possible_truncation)]
    async fn test_get_sync_max_eviction() {
        // Same regression test as test_get_max_eviction but for get_sync (Bug 2).
        const MAX: usize = 3;
        let mut cache = AsyncCache::<u32, u32, TestErr>::default();
        cache.set_max(MAX);

        for i in 0..MAX as u32 {
            let v = cache.get_sync(i, Ok).unwrap();
            assert_eq!(v, i);
        }
        assert_eq!(cache.map.len(), MAX);

        let new_key = MAX as u32;
        let val = cache.get_sync(new_key, |k| Ok(k * 10)).unwrap();
        assert_eq!(
            val,
            new_key * 10,
            "newly inserted sync entry must return correct value"
        );

        assert!(
            cache.map.len() <= MAX,
            "map length {} exceeds max {}",
            cache.map.len(),
            MAX
        );

        // Must be a cache hit now.
        let cached = cache.get_sync(new_key, |_| Ok(0)).unwrap();
        assert_eq!(
            cached,
            new_key * 10,
            "new entry must be retrievable from cache after get_sync"
        );
    }

    #[tokio::test]
    async fn test_get_sync_concurrent_waiters() {
        // One thread calls get_sync with a slow factory while another calls get_sync for
        // the same key. The second thread must receive the value via the broadcast channel
        // without invoking the factory a second time.
        let cache = Arc::new(AsyncCache::<u32, u32, TestErr>::default());
        let counter = Arc::new(AtomicUsize::new(0));

        // A barrier so both threads start at roughly the same time.
        let barrier = Arc::new(std::sync::Barrier::new(2));

        let cache1 = cache.clone();
        let counter1 = counter.clone();
        let barrier1 = barrier.clone();
        let t1 = std::thread::spawn(move || {
            barrier1.wait();
            cache1.get_sync(55, move |k| {
                counter1.fetch_add(1, Ordering::SeqCst);
                // Slow factory so t2 arrives while the entry is still Pending.
                std::thread::sleep(std::time::Duration::from_millis(100));
                Ok(k + 1)
            })
        });

        let cache2 = cache;
        let counter2 = counter.clone();
        let barrier2 = barrier;
        let t2 = std::thread::spawn(move || {
            barrier2.wait();
            // Small sleep so t1 wins the race to insert.
            std::thread::sleep(std::time::Duration::from_millis(10));
            cache2.get_sync(55, move |k| {
                counter2.fetch_add(1, Ordering::SeqCst);
                Ok(k + 100)
            })
        });

        let r1 = t1.join().expect("t1 panicked").unwrap();
        let r2 = t2.join().expect("t2 panicked").unwrap();

        assert_eq!(r1, 56);
        assert_eq!(r2, 56, "second thread must receive the broadcast value");
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "factory must only be called once"
        );
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();

        for i in 0..5u32 {
            cache.get(i, |k| async move { Ok(k) }).await.unwrap();
        }

        assert_eq!(cache.map.len(), 5);
        cache.clear();

        let mut visited = 0usize;
        cache.all(|_, _| {
            visited += 1;
        });
        assert_eq!(visited, 0, "all must visit no entries after clear");
        assert_eq!(cache.map.len(), 0);
    }

    #[tokio::test]
    async fn test_retain() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();

        // Insert Ok values 0..=9 and one Err.
        for i in 0..10u32 {
            cache.get(i, |k| async move { Ok(k) }).await.unwrap();
        }
        cache
            .get(100, |_| async move { Err(TestErr("bad".into())) })
            .await
            .unwrap_err();

        // Retain only Ok values where val >= 5.
        cache.retain(|_, v| matches!(v, Ok(n) if *n >= 5));

        let mut keys: Vec<u32> = Vec::new();
        cache.all(|_k, opt| {
            if let Some(Ok(v)) = opt {
                keys.push(*v);
            }
        });
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec![5, 6, 7, 8, 9],
            "retain must keep only Ok values >= 5"
        );
        // The Err entry (key=100) must have been removed since retain only keeps
        // Ok(_) entries matching the predicate, and errors don't match Ok(n) if n>=5.
        assert!(
            !cache.map.contains_key(&100),
            "error entry must have been evicted by retain"
        );
    }

    #[tokio::test]
    async fn test_all() {
        let cache = AsyncCache::<u32, u32, TestErr>::default();

        for i in 0..8u32 {
            cache.get(i, |k| async move { Ok(k * 3) }).await.unwrap();
        }

        let mut visited_keys: Vec<u32> = Vec::new();
        cache.all(|k, opt| {
            if let Some(Ok(_)) = opt {
                visited_keys.push(*k);
            }
        });
        visited_keys.sort_unstable();

        assert_eq!(
            visited_keys,
            (0..8u32).collect::<Vec<_>>(),
            "all must visit every key"
        );
    }
}
