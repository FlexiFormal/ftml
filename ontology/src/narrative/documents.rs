use ftml_uris::{DocumentUri, Id, NarrativeUriRef, errors::SegmentParseError};

use crate::narrative::{
    Narrative,
    elements::{
        DocumentElement,
        paragraphs::{InvalidParagraphKind, ParagraphKind},
        sections::SectionLevel,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentData {
    pub uri: DocumentUri,
    pub title: Option<Box<str>>,
    pub elements: Box<[DocumentElement]>,
    pub styles: DocumentStyles,
}

impl crate::__private::Sealed for DocumentData {}
impl crate::Ftml for DocumentData {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        todo!();
        Vec::new()
    }
}
impl Narrative for DocumentData {
    #[inline]
    fn narrative_uri(&self) -> Option<NarrativeUriRef<'_>> {
        Some(NarrativeUriRef::Document(&self.uri))
    }
    #[inline]
    fn children(&self) -> &[DocumentElement] {
        &self.elements
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Document(triomphe::Arc<DocumentData>);
impl std::ops::Deref for Document {
    type Target = DocumentData;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentStyles {
    pub counters: Box<[DocumentCounter]>,
    pub styles: Box<[DocumentStyle]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentCounter {
    pub name: Id,
    pub parent: Option<SectionLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentStyle {
    pub kind: ParagraphKind,
    pub name: Option<Id>,
    pub counter: Option<Id>,
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
