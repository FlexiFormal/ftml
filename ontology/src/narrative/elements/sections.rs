use ftml_uris::DocumentElementUri;

use crate::narrative::{
    DocumentRange, Narrative,
    elements::{DocumentElement, DocumentElementRef, IsDocumentElement},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Section {
    pub range: DocumentRange,
    pub uri: DocumentElementUri,
    //pub level: SectionLevel,
    pub title: Option<DocumentRange>,
    pub children: Box<[DocumentElement]>,
}
impl crate::__private::Sealed for Section {}
impl crate::Ftml for Section {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();
        self.contains_triples()
            .into_iter()
            .chain(std::iter::once(triple!(<(iri)> : ulo:section)))
    }
}
impl Narrative for Section {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        Some(ftml_uris::NarrativeUriRef::Element(&self.uri))
    }
    #[inline]
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        self.children.iter().map(DocumentElement::as_ref)
    }
}
impl IsDocumentElement for Section {
    #[inline]
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
    #[inline]
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::Section(self)
    }
    #[inline]
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        match e {
            DocumentElementRef::Section(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum SectionLevel {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    Subparagraph,
}
#[cfg(feature = "typescript")]
impl wasm_bindgen::convert::TryFromJsValue for SectionLevel {
    type Error = wasm_bindgen::JsValue;
    fn try_from_js_value(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        let Some(jstr) = value.as_string() else {
            return Err(value);
        };
        Ok(match jstr.as_str() {
            "Part" => Self::Part,
            "Chapter" => Self::Chapter,
            "Section" => Self::Section,
            "Subsection" => Self::Subsection,
            "Subsubsection" => Self::Subsubsection,
            "Paragraph" => Self::Paragraph,
            "Subparagraph" => Self::Subparagraph,
            _ => return Err(value),
        })
    }
}
impl Ord for SectionLevel {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let su: u8 = (*self).into();
        let ou: u8 = (*other).into();
        ou.cmp(&su)
    }
}
impl PartialOrd for SectionLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl SectionLevel {
    #[must_use]
    pub const fn inc(self) -> Self {
        match self {
            Self::Part => Self::Chapter,
            Self::Chapter => Self::Section,
            Self::Section => Self::Subsection,
            Self::Subsection => Self::Subsubsection,
            Self::Subsubsection => Self::Paragraph,
            _ => Self::Subparagraph,
        }
    }
}
impl std::fmt::Display for SectionLevel {
    #[allow(clippy::enum_glob_use)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SectionLevel::*;
        write!(
            f,
            "{}",
            match self {
                Part => "Part",
                Chapter => "Chapter",
                Section => "Section",
                Subsection => "Subsection",
                Subsubsection => "Subsubsection",
                Paragraph => "Paragraph",
                Subparagraph => "Subparagraph",
            }
        )
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid section level")]
pub struct InvalidSectionLevel;

impl TryFrom<u8> for SectionLevel {
    type Error = InvalidSectionLevel;
    #[allow(clippy::enum_glob_use)]
    fn try_from(value: u8) -> Result<Self, InvalidSectionLevel> {
        use SectionLevel::*;
        match value {
            0 => Ok(Part),
            1 => Ok(Chapter),
            2 => Ok(Section),
            3 => Ok(Subsection),
            4 => Ok(Subsubsection),
            5 => Ok(Paragraph),
            6 => Ok(Subparagraph),
            _ => Err(InvalidSectionLevel),
        }
    }
}
impl From<SectionLevel> for u8 {
    #[allow(clippy::enum_glob_use)]
    fn from(s: SectionLevel) -> Self {
        use SectionLevel::*;
        match s {
            Part => 0,
            Chapter => 1,
            Section => 2,
            Subsection => 3,
            Subsubsection => 4,
            Paragraph => 5,
            Subparagraph => 6,
        }
    }
}
