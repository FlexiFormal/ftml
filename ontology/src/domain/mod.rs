use ftml_uris::{DomainUriRef, UriName};

use crate::{
    domain::{
        declarations::{AnyDeclarationRef, IsDeclaration},
        modules::Module,
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
impl Module {
    pub fn get_as<T: IsDeclaration>(&self, name: &UriName) -> Option<SharedDeclaration<T>> {
        SharedArc::opt_new(self, |m| &m.0, move |e| e.find(name.steps()).ok_or(()))
            .ok()
            .map(SharedDeclaration)
    }
}

pub trait HasDeclarations: crate::Ftml {
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator;
    fn domain_uri(&self) -> DomainUriRef<'_>;

    fn find<'s, T: IsDeclaration>(&self, steps: impl IntoIterator<Item = &'s str>) -> Option<&T> {
        #[allow(clippy::enum_glob_use)]
        use either_of::EitherOf5::*;
        let mut steps = steps.into_iter().peekable();
        let mut curr = A(self.declarations());
        'outer: while let Some(step) = steps.next() {
            macro_rules! ret {
                ($i:ident $e:expr;$m:expr) => {{
                    if steps.peek().is_none() {
                        return T::from_declaration($e);
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
                            T::from_declaration(c)
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
