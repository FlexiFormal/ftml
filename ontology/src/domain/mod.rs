use declarations::AnyDeclaration;
use ftml_uris::DomainUriRef;

use crate::domain::declarations::IsDeclaration;

pub mod declarations;
pub mod modules;

pub trait HasDeclarations: crate::Ftml {
    fn declarations(&self) -> &[AnyDeclaration];
    fn domain_uri(&self) -> DomainUriRef<'_>;
    fn find<'s, T: IsDeclaration>(&self, steps: impl IntoIterator<Item = &'s str>) -> Option<&T> {
        let mut steps = steps.into_iter().peekable();
        let mut curr = self.declarations().iter();
        macro_rules! ret {
            ($e:expr;$m:expr) => {{
                if steps.peek().is_none() {
                    return T::from_declaration($e.as_ref());
                }
                curr = $m.declarations().iter();
            }};
        }
        while let Some(step) = steps.next() {
            while let Some(c) = curr.next() {
                match c {
                    AnyDeclaration::NestedModule(m) if m.uri.name().last() == step => ret!(c;m),
                    AnyDeclaration::MathStructure(m) if m.uri.name().last() == step => ret!(c;m),
                    AnyDeclaration::Morphism(m) if m.uri.name().last() == step => ret!(c;m),
                    AnyDeclaration::Extension(m) if m.uri.name().last() == step => ret!(c;m),
                    AnyDeclaration::Symbol(s) if s.uri.name().last() == step => {
                        return if steps.peek().is_none() {
                            T::from_declaration(c.as_ref())
                        } else {
                            None
                        };
                    }
                    _ => (),
                }
            }
        }
        None
    }

    #[cfg(feature = "rdf")]
    fn declares_triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.domain_uri().to_iri();
        self.declarations().iter().filter_map(move |e| {
            e.uri()
                .map(|e| triple!(<(iri.clone())> ulo:declares <(e.to_iri())>))
        })
    }
}
