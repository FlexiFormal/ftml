use send_wrapper::SendWrapper;

pub struct FutWrap<F: Future>(send_wrapper::SendWrapper<F>);
impl<F: Future> std::ops::Deref for FutWrap<F> {
    type Target = F;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<F: Future> std::ops::DerefMut for FutWrap<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<F: Future> Future for FutWrap<F> {
    type Output = F::Output;
    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let inner: std::pin::Pin<&mut F> = unsafe { self.map_unchecked_mut(|s| &mut *s.0) };
        inner.poll(cx)
    }
}
impl<F: Future> FutWrap<F> {
    #[inline]
    pub fn new(f: F) -> Self {
        Self(SendWrapper::new(f))
    }
}
