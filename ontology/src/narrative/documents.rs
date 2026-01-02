use std::borrow::Borrow;

use ftml_uris::{DocumentElementUri, DocumentUri, Id, NarrativeUriRef, errors::SegmentParseError};

use crate::{
    narrative::{
        Narrative,
        elements::{
            DocumentElement, DocumentElementRef,
            paragraphs::{InvalidParagraphKind, ParagraphKind},
            sections::SectionLevel,
        },
    },
    utils::{RefTree, TreeChild, time::Timestamp},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentData {
    pub uri: DocumentUri,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub title: Option<Box<str>>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
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
        use either_of::EitherOf3::{A, B, C};
        use ftml_uris::{FtmlUri, IsNarrativeUri, UriWithArchive};
        use ulo::triple;
        let iri = self.uri.to_iri();
        [
            triple!(<(iri.clone())> dc:language = (self.uri.language().to_string()) ),
            triple!(<(iri.clone())> : ulo:document),
            triple!(<(self.uri.archive_uri().to_iri())> ulo:contains <(iri.clone())>),
        ]
        .into_iter()
        .chain(match &self.kind {
            DocumentKind::Article | DocumentKind::Fragment => A(std::iter::empty()),
            DocumentKind::Exam { date, course,retake,num,term } => B([
                Some(triple!(<(iri.clone())> : ulo:exam)),
                if *retake {Some(triple!(<(iri.clone())> : ulo:retake_exam))} else {None},
                Some(triple!(<(iri.clone())> ulo:has_course = (course.to_string()))),
                term.as_ref().map(|term| triple!(<(iri.clone())> ulo:has_course_term = (term.to_string()))),
                Some(triple!(<(iri.clone())> ulo:is_number != (ulo::rdf_types::RDFTerm::Literal(
                    ulo::rdf_types::Literal::new_typed_literal(num.to_string(), ulo::xsd::POSITIVE_INTEGER.into_owned())
                )))),
                Some(triple!(<(iri)> ulo:has_date != (ulo::rdf_types::RDFTerm::Literal(
                    ulo::rdf_types::Literal::new_typed_literal(date.xsd().to_string(), ulo::xsd::DATE_TIME.into_owned())
                )))),
                ].into_iter().flatten()),
            DocumentKind::Quiz { date, course,num,term } => C([
                    Some(triple!(<(iri.clone())> : ulo:quiz)),
                    Some(triple!(<(iri.clone())> ulo:has_course = (course.to_string()))),
                    term.as_ref().map(|term| triple!(<(iri.clone())> ulo:has_course_term = (term.to_string()))),
                    Some(triple!(<(iri.clone())> ulo:is_number != (ulo::rdf_types::RDFTerm::Literal(
                        ulo::rdf_types::Literal::new_typed_literal(num.to_string(), ulo::xsd::POSITIVE_INTEGER.into_owned())
                    )))),
                    Some(triple!(<(iri)> ulo:has_date != (ulo::rdf_types::RDFTerm::Literal(
                        ulo::rdf_types::Literal::new_typed_literal(date.xsd().to_string(), ulo::xsd::DATE_TIME.into_owned())
                    )))),
                ].into_iter().flatten()),
            DocumentKind::Homework { date, course,term,num } => {
                C([
                    Some(triple!(<(iri.clone())> : ulo:homework)),
                    Some(triple!(<(iri.clone())> ulo:has_course = (course.to_string()))),
                    term.as_ref().map(|term| triple!(<(iri.clone())> ulo:has_course_term = (term.to_string()))),
                    Some(triple!(<(iri.clone())> ulo:is_number != (ulo::rdf_types::RDFTerm::Literal(
                        ulo::rdf_types::Literal::new_typed_literal(num.to_string(), ulo::xsd::POSITIVE_INTEGER.into_owned())
                    )))),
                    Some(triple!(<(iri)> ulo:has_date != (ulo::rdf_types::RDFTerm::Literal(
                        ulo::rdf_types::Literal::new_typed_literal(date.xsd().to_string(), ulo::xsd::DATE_TIME.into_owned())
                    )))),
                ].into_iter().flatten())
            }
        })
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentStyles {
    pub counters: Box<[DocumentCounter]>,
    pub styles: Box<[DocumentStyle]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentCounter {
    pub name: Id,
    pub parent: Option<SectionLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentStyle {
    pub kind: ParagraphKind,
    pub name: Option<Id>,
    pub counter: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DocumentKind {
    #[default]
    Article,
    Fragment,
    Exam {
        date: Timestamp,
        course: Id,
        retake: bool,
        num: u16,
        term: Option<Id>,
    },
    Homework {
        date: Timestamp,
        course: Id,
        num: u16,
        term: Option<Id>,
    },
    Quiz {
        date: Timestamp,
        course: Id,
        num: u16,
        term: Option<Id>,
    },
}
impl std::str::FromStr for DocumentKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "article" => Self::Article,
            "fragment" => Self::Fragment,
            "exam" => Self::Exam {
                date: Timestamp::default(),
                // SAFETY: known to be a valid Id
                course: unsafe { Id::new("course").unwrap_unchecked() },
                retake: false,
                num: 0,
                term: None,
            },
            "homework" => Self::Homework {
                date: Timestamp::default(),
                // SAFETY: known to be a valid Id
                course: unsafe { Id::new("course").unwrap_unchecked() },
                num: 0,
                term: None,
            },
            "quiz" => Self::Quiz {
                date: Timestamp::default(),
                // SAFETY: known to be a valid Id
                course: unsafe { Id::new("course").unwrap_unchecked() },
                num: 0,
                term: None,
            },
            _ => return Err(()),
        })
    }
}
impl std::fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Article => "article",
            Self::Fragment => "fragment",
            Self::Exam { .. } => "exam",
            Self::Homework { .. } => "homework",
            Self::Quiz { .. } => "quiz",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(tag = "type"))]
/// An entry in a table of contents. Either:
/// 1. a section; the title is assumed to be an HTML string, or
/// 2. an inputref to some other document; the URI is the one for the
///    inputref itself; not the referenced Document. For the TOC,
///    which document is inputrefed is actually irrelevant.
pub enum TocElem {
    /// A section; the title is assumed to be an HTML string
    Section {
        title: Option<Box<str>>,
        uri: DocumentElementUri,
        id: String,
        children: Vec<Self>,
    },
    SkippedSection {
        children: Vec<Self>,
    },
    /// An inputref to some other document; the URI is the one for the
    /// referenced Document.
    Inputref {
        uri: DocumentUri,
        title: Option<Box<str>>,
        id: String,
        children: Vec<Self>,
    },
    Paragraph {
        styles: Vec<Id>,
        kind: ParagraphKind,
    },
    Slide, //{uri:DocumentElementUri}
}
/*
impl TocElem {
    pub fn iter(v: &[Self]) -> impl Iterator<Item = &Self> {
        v.iter().flat_map(|e| std::iter::once(e).chain(e.dfs()))
    }
}
 */

impl RefTree for TocElem {
    type Child<'a> = &'a Self;
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        match self {
            Self::Section { children, .. }
            | Self::SkippedSection { children }
            | Self::Inputref { children, .. } => either::Right(children.iter()),
            _ => either::Left(std::iter::empty()),
        }
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

impl RefTree for Document {
    type Child<'a>
        = DocumentElementRef<'a>
    where
        Self: 'a;
    #[inline]
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        self.children()
    }
}
impl<'a> TreeChild<'a> for DocumentElementRef<'a> {
    #[inline]
    fn tree_children(self) -> impl Iterator<Item = Self> {
        self.children_lt()
    }
}

#[cfg(feature = "serde-lite")]
mod serde_lite_impl {
    use crate::narrative::documents::DocumentData;

    impl serde_lite::Serialize for super::Document {
        #[inline]
        fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
            self.0.serialize()
        }
    }
    impl serde_lite::Deserialize for super::Document {
        fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error> {
            Ok(Self(triomphe::Arc::new(DocumentData::deserialize(val)?)))
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::narrative::documents::DocumentData;

    impl<Context> bincode::Decode<Context> for super::Document {
        fn decode<D: bincode::de::Decoder<Context = Context>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            DocumentData::decode(decoder).map(|d| Self(triomphe::Arc::new(d)))
        }
    }
    impl bincode::Encode for super::Document {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            self.0.encode(encoder)
        }
    }

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
