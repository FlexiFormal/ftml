use ftml_uris::{DomainUriRef, ModuleUri, UriName};

use crate::{
    domain::{
        declarations::{
            AnyDeclarationRef, Declaration, IsDeclaration, SharedSymbolLike,
            structures::StructureDeclaration,
        },
        modules::{Module, ModuleLike},
    },
    utils::SharedArc,
};

pub mod declarations;
pub mod modules;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SharedDeclaration<T: IsDeclaration>(pub SharedArc<Module, T>);
impl<T: IsDeclaration> std::ops::Deref for SharedDeclaration<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: IsDeclaration> SharedDeclaration<T> {
    /// #### Safety
    /// requires that `other` is a "child" of this, i.e. the reference is guaranteed
    /// to be valid for the lifetime of `self`.
    pub unsafe fn inherit_unsafe<T2: IsDeclaration>(&self, other: &T2) -> SharedDeclaration<T2> {
        SharedDeclaration(unsafe { self.0.clone().inherit_unchecked(other) })
    }
}

impl Module {
    pub fn get_as<T: IsDeclaration>(&self, name: &UriName) -> Option<SharedDeclaration<T>> {
        SharedArc::opt_new(self, |m| &m.0, move |e| e.find(name.steps()).ok_or(()))
            .ok()
            .map(SharedDeclaration)
    }
    #[must_use]
    pub fn get_symbol_like(&self, name: &UriName) -> Option<SharedSymbolLike> {
        Some(match self.find_declaration(name.steps())? {
            AnyDeclarationRef::Symbol(s) => SharedSymbolLike::Symbol(SharedDeclaration(unsafe {
                SharedArc::new_unsafe(self.clone(), s)
            })),
            AnyDeclarationRef::MathStructure(ms) => {
                SharedSymbolLike::MathStructure(SharedDeclaration(unsafe {
                    SharedArc::new_unsafe(self.clone(), ms)
                }))
            }
            AnyDeclarationRef::Extension(e) => {
                SharedSymbolLike::Extension(SharedDeclaration(unsafe {
                    SharedArc::new_unsafe(self.clone(), e)
                }))
            }
            AnyDeclarationRef::Morphism(m) => {
                SharedSymbolLike::Morphism(SharedDeclaration(unsafe {
                    SharedArc::new_unsafe(self.clone(), m)
                }))
            }
            _ => return None,
        })
    }

    #[must_use]
    pub fn as_module_like(&self, name: &UriName) -> Option<ModuleLike> {
        Some(match self.find_declaration(name.steps())? {
            AnyDeclarationRef::NestedModule(nm) => ModuleLike::Nested(SharedDeclaration(unsafe {
                SharedArc::new_unsafe(self.clone(), nm)
            })),
            AnyDeclarationRef::MathStructure(ms) => {
                ModuleLike::Structure(SharedDeclaration(unsafe {
                    SharedArc::new_unsafe(self.clone(), ms)
                }))
            }
            AnyDeclarationRef::Extension(e) => ModuleLike::Extension(SharedDeclaration(unsafe {
                SharedArc::new_unsafe(self.clone(), e)
            })),
            AnyDeclarationRef::Morphism(m) => ModuleLike::Morphism(SharedDeclaration(unsafe {
                SharedArc::new_unsafe(self.clone(), m)
            })),
            _ => return None,
        })
    }
}

type DeclarationsIter<'a> = either::Either<
    std::iter::Map<std::slice::Iter<'a, Declaration>, fn(&'a Declaration) -> AnyDeclarationRef<'a>>,
    std::iter::Map<
        std::slice::Iter<'a, StructureDeclaration>,
        fn(&'a StructureDeclaration) -> AnyDeclarationRef<'a>,
    >,
>;
pub trait DeclIter<'a>:
    ExactSizeIterator<Item = AnyDeclarationRef<'a>> + DoubleEndedIterator
{
    fn either(self) -> DeclarationsIter<'a>;
}
impl<'a> DeclIter<'a>
    for std::iter::Map<
        std::slice::Iter<'a, Declaration>,
        fn(&'a Declaration) -> AnyDeclarationRef<'a>,
    >
{
    #[inline]
    fn either(self) -> DeclarationsIter<'a> {
        either::Left(self)
    }
}
impl<'a> DeclIter<'a>
    for std::iter::Map<
        std::slice::Iter<'a, StructureDeclaration>,
        fn(&'a StructureDeclaration) -> AnyDeclarationRef<'a>,
    >
{
    #[inline]
    fn either(self) -> DeclarationsIter<'a> {
        either::Right(self)
    }
}
impl<'a> DeclIter<'a> for DeclarationsIter<'a> {
    #[inline]
    fn either(self) -> Self {
        self
    }
}

pub trait HasDeclarations: crate::Ftml + Sync {
    type DeclIter<'a>: DeclIter<'a>
    where
        Self: 'a;
    fn declarations(&self) -> Self::DeclIter<'_>;
    fn domain_uri(&self) -> DomainUriRef<'_>;

    /// #### Errors
    /// ...if get errors
    fn initialize(
        &self,
        get: &mut dyn FnMut(&ModuleUri) -> Option<ModuleLike>,
    ) -> Result<(), ModuleUri> {
        for d in self.declarations() {
            match d {
                AnyDeclarationRef::Extension(e) => e.initialize(get)?,
                AnyDeclarationRef::MathStructure(e) => e.initialize(get)?,
                AnyDeclarationRef::Morphism(e) => e.initialize(get)?,
                AnyDeclarationRef::NestedModule(e) => e.initialize(get)?,
                AnyDeclarationRef::Import { .. }
                | AnyDeclarationRef::Symbol(_)
                | AnyDeclarationRef::Rule { .. } => (),
            }
        }
        Ok(())
    }
    /*
    fn initialize_async<E: std::fmt::Display, F>(
        &self,
        get: &mut dyn FnMut(&ModuleUri) -> F,
    ) -> impl std::future::Future<Output = ()>
    where
        F: Future<Output = Result<ModuleLike, E>> + Send,
    {
        async {
            for d in self.declarations() {
                match d {
                    AnyDeclarationRef::Extension(e) => {
                        (Box::pin(e.initialize_async(get))
                            as std::pin::Pin<Box<dyn Future<Output = _>>>)
                            .await;
                    }
                    AnyDeclarationRef::MathStructure(e) => {
                        (Box::pin(e.initialize_async(get))
                            as std::pin::Pin<Box<dyn Future<Output = _>>>)
                            .await;
                    }
                    AnyDeclarationRef::Morphism(e) => {
                        (Box::pin(e.initialize_async(get))
                            as std::pin::Pin<Box<dyn Future<Output = _>>>)
                            .await;
                    }
                    AnyDeclarationRef::NestedModule(e) => {
                        (Box::pin(e.initialize_async(get))
                            as std::pin::Pin<Box<dyn Future<Output = _>>>)
                            .await;
                    }
                    AnyDeclarationRef::Import { .. }
                    | AnyDeclarationRef::Symbol(_)
                    | AnyDeclarationRef::Rule { .. } => (),
                }
            }
        }
    }
     */

    fn find_declaration<'s>(
        &self,
        steps: impl IntoIterator<Item = &'s str>,
    ) -> Option<AnyDeclarationRef<'_>> {
        fn get<'d>(
            step: &str,
            d: AnyDeclarationRef<'d>,
        ) -> (Option<AnyDeclarationRef<'d>>, Option<DeclarationsIter<'d>>) {
            match d {
                AnyDeclarationRef::NestedModule(m) if m.uri.name().last() == step => {
                    (Some(d), Some(m.declarations().either()))
                }
                AnyDeclarationRef::MathStructure(m) if m.uri.name().last() == step => {
                    (Some(d), Some(m.declarations().either()))
                }
                AnyDeclarationRef::Morphism(m) if m.uri.name().last() == step => {
                    (Some(d), Some(m.declarations().either()))
                }
                AnyDeclarationRef::Morphism(m) => {
                    for d in m.declarations() {
                        if d.uri().is_some_and(|uri| {
                            !uri.name().as_ref().starts_with(m.uri.name().as_ref())
                        }) {
                            let r = get(step, d);
                            if r.0.is_some() {
                                return r;
                            }
                        }
                    }
                    (None, None)
                }
                AnyDeclarationRef::Extension(m) if m.uri.name().last() == step => {
                    (Some(d), Some(m.declarations().either()))
                }
                AnyDeclarationRef::Symbol(s) if s.uri.name().last() == step => (Some(d), None),
                _ => (None, None),
            }
        }
        let mut steps = steps.into_iter().peekable();
        let mut curr: DeclarationsIter = self.declarations().either();
        'outer: while let Some(step) = steps.next() {
            while let Some(c) = curr.next() {
                match get(step, c) {
                    (Some(d), _) if steps.peek().is_none() => return Some(d),
                    (Some(_), Some(e)) => {
                        curr = e;
                        continue 'outer;
                    }
                    (Some(_), None) => return None,
                    (None, _) => (),
                }
            }
            return None;
        }
        None
    }

    fn find<'s, T: IsDeclaration>(&self, steps: impl IntoIterator<Item = &'s str>) -> Option<&T> {
        self.find_declaration(steps).and_then(T::from_declaration)
    }

    #[cfg(feature = "rdf")]
    fn declares_triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.domain_uri().to_iri();
        self.declarations().filter_map(move |e| {
            e.uri()
                .map(|e| triple!(<(iri.clone())> ulo:declares <(e.to_iri())>))
        })
    }
}
