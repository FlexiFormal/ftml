pub mod documents;
pub mod elements;

use ftml_uris::{DocumentUri, NarrativeUriRef};
use std::marker::PhantomData;

use crate::narrative::elements::DocumentElementRef;

pub trait Narrative: crate::Ftml {
    fn narrative_uri(&self) -> Option<NarrativeUriRef<'_>>;
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator;

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
                .flat_map(|e| {
                    e.opaque_children().map_or_else(
                        || either::Either::Left(std::iter::once(e)),
                        either::Either::Right,
                    )
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
        fn find_e<'r, 's, T: elements::IsDocumentElement>(
            slf: DocumentElementRef<'r>,
            mut steps: std::iter::Peekable<impl Iterator<Item = &'s str>>,
        ) -> Option<&'r T> {
            let Some(step) = steps.next() else {
                return T::from_element(slf);
            };
            if let Some(i) = slf.opaque_children() {
                find_inner(i, step, steps)
            } else {
                find_inner(slf.children_lt(), step, steps)
            }
        }
        fn find_inner<'r, 's, T: elements::IsDocumentElement>(
            mut iter: impl Iterator<Item = DocumentElementRef<'r>>,
            step: &'s str,
            mut steps: std::iter::Peekable<impl Iterator<Item = &'s str>>,
        ) -> Option<&'r T> {
            while let Some(c) = iter.next() {
                match c {
                    DocumentElementRef::Section(elements::Section { uri, .. })
                    | DocumentElementRef::Paragraph(elements::LogicalParagraph { uri, .. })
                    | DocumentElementRef::Problem(elements::Problem { uri, .. })
                        if uri.name().last() == step =>
                    {
                        return if steps.peek().is_none() {
                            T::from_element(c)
                        } else {
                            find_e(c, steps)
                        };
                    }
                    DocumentElementRef::Slide { uri, .. }
                        if uri.name().last() == step && steps.peek().is_none() =>
                    {
                        return T::from_element(c);
                    }
                    DocumentElementRef::Module { .. }
                    | DocumentElementRef::Morphism { .. }
                    | DocumentElementRef::MathStructure { .. }
                    | DocumentElementRef::Slide { .. }
                    | DocumentElementRef::Extension { .. } => {
                        return find_inner(iter, step, steps);
                    }
                    DocumentElementRef::Notation { uri, .. }
                    | DocumentElementRef::VariableNotation { uri, .. }
                    | DocumentElementRef::VariableDeclaration(elements::VariableDeclaration {
                        uri,
                        ..
                    })
                    | DocumentElementRef::Expr { uri, .. }
                        if uri.name().last() == step =>
                    {
                        return if steps.peek().is_none() {
                            T::from_element(c)
                        } else {
                            None
                        };
                    }
                    DocumentElementRef::Section(_)
                    | DocumentElementRef::Paragraph(_)
                    | DocumentElementRef::Problem(_)
                    | DocumentElementRef::SetSectionLevel(_)
                    | DocumentElementRef::SymbolDeclaration(_)
                    | DocumentElementRef::UseModule(_)
                    | DocumentElementRef::ImportModule(_)
                    | DocumentElementRef::SkipSection(_)
                    | DocumentElementRef::VariableDeclaration(_)
                    | DocumentElementRef::Definiendum { .. }
                    | DocumentElementRef::SymbolReference { .. }
                    | DocumentElementRef::VariableReference { .. }
                    | DocumentElementRef::DocumentReference { .. }
                    | DocumentElementRef::Notation { .. }
                    | DocumentElementRef::VariableNotation { .. }
                    | DocumentElementRef::Expr { .. } => (),
                }
            }
            None
        }
        let mut steps = steps.into_iter();
        let step = steps.next()?;
        find_inner(self.children(), step, steps.peekable())
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocDataRef<T> {
    pub start: usize,
    pub end: usize,
    pub in_doc: DocumentUri,
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom_data: PhantomData<T>,
}

#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
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
pub use serde_impl::DataBuffer;

#[cfg(feature = "serde")]
mod serde_impl {
    use std::marker::PhantomData;

    use serde::ser::SerializeStruct;

    use crate::narrative::DataRef;

    #[derive(Debug, Default)]
    pub struct DataBuffer(Vec<u8>);
    impl DataBuffer {
        /// ### Errors
        pub fn push<T: serde::Serialize>(
            &mut self,
            t: &T,
        ) -> Result<DataRef<T>, bincode::error::EncodeError> {
            let curr = self.0.len();
            bincode::serde::encode_into_std_write(t, &mut self.0, bincode::config::standard())?;
            Ok(DataRef {
                start: curr,
                end: self.0.len(),
                phantom_data: PhantomData,
            })
        }

        #[inline]
        #[must_use]
        pub fn take(self) -> Box<[u8]> {
            self.0.into_boxed_slice()
        }
    }

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
