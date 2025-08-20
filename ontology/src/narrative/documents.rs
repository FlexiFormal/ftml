use std::borrow::Borrow;

use ftml_uris::{DocumentUri, Id, NarrativeUriRef, errors::SegmentParseError};

use crate::narrative::{
    Narrative,
    elements::{
        DocumentElement, DocumentElementRef,
        paragraphs::{InvalidParagraphKind, ParagraphKind},
        sections::SectionLevel,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentData {
    pub uri: DocumentUri,
    #[cfg_attr(feature = "serde", serde(default))]
    pub title: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub elements: Box<[DocumentElement]>,
    pub styles: DocumentStyles,
    pub top_section_level: SectionLevel,
    pub kind: DocumentKind,
}
impl DocumentData {
    #[must_use]
    #[inline]
    pub fn close(self) -> Document {
        Document(triomphe::Arc::new(self))
    }
}

impl crate::__private::Sealed for DocumentData {}
impl crate::Ftml for DocumentData {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::{FtmlUri, IsNarrativeUri, UriWithArchive};
        use ulo::triple;
        let iri = self.uri.to_iri();
        [
            triple!(<(iri.clone())> dc:language = (self.uri.language().to_string()) ),
            triple!(<(iri.clone())> : ulo:document),
            triple!(<(self.uri.archive_uri().to_iri())> ulo:contains <(iri)>),
        ]
        .into_iter()
        .chain(self.contains_triples())
    }
}
impl Narrative for DocumentData {
    #[inline]
    fn narrative_uri(&self) -> Option<NarrativeUriRef<'_>> {
        Some(NarrativeUriRef::Document(&self.uri))
    }
    #[inline]
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        self.elements.iter().map(DocumentElement::as_ref)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document(pub(crate) triomphe::Arc<DocumentData>);
impl std::ops::Deref for Document {
    type Target = DocumentData;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::hash::Hash for Document {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}
impl Borrow<DocumentUri> for Document {
    #[inline]
    fn borrow(&self) -> &DocumentUri {
        &self.uri
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentStyles {
    pub counters: Box<[DocumentCounter]>,
    pub styles: Box<[DocumentStyle]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentCounter {
    pub name: Id,
    pub parent: Option<SectionLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentStyle {
    pub kind: ParagraphKind,
    pub name: Option<Id>,
    pub counter: Option<Id>,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
pub enum DocumentKind {
    #[default]
    Article,
    Fragment,
    Exam,
    Homework,
    Quiz,
}
impl std::str::FromStr for DocumentKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "article" => Self::Article,
            "fragment" => Self::Fragment,
            "exam" => Self::Exam,
            "homework" => Self::Homework,
            "quiz" => Self::Quiz,
            _ => return Err(()),
        })
    }
}
impl std::fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Article => "article",
            Self::Fragment => "fragment",
            Self::Exam => "exam",
            Self::Homework => "homework",
            Self::Quiz => "quiz",
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StyleParseError {
    #[error("invalid paragraph kind in style: {0}")]
    Paragraph(#[from] InvalidParagraphKind),
    #[error("invalid style id: {0}")]
    Parse(#[from] SegmentParseError),
}

impl std::str::FromStr for DocumentStyle {
    type Err = StyleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((a, b)) = s.split_once('-') {
            let kind = ParagraphKind::from_str(a)?;
            let name = Some(Id::from_str(b)?);
            return Ok(Self {
                kind,
                name,
                counter: None,
            });
        }
        let kind = ParagraphKind::from_str(s)?;
        Ok(Self {
            kind,
            name: None,
            counter: None,
        })
    }
}
#[cfg(feature = "serde")]
mod serde_impl {
    use crate::narrative::documents::DocumentData;

    impl serde::Serialize for super::Document {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }
    impl<'de> serde::Deserialize<'de> for super::Document {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            DocumentData::deserialize(deserializer).map(|d| Self(triomphe::Arc::new(d)))
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for DocumentStyles {
    fn deep_size_of_children(&self, _: &mut deepsize::Context) -> usize {
        (self.counters.len() * std::mem::size_of::<DocumentCounter>())
            + (self.styles.len() * std::mem::size_of::<DocumentStyle>())
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for DocumentData {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.title.as_ref().map(|s| s.len()).unwrap_or_default()
            + self.styles.deep_size_of_children(context)
            + self
                .elements
                .iter()
                .map(|e| std::mem::size_of_val(e) + e.deep_size_of_children(context))
                .sum::<usize>()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Document {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        std::mem::size_of::<DocumentData>() + self.0.deep_size_of_children(context)
    }
}
