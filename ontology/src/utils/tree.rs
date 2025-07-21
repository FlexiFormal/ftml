use std::{fmt::Write, marker::PhantomData, ops::ControlFlow};

pub trait TreeChild<'a>: Copy {
    fn tree_children(self) -> impl Iterator<Item = Self>;
}

impl<'a, T: RefTree<Child<'a> = &'a T> + 'a> TreeChild<'a> for &'a T {
    #[inline]
    fn tree_children(self) -> impl Iterator<Item = Self> {
        T::tree_children(self)
    }
}

const STACK: usize = 4;

pub trait TreeIter<'a>: IntoIterator
where
    Self::Item: TreeChild<'a>,
{
    #[inline]
    fn dfs(self) -> impl Iterator<Item = Self::Item>
    where
        Self: Sized,
    {
        self.into_iter().flat_map(|c: Self::Item| {
            std::iter::once(c).chain(RefIter::<_, _, true> {
                get: |e: Self::Item| e.tree_children(),
                current: c.tree_children(),
                stack: smallvec::SmallVec::new(),
                _phantom: PhantomData,
            })
        })
    }

    #[inline]
    fn bfs(self) -> impl Iterator<Item = Self::Item>
    where
        Self: Sized,
    {
        enum Itr<
            'b,
            It: TreeChild<'b>,
            I: Iterator<Item = It>,
            J: Iterator<Item = It>,
            K: Iterator<Item = It>,
        > {
            First {
                iter: I,
                get: fn(It) -> J,
                _phantom: PhantomData<&'b ()>,
                stack: smallvec::SmallVec<J, STACK>,
                then: fn(smallvec::SmallVec<J, STACK>) -> Option<K>,
            },
            Second(K),
        }
        impl<
            'b,
            It: TreeChild<'b>,
            I: Iterator<Item = It>,
            J: Iterator<Item = It>,
            K: Iterator<Item = It>,
        > Iterator for Itr<'b, It, I, J, K>
        {
            type Item = It;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::First {
                        iter,
                        stack,
                        then,
                        get,
                        ..
                    } => {
                        if let Some(n) = iter.next() {
                            stack.push(get(n));
                            Some(n)
                        } else {
                            let ch = std::mem::take(stack);
                            then(ch).and_then(|j| {
                                *self = Self::Second(j);
                                self.next()
                            })
                        }
                    }
                    Self::Second(i) => i.next(),
                }
            }
        }
        Itr::First {
            iter: self.into_iter(),
            _phantom: PhantomData,
            stack: smallvec::SmallVec::new(),
            get: |c| c.tree_children(),
            then: |mut s| {
                if s.is_empty() {
                    return None;
                }
                let first = s.remove(0);
                Some(RefIter::<_, _, false> {
                    get: |e: Self::Item| e.tree_children(),
                    current: first,
                    stack: s,
                    _phantom: PhantomData,
                })
            },
        }
    }

    fn dfs_with_state<R, S, M>(
        self,
        initial: S,
        mut state: M,
        mut f: impl FnMut(Self::Item, &mut S, &mut M) -> IterCont<R, S>,
        mut then: impl FnMut(Self::Item, S, &mut M) -> ControlFlow<R>,
    ) -> Option<R>
    where
        Self: Sized,
    {
        let mut current = either::Either::Left(self.into_iter());
        let mut inner_state = initial;
        let mut stack = smallvec::SmallVec::<_, STACK>::new();
        loop {
            let Some(n) = current.next() else {
                let (e, iter, mut s) = stack.pop()?;
                current = iter;
                std::mem::swap(&mut inner_state, &mut s);
                match then(e, s, &mut state) {
                    ControlFlow::Break(r) => return Some(r),
                    ControlFlow::Continue(()) => continue,
                }
            };
            match f(n, &mut inner_state, &mut state) {
                IterCont::Break(r) => return Some(r),
                IterCont::Skip => (),
                IterCont::Recurse(s) => {
                    stack.push((
                        n,
                        std::mem::replace(&mut current, either::Either::Right(n.tree_children())),
                        std::mem::replace(&mut inner_state, s),
                    ));
                }
            }
        }
    }
}
impl<'a, T: IntoIterator> TreeIter<'a> for T where T::Item: TreeChild<'a> {}

pub trait RefTree {
    type Child<'a>: TreeChild<'a>
    where
        Self: 'a;

    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>>;

    #[inline]
    fn dfs(&self) -> impl Iterator<Item = Self::Child<'_>>
    where
        Self: Sized,
    {
        self.tree_children().dfs()
    }

    #[inline]
    fn bfs(&self) -> impl Iterator<Item = Self::Child<'_>>
    where
        Self: Sized,
    {
        self.tree_children().bfs()
    }

    #[inline]
    fn dfs_with_state<R, S, M>(
        &self,
        initial: S,
        state: M,
        f: impl FnMut(Self::Child<'_>, &mut S, &mut M) -> IterCont<R, S>,
        then: impl FnMut(Self::Child<'_>, S, &mut M) -> ControlFlow<R>,
    ) -> Option<R> {
        self.tree_children().dfs_with_state(initial, state, f, then)
    }
}

impl<T> RefTree for T
where
    for<'b> T: TreeChild<'b>,
{
    type Child<'a>
        = Self
    where
        Self: 'a;
    #[inline]
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        T::tree_children(*self)
    }
}

pub enum IterCont<R, S> {
    Recurse(S),
    Skip,
    Break(R),
}

struct RefIter<'i, T: TreeChild<'i>, I: Iterator<Item = T>, const DFS: bool> {
    get: fn(T) -> I,
    current: I,
    stack: smallvec::SmallVec<I, STACK>,
    _phantom: PhantomData<&'i ()>,
}
impl<'i, T: TreeChild<'i>, I: Iterator<Item = T>, const DFS: bool> Iterator
    for RefIter<'i, T, I, DFS>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(n) = self.current.next() else {
            return self.stack.pop().and_then(|s| {
                self.current = s;
                self.next()
            });
        };
        if DFS {
            self.stack
                .push(std::mem::replace(&mut self.current, (self.get)(n)));
        } else {
            self.stack.push((self.get)(n));
        }
        Some(n)
    }
}

#[derive(Copy, Clone)]
pub struct Indentation<'s> {
    pub with: &'s str,
    pub times: u8,
}
impl std::fmt::Display for Indentation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.times {
            f.write_str(self.with)?;
        }
        Ok(())
    }
}

pub trait DisplayIndented: RefTree {
    /// # Errors
    /// if the underlying [`Formatter`](std::fmt::Formatter) errors.
    fn open_line(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<IterCont<(), ()>, std::fmt::Error>;
    /// # Errors
    /// if the underlying [`Formatter`](std::fmt::Formatter) errors.
    fn close_line(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
impl<'a, T: DisplayIndented + RefTree<Child<'a> = &'a T>> DisplayIndentedChild<'a> for &'a T {
    #[inline]
    fn open_line(
        self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<IterCont<(), ()>, std::fmt::Error> {
        T::open_line(self, f)
    }

    #[inline]
    fn close_line(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::close_line(self, f)
    }
}

pub trait DisplayIndentedChild<'a>: TreeChild<'a> {
    /// # Errors
    /// if the underlying [`Formatter`](std::fmt::Formatter) errors.
    fn open_line(
        self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<IterCont<(), ()>, std::fmt::Error>;

    /// # Errors
    /// if the underlying [`Formatter`](std::fmt::Formatter) errors.
    fn close_line(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn indented(self, with: &'a str) -> impl std::fmt::Display + use<'a, Self>
    where
        Self: Sized,
    {
        Indented {
            slf: self,
            indent: with,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Indented<'a, T: DisplayIndentedChild<'a>> {
    slf: T,
    indent: &'a str,
}

impl<'a, T: DisplayIndentedChild<'a>> Indented<'a, T> {
    fn open(
        e: T,
        curr: Indentation<'a>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<IterCont<std::fmt::Error, Indentation<'a>>, std::fmt::Error> {
        write!(f, "{curr}")?;
        let r = e.open_line(f)?;
        f.write_char('\n')?;
        Ok(match r {
            IterCont::Recurse(()) => IterCont::Recurse(Indentation {
                with: curr.with,
                times: curr.times + 1,
            }),
            IterCont::Skip => IterCont::Skip,
            IterCont::Break(()) => return Err(std::fmt::Error),
        })
    }
    fn close(
        e: T,
        mut curr: Indentation<'a>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        curr.times -= 1;
        write!(f, "{curr}")?;
        e.close_line(f)?;
        f.write_char('\n')?;
        Ok(())
    }
}

impl<'a, T: DisplayIndentedChild<'a>> std::fmt::Display for Indented<'a, T> {
    fn fmt(&self, mut f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut curr = Indentation {
            with: self.indent,
            times: 0,
        };
        self.slf.open_line(f)?;
        f.write_char('\n')?;
        curr.times += 1;
        if let Some(r) = self.slf.tree_children().dfs_with_state(
            curr,
            &mut f,
            |e, curr, f| match Self::open(e, *curr, f) {
                Err(e) => IterCont::Break(e),
                Ok(c) => c,
            },
            |e, curr, f| match Self::close(e, curr, f) {
                Err(e) => ControlFlow::Break(e),
                Ok(()) => ControlFlow::Continue(()),
            },
        ) {
            return Err(r);
        }
        self.slf.close_line(f)
    }
}

#[cfg(test)]
mod indent_test {
    use std::fmt::Write;

    use super::*;

    #[derive(Debug)]
    enum Test {
        A(A),
        B(B),
        C(C),
    }
    #[derive(Debug)]
    struct A {
        children: &'static [Test],
        head: &'static str,
    }
    #[derive(Debug)]
    struct B {
        children: &'static [Test],
        head: &'static str,
    }
    #[derive(Debug)]
    struct C {
        children: &'static [Test],
        head: &'static str,
    }
    impl Test {
        /*fn head(&self) -> &'static str {
            match self {
                Self::A(e) => e.head,
                Self::B(e) => e.head,
                Self::C(e) => e.head,
            }
        }*/
        fn children(&self) -> &'static [Self] {
            match self {
                Self::A(e) => e.children,
                Self::B(e) => e.children,
                Self::C(e) => e.children,
            }
        }
    }

    impl RefTree for Test {
        type Child<'a> = &'a Self;
        //type Iter<'a> = std::slice::Iter<'a, Self>;
        fn tree_children(&self) -> impl Iterator<Item = &Self> {
            // Self::Iter<'_> {
            self.children().iter()
        }
    }

    impl DisplayIndented for Test {
        fn open_line(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> Result<IterCont<(), ()>, std::fmt::Error> {
            let ch = match self {
                Self::A(a) => {
                    write!(f, "A: {}", a.head)?;
                    a.children
                }
                Self::B(a) => {
                    write!(f, "B: {}", a.head)?;
                    a.children
                }
                Self::C(a) => {
                    write!(f, "C: {}", a.head)?;
                    a.children
                }
            };
            Ok(if ch.is_empty() {
                IterCont::Skip
            } else {
                f.write_char(' ')?;
                f.write_char('{')?;
                IterCont::Recurse(())
            })
        }
        fn close_line(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char('}')?;
            Ok(())
        }
    }

    const TST: Test = Test::A(A {
        head: "Hello",
        children: &[
            Test::B(B {
                head: "I'm a B",
                children: &[],
            }),
            Test::C(C {
                head: "I'm a C",
                children: &[
                    Test::A(A {
                        head: "I'm another A",
                        children: &[
                            Test::B(B {
                                head: "I'm another B",
                                children: &[],
                            }),
                            Test::C(C {
                                head: "I'm another C",
                                children: &[],
                            }),
                        ],
                    }),
                    Test::B(B {
                        head: "I'm yet another B",
                        children: &[],
                    }),
                ],
            }),
        ],
    });

    const AS_STR: &str = r"A: Hello {
  B: I'm a B
  C: I'm a C {
    A: I'm another A {
      B: I'm another B
      C: I'm another C
    }
    B: I'm yet another B
  }
}";

    #[test]
    fn indent_display() {
        //tracing_subscriber::fmt().init();
        //tracing::info!("\n{}", TST.indented("  "));
        assert_eq!(TST.indented("  ").to_string(), AS_STR);
    }
}

/*
pub trait MutTree {
    type Iter<'a>: Iterator<Item = &'a mut Self>
    where
        Self: 'a;
    fn children_mut(&mut self) -> Self::Iter<'_>;
}

pub struct MutIter<'i, T: MutTree + 'i, const DFS: bool> {
    current: T::Iter<'i>,
    stack: smallvec::SmallVec<T::Iter<'i>, 1>,
}
impl<'i, T: MutTree + 'i, const DFS: bool> Iterator for MutIter<'i, T, DFS> {
    type Item = &'i mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let n = match self.current.next() {
            Some(n) => n,
            None => match self.stack.pop() {
                Some(s) => {
                    self.current = s;
                    return self.next();
                }
                None => return None,
            },
        };
        if DFS {
            self.stack
                .push(std::mem::replace(&mut self.current, n.children_mut()));
        } else {
            self.stack.push(n.children_mut());
        }
        Some(n)
    }
}
 */
