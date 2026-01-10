use crate::{
    narrative::{
        DocumentRange, Narrative,
        elements::{DocumentElement, DocumentElementRef, IsDocumentElement},
    },
    terms::Term,
    utils::SourceRange,
};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
pub struct LogicalParagraph {
    pub kind: ParagraphKind,
    pub uri: DocumentElementUri,
    pub formatting: ParagraphFormatting,
    pub range: DocumentRange,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub title: Option<Box<str>>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub styles: Box<[Id]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub children: Box<[DocumentElement]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub fors: Box<[(SymbolUri, Option<Term>)]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub source: SourceRange,
}
impl crate::__private::Sealed for LogicalParagraph {}
impl crate::Ftml for LogicalParagraph {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();
        let iri2 = iri.clone();
        self.contains_triples()
            .into_iter()
            .chain(self.fors.iter().map(move |(s, _)| {
                if self.kind.is_definition_like(&self.styles) {
                    triple!(<(iri2.clone())> ulo:defines <(s.to_iri())>)
                } else if self.kind == ParagraphKind::Example {
                    triple!(<(iri2.clone())> ulo:example_for <(s.to_iri())>)
                } else {
                    triple!(<(iri2.clone())> ulo:crossrefs <(s.to_iri())>)
                }
            }))
            .chain(std::iter::once(
                triple!(<(iri)> : <(self.kind.rdf_type().into_owned())>),
            ))
    }
    #[inline]
    fn source_range(&self) -> SourceRange {
        self.source
    }
}
impl Narrative for LogicalParagraph {
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
impl IsDocumentElement for LogicalParagraph {
    #[inline]
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
    #[inline]
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::Paragraph(self)
    }
    #[inline]
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        match e {
            DocumentElementRef::Paragraph(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
pub enum ParagraphFormatting {
    Block,
    Inline,
    Collapsed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
#[non_exhaustive]
#[repr(u8)]
pub enum ParagraphKind {
    Definition = 0,
    Assertion = 1,
    Paragraph = 2,
    Proof = 3,
    SubProof = 4,
    Example = 5,
}

impl ParagraphKind {
    #[must_use]
    pub fn is_definition_like(&self, styles: &[Id]) -> bool {
        match &self {
            Self::Definition | Self::Assertion | Self::Proof | Self::SubProof => true,
            _ => styles
                .iter()
                .any(|s| s.as_ref() == "symdoc" || s.as_ref() == "decl"),
        }
    }

    #[cfg(feature = "rdf")]
    #[must_use]
    #[allow(clippy::wildcard_imports)]
    pub const fn rdf_type(&self) -> ulo::rdf_types::NamedNodeRef<'static> {
        use ulo::ulo::*;
        match self {
            Self::Definition => definition,
            Self::Assertion => proposition,
            Self::Paragraph => para,
            Self::Proof => proof,
            Self::SubProof => subproof,
            Self::Example => example,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::Assertion => "assertion",
            Self::Paragraph => "paragraph",
            Self::Proof => "proof",
            Self::SubProof => "subproof",
            Self::Example => "example",
        }
    }

    #[must_use]
    pub const fn as_display_str(self) -> &'static str {
        match self {
            Self::Definition => "Definition",
            Self::Assertion => "Assertion",
            Self::Paragraph => "Paragraph",
            Self::Proof => "Proof",
            Self::SubProof => "Subproof",
            Self::Example => "Example",
        }
    }
}

impl std::fmt::Display for ParagraphKind {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_display_str())
    }
}

#[cfg(feature = "typescript")]
impl ftml_js_utils::conversion::ToJs for ParagraphKind {
    type Error = std::convert::Infallible;
    fn to_js(&self) -> Result<wasm_bindgen::JsValue, Self::Error> {
        Ok(wasm_bindgen::JsValue::from_f64((*self as u8).into()))
    }
}

#[cfg(feature = "typescript")]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
impl wasm_bindgen::convert::TryFromJsValue for ParagraphKind {
    fn try_from_js_value(value: wasm_bindgen::JsValue) -> Result<Self, wasm_bindgen::JsValue> {
        let Some(jbyte) = value.as_f64() else {
            return Err(value);
        };
        let u = jbyte as u8;
        Ok(match u {
            0 => Self::Definition,
            1 => Self::Assertion,
            2 => Self::Paragraph,
            3 => Self::Proof,
            4 => Self::SubProof,
            5 => Self::Example,
            _ => return Err(value),
        })
    }

    fn try_from_js_value_ref(value: &wasm_bindgen::JsValue) -> Option<Self> {
        let u = value.as_f64()? as u8;
        Some(match u {
            0 => Self::Definition,
            1 => Self::Assertion,
            2 => Self::Paragraph,
            3 => Self::Proof,
            4 => Self::SubProof,
            5 => Self::Example,
            _ => return None,
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid paragraph kind")]
pub struct InvalidParagraphKind;

impl std::str::FromStr for ParagraphKind {
    type Err = InvalidParagraphKind;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "definition" => Ok(Self::Definition),
            "assertion" => Ok(Self::Assertion),
            "paragraph" => Ok(Self::Paragraph),
            "proof" => Ok(Self::Proof),
            "subproof" => Ok(Self::SubProof),
            "example" => Ok(Self::Example),
            _ => Err(InvalidParagraphKind),
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for LogicalParagraph {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.title.as_ref().map(|s| s.len()).unwrap_or_default()
            + self
                .children
                .iter()
                .map(|e| std::mem::size_of_val(e) + e.deep_size_of_children(context))
                .sum::<usize>()
            + (self.styles.len() * std::mem::size_of::<Id>())
            + self
                .fors
                .iter()
                .map(|p| {
                    std::mem::size_of_val(p)
                        + p.1
                            .as_ref()
                            .map(|t| t.deep_size_of_children(context))
                            .unwrap_or_default()
                })
                .sum::<usize>()
    }
}
