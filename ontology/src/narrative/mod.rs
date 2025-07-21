pub mod documents;
pub mod elements;

use ftml_uris::{DocumentUri, NarrativeUriRef};
use smallvec::SmallVec;
use std::marker::PhantomData;

use crate::narrative::elements::DocumentElement;

pub trait Narrative: crate::Ftml {
    fn narrative_uri(&self) -> Option<NarrativeUriRef<'_>>;
    fn children(&self) -> &[DocumentElement];

    #[cfg(feature = "rdf")]
    #[deprecated(note = "inputref etc missing")]
    fn contains_triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let Some(iri) = self.narrative_uri().map(NarrativeUriRef::to_iri) else {
            return either::Either::Left(std::iter::empty());
        };
        either::Either::Right(
            self.children()
                .iter()
                .flat_map(|e| {
                    let ch = e.opaque_children();
                    if ch.is_empty() {
                        either::Either::Left(std::iter::once(e))
                    } else {
                        either::Either::Right(ch.iter())
                    }
                })
                .filter_map(move |e| {
                    e.element_uri()
                        .map(|e| triple!(<(iri.clone())> ulo:contains <(e.to_iri())>))
                }),
        )
    }

    #[allow(clippy::too_many_lines)]
    fn find<'s, T: elements::IsDocumentElement>(
        &self,
        steps: impl IntoIterator<Item = &'s str>,
    ) -> Option<&T> {
        enum I<'a> {
            One(std::slice::Iter<'a, DocumentElement>),
            Mul(
                std::slice::Iter<'a, DocumentElement>,
                SmallVec<std::slice::Iter<'a, DocumentElement>, 2>,
            ),
        }
        impl<'a> I<'a> {
            fn push(&mut self, es: &'a [DocumentElement]) {
                match self {
                    Self::One(_) => {
                        let new = Self::Mul(es.iter(), SmallVec::with_capacity(1));
                        let Self::One(s) = std::mem::replace(self, new) else {
                            unreachable!()
                        };
                        let Self::Mul(_, v) = self else {
                            unreachable!()
                        };
                        v.push(s);
                    }
                    Self::Mul(f, r) => {
                        let of = std::mem::replace(f, es.iter());
                        r.push(of);
                    }
                }
            }
        }
        impl<'a> Iterator for I<'a> {
            type Item = &'a DocumentElement;
            #[allow(clippy::option_if_let_else)]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::One(s) => s.next(),
                    Self::Mul(f, r) => loop {
                        if let Some(n) = f.next() {
                            return Some(n);
                        }
                        let Some(mut n) = r.pop() else { unreachable!() };
                        if r.is_empty() {
                            let r = n.next();
                            *self = Self::One(n);
                            return r;
                        }
                        *f = n;
                    },
                }
            }
        }
        let mut steps = steps.into_iter().peekable();
        let mut curr = I::One(self.children().iter());
        'outer: while let Some(step) = steps.next() {
            while let Some(c) = curr.next() {
                match c {
                    DocumentElement::Section(elements::Section { uri, children, .. })
                    | DocumentElement::Paragraph(elements::LogicalParagraph {
                        uri,
                        children,
                        ..
                    })
                    | DocumentElement::Problem(elements::Problem { uri, children, .. })
                        if uri.name().last() == step =>
                    {
                        if steps.peek().is_none() {
                            return T::from_element(c.as_ref());
                        }
                        curr = I::One(children.iter());
                        continue 'outer;
                    }
                    DocumentElement::Slide { uri, .. }
                        if uri.name().last() == step && steps.peek().is_none() =>
                    {
                        return T::from_element(c.as_ref());
                    }
                    DocumentElement::Module { children, .. }
                    | DocumentElement::Morphism { children, .. }
                    | DocumentElement::MathStructure { children, .. }
                    | DocumentElement::Slide { children, .. }
                    | DocumentElement::Extension { children, .. } => curr.push(children),
                    DocumentElement::Notation { uri, .. }
                    | DocumentElement::VariableNotation { uri, .. }
                    | DocumentElement::VariableDeclaration(elements::VariableDeclaration {
                        uri,
                        ..
                    })
                    | DocumentElement::Expr { uri, .. }
                        if uri.name().last() == step =>
                    {
                        if steps.peek().is_none() {
                            return T::from_element(c.as_ref());
                        }
                        return None;
                    }
                    DocumentElement::Section(_)
                    | DocumentElement::Paragraph(_)
                    | DocumentElement::Problem(_)
                    | DocumentElement::SetSectionLevel(_)
                    | DocumentElement::SymbolDeclaration(_)
                    | DocumentElement::UseModule(_)
                    | DocumentElement::ImportModule(_)
                    | DocumentElement::SkipSection(_)
                    | DocumentElement::VariableDeclaration(_)
                    | DocumentElement::Definiendum { .. }
                    | DocumentElement::SymbolReference { .. }
                    | DocumentElement::VariableReference { .. }
                    | DocumentElement::DocumentReference { .. }
                    | DocumentElement::Notation { .. }
                    | DocumentElement::VariableNotation { .. }
                    | DocumentElement::Expr { .. } => (),
                }
            }
        }
        None
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocDataRef<T> {
    pub start: usize,
    pub end: usize,
    pub in_doc: DocumentUri,
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom_data: PhantomData<T>,
}

#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DataRef<T> {
    pub start: usize,
    pub end: usize,
    phantom_data: PhantomData<T>,
}
impl<T> std::fmt::Debug for DataRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataRef")
            .field("type", &std::any::type_name::<T>())
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}
impl<T> Clone for DataRef<T> {
    #[inline]
    #[allow(clippy::non_canonical_clone_impl)]
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            phantom_data: self.phantom_data,
        }
    }
}
impl<T> Copy for DataRef<T> {}
impl<T> PartialEq for DataRef<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl<T> Eq for DataRef<T> {}
impl<T> std::hash::Hash for DataRef<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}
#[cfg(feature = "serde")]
mod serde_impl {
    use std::marker::PhantomData;

    use serde::ser::SerializeStruct;

    impl<T> serde::Serialize for super::DataRef<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_struct("DataRef", 2)?;
            s.serialize_field("start", &self.start)?;
            s.serialize_field("end", &self.end)?;
            s.end()
        }
    }
    impl<'de, T> serde::Deserialize<'de> for super::DataRef<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(serde::Serialize, serde::Deserialize)]
            struct DataRef {
                start: usize,
                end: usize,
            }
            DataRef::deserialize(deserializer).map(|DataRef { start, end }| Self {
                start,
                end,
                phantom_data: PhantomData,
            })
        }
    }
}
