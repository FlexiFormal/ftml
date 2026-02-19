use ftml_uris::{DomainUriRef, ModuleUri, UriName};

use crate::{
    domain::{
        declarations::{AnyDeclarationRef, IsDeclaration, symbols::ArgumentSpec},
        modules::{Module, ModuleLike},
    },
    utils::{SharedArc, TreeIter},
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
impl Module {
    pub fn get_as<T: IsDeclaration>(&self, name: &UriName) -> Option<SharedDeclaration<T>> {
        SharedArc::opt_new(self, |m| &m.0, move |e| e.find(name.steps()).ok_or(()))
            .ok()
            .map(SharedDeclaration)
    }

    #[must_use]
    pub fn as_module_like(&self, name: &UriName) -> Option<ModuleLike> {
        // meh: this needs to find() twice
        Some(match self.find_declaration(name.steps())? {
            AnyDeclarationRef::NestedModule(_) => ModuleLike::Nested(SharedDeclaration(unsafe {
                SharedArc::new(self.clone(), |m| &m.0, |e| e.find(name.steps()).ok_or(()))
                    .unwrap_unchecked()
            })),
            AnyDeclarationRef::MathStructure(_) => {
                ModuleLike::Structure(SharedDeclaration(unsafe {
                    SharedArc::new(self.clone(), |m| &m.0, |e| e.find(name.steps()).ok_or(()))
                        .unwrap_unchecked()
                }))
            }
            AnyDeclarationRef::Extension(_) => ModuleLike::Extension(SharedDeclaration(unsafe {
                SharedArc::new(self.clone(), |m| &m.0, |e| e.find(name.steps()).ok_or(()))
                    .unwrap_unchecked()
            })),
            AnyDeclarationRef::Morphism(_) => ModuleLike::Morphism(SharedDeclaration(unsafe {
                SharedArc::new(self.clone(), |m| &m.0, |e| e.find(name.steps()).ok_or(()))
                    .unwrap_unchecked()
            })),
            _ => return None,
        })
    }
}

pub trait HasDeclarations: crate::Ftml {
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator;
    fn domain_uri(&self) -> DomainUriRef<'_>;

    fn initialize<E: std::fmt::Display>(
        &self,
        get: &mut dyn FnMut(&ModuleUri) -> Result<ModuleLike, E>,
    ) {
        for d in self.declarations() {
            match d {
                AnyDeclarationRef::Extension(e) => e.initialize(get),
                AnyDeclarationRef::MathStructure(e) => e.initialize(get),
                AnyDeclarationRef::Morphism(e) => e.initialize(get),
                AnyDeclarationRef::NestedModule(e) => e.initialize(get),
                AnyDeclarationRef::Import(_) | AnyDeclarationRef::Symbol(_) => (),
            }
        }
    }

    async fn initialize_async<E: std::fmt::Display, F>(&self, get: &mut dyn FnMut(&ModuleUri) -> F)
    where
        F: Future<Output = Result<ModuleLike, E>>,
    {
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
                AnyDeclarationRef::Import(_) | AnyDeclarationRef::Symbol(_) => (),
            }
        }
    }

    fn find_declaration<'s>(
        &self,
        steps: impl IntoIterator<Item = &'s str>,
    ) -> Option<AnyDeclarationRef<'_>> {
        use either_of::EitherOf5::{A, B, C, D, E};
        let mut steps = steps.into_iter().peekable();
        let mut curr = A(self.declarations());
        'outer: while let Some(step) = steps.next() {
            macro_rules! ret {
                ($i:ident $e:expr;$m:expr) => {{
                    if steps.peek().is_none() {
                        return Some($e);
                    }
                    curr = $i($m.declarations());
                    continue 'outer;
                }};
            }
            while let Some(c) = curr.next() {
                match c {
                    AnyDeclarationRef::NestedModule(m) if m.uri.name().last() == step => {
                        ret!(B c;m);
                    }
                    AnyDeclarationRef::MathStructure(m) if m.uri.name().last() == step => {
                        ret!(C c;m);
                    }
                    AnyDeclarationRef::Morphism(m) if m.uri.name().last() == step => ret!(D c;m),
                    AnyDeclarationRef::Extension(m) if m.uri.name().last() == step => ret!(E c;m),
                    AnyDeclarationRef::Symbol(s) if s.uri.name().last() == step => {
                        return if steps.peek().is_none() {
                            Some(c)
                        } else {
                            None
                        };
                    }
                    _ => (),
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
