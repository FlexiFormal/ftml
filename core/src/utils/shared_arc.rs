/// A [`SharedArc`] models the situation where:
/// - an `o:Outer` holds an <code>[Arc](triomphe::Arc)&lt;Arced&gt;</code>
/// - `Arced` holds a (potentially expensive to obtain) reference `i:&Inner`.
/// - We have the `o:Outer` around, but are only, or mostly, interested in the `i:&Inner`.
///
/// In that case, we could, in principle, safely pass around the pair `(o,i)`, since by *holding
/// on to* `o`, we guarantee that the reference target of `i` cannot move or get dropped, since it is
/// behind the [`Arc`](triomphe::Arc), an instance of which is owned by `o`.
///
/// [`SharedArc`] conceptually is such a pair `(o,i)` which dereferences to `Inner`.
pub struct SharedArc<Outer, Inner> {
    outer: Outer,
    elem: *const Inner,
}
impl<Outer, Inner> SharedArc<Outer, Inner> {
    /// Fallibly construct a new [`SharedArc`]. The `outer`,
    /// in the simplest case, is just an <code>[Arc](triomphe::Arc)&lt;Arced&gt;</code>, or a newtype Wrapper around one.
    ///
    /// `arc` is used to get the actual <code>[Arc](triomphe::Arc)&lt;Arced&gt;</code>. By assumption,
    /// the <code>[Arc](triomphe::Arc)&lt;Arced&gt;</code> should be owned by `outer`, so an `fn` should perfectly suffice.
    ///
    /// `get` is used to get at the inner. Again, by assumption, the `Inner` should be owned
    /// by the `Arced`, and thus be behind the same reference counter, so an `fn` should perfectly suffice.
    ///
    /// The core assumption behind a [`SharedArc`] is then, that, subsequently, as long
    /// as the <code>[Arc](triomphe::Arc)&lt;Arced&gt;</code> lives, the reference to the `Inner` is valid.
    ///
    /// ## Errors
    /// iff `get` errors.
    pub fn new<Arced, Err>(
        outer: Outer,
        arc: fn(&Outer) -> &triomphe::Arc<Arced>,
        get: fn(&Arced) -> Result<&Inner, Err>,
    ) -> Result<Self, Err> {
        let elem = get(arc(&outer))?;
        let elem = elem as *const Inner;
        Ok(Self { outer, elem })
    }

    /// If a reference to an `Inner` allows to get at a `NewInner`, then we can safely turn this
    /// `SharedArc<Outer,Inner>` into a `SharedArc<Outer,NewInner>`.
    ///
    /// ## Errors
    /// iff `get` errors. In that case, we also return the original `self`.
    pub fn inherit<NewInner, Err>(
        self,
        get: fn(&Inner) -> Result<&NewInner, Err>,
    ) -> Result<SharedArc<Outer, NewInner>, (Self, Err)> {
        let elem = match get(&*self) {
            Ok(e) => e as *const NewInner,
            Err(e) => return Err((self, e)),
        };
        Ok(SharedArc {
            outer: self.outer,
            elem,
        })
    }
    /*
    pub fn new_from_outer<Err>(
        outer: Outer,
        get: fn(&Outer) -> Result<&Inner, Err>,
    ) -> Result<Self, Err> {
        let elem = get(&outer)?;
        let elem = elem as *const Inner;
        Ok(Self { outer, elem })
    }
     */

    #[inline]
    /// Get a reference to the `Outer` held by this
    pub const fn outer(&self) -> &Outer {
        &self.outer
    }
    #[inline]
    /// Get a reference to the `Outer` held by this
    pub fn into_outer(self) -> Outer {
        self.outer
    }
}

impl<Outer, Inner> AsRef<Outer> for SharedArc<Outer, Inner> {
    #[inline]
    fn as_ref(&self) -> &Outer {
        self.outer()
    }
}
impl<Outer, Inner> std::ops::Deref for SharedArc<Outer, Inner> {
    type Target = Inner;
    #[inline]
    fn deref(&self) -> &Inner {
        // safe, because data holds an Arc to the Outer this comes from,
        // and no inner mutability is employed that might move the
        // element around, by contract of unsafe Self::new.
        unsafe { &*self.elem } //.as_ref_unchecked() }
    }
}
unsafe impl<Outer: Send, Inner> Send for SharedArc<Outer, Inner> {}
unsafe impl<Outer: Sync, Inner> Sync for SharedArc<Outer, Inner> {}
