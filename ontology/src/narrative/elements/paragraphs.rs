use crate::{
    narrative::{
        DocumentRange, Narrative,
        elements::{DocumentElement, DocumentElementRef, IsDocumentElement},
    },
    terms::Term,
};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct LogicalParagraph {
    pub kind: ParagraphKind,
    pub uri: DocumentElementUri,
    pub formatting: ParagraphFormatting,
    pub title: Option<Box<str>>,
    pub range: DocumentRange,
    pub styles: Box<[Id]>,
    pub children: Box<[DocumentElement]>,
    pub fors: Box<[(SymbolUri, Option<Term>)]>,
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
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[non_exhaustive]
pub enum ParagraphKind {
    Definition,
    Assertion,
    Paragraph,
    Proof,
    SubProof,
    Example,
}

impl ParagraphKind {
    #[must_use]
    pub fn is_definition_like(&self, styles: &[Id]) -> bool {
        match &self {
            Self::Definition | Self::Assertion => true,
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
impl ftml_js_utils::conversion::SerdeToJs for ParagraphKind {}

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
