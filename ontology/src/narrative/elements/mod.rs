pub mod notations;
pub mod paragraphs;
pub mod problems;
pub mod sections;
pub mod variables;

use crate::{
    Ftml,
    narrative::{
        DocumentRange,
        elements::notations::{NotationReference, VariableNotationReference},
    },
    terms::Term,
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, ModuleUri, SymbolUri};
pub use notations::Notation;
pub use paragraphs::LogicalParagraph;
pub use problems::Problem;
pub use sections::{Section, SectionLevel};
pub use variables::VariableDeclaration;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum ParagraphOrProblemKind {
    Definition,
    Example,
    Problem(problems::CognitiveDimension),
    SubProblem(problems::CognitiveDimension),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum SlideElement {
    Slide {
        html: Box<str>,
        uri: DocumentElementUri,
    },
    Paragraph {
        html: Box<str>,
        uri: DocumentElementUri,
    },
    Inputref {
        uri: DocumentUri,
    },
    Section {
        uri: DocumentElementUri,
        title: Option<Box<str>>,
        children: Vec<SlideElement>,
    },
}

pub trait IsDocumentElement: super::Narrative {
    fn as_ref(&self) -> DocumentElementRef<'_>;
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized;

    fn element_uri(&self) -> Option<&DocumentElementUri>;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DocumentElement {
    //SetSectionLevel(SectionLevel),
    UseModule(ModuleUri),
    Module {
        range: DocumentRange,
        module: ModuleUri,
        #[cfg_attr(feature = "typescript", tsify(type = "DocumentElement[]"))]
        #[cfg_attr(feature = "serde", serde(default))]
        children: Box<[Self]>,
    },
    MathStructure {
        range: DocumentRange,
        structure: SymbolUri,
        #[cfg_attr(feature = "typescript", tsify(type = "DocumentElement[]"))]
        #[cfg_attr(feature = "serde", serde(default))]
        children: Box<[Self]>,
    },
    Extension {
        range: DocumentRange,
        extension: SymbolUri,
        target: SymbolUri,
        #[cfg_attr(feature = "typescript", tsify(type = "DocumentElement[]"))]
        #[cfg_attr(feature = "serde", serde(default))]
        children: Box<[Self]>,
    },
    Morphism {
        range: DocumentRange,
        morphism: SymbolUri,
        #[cfg_attr(feature = "typescript", tsify(type = "DocumentElement[]"))]
        #[cfg_attr(feature = "serde", serde(default))]
        children: Box<[Self]>,
    },
    SymbolDeclaration(SymbolUri),
    ImportModule(ModuleUri),

    Section(Section),
    SkipSection(
        #[cfg_attr(feature = "typescript", tsify(type = "DocumentElement[]"))]
        #[cfg_attr(feature = "serde", serde(default))]
        Box<[Self]>,
    ),
    Paragraph(LogicalParagraph),
    Problem(Problem),
    Slide {
        range: DocumentRange,
        uri: DocumentElementUri,
        #[cfg_attr(feature = "serde", serde(default))]
        title: Option<Box<str>>,
        #[cfg_attr(feature = "typescript", tsify(type = "DocumentElement[]"))]
        children: Box<[Self]>,
    },
    DocumentReference {
        uri: DocumentElementUri,
        target: DocumentUri,
    },
    Notation(NotationReference),
    VariableDeclaration(VariableDeclaration),
    VariableNotation(VariableNotationReference),
    Definiendum {
        range: DocumentRange,
        uri: SymbolUri,
    },
    SymbolReference {
        range: DocumentRange,
        uri: SymbolUri,
        #[cfg_attr(feature = "serde", serde(default))]
        notation: Option<Id>,
    },
    VariableReference {
        range: DocumentRange,
        uri: DocumentElementUri,
        #[cfg_attr(feature = "serde", serde(default))]
        notation: Option<Id>,
    },
    Term(DocumentTerm),
}
impl crate::__private::Sealed for DocumentElement {}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DocumentTerm {
    pub uri: DocumentElementUri,
    pub term: Term,
}
impl crate::__private::Sealed for DocumentTerm {}
impl crate::Ftml for DocumentTerm {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        macro_rules! syms {
            ($iri:ident $e:expr) => {{
                $e.symbols().collect::<rustc_hash::FxHashSet<_>>().into_iter()
                    .map(move |s| triple!(<($iri.clone())> dc:hasPart <(s.to_iri())>))
            }};
        }
        let iri = self.uri.to_iri();
        let iri2 = iri.clone();
        let term = &self.term;
        syms!(iri term).chain(std::iter::once(triple!(<(iri2)>: ulo:term)))
    }
}
impl IsDocumentElement for DocumentTerm {
    #[inline]
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::Term(self)
    }
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        if let DocumentElementRef::Term(t) = e {
            Some(t)
        } else {
            None
        }
    }
    #[inline]
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
}

impl super::Narrative for DocumentTerm {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        Some(ftml_uris::NarrativeUriRef::Element(&self.uri))
    }
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        std::iter::empty()
    }
}

impl super::Narrative for DocumentElement {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        self.element_uri().map(ftml_uris::NarrativeUriRef::Element)
    }
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        #[allow(clippy::enum_glob_use)]
        use either_of::EitherOf5::*;
        match self {
            //Self::SetSectionLevel(_) |
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::VariableDeclaration(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Term { .. } => A(std::iter::empty()),
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => B(children.iter().map(Self::as_ref)),
            Self::Section(s) => C(s.children()),
            Self::Paragraph(s) => D(s.children()),
            Self::Problem(s) => E(s.children()),
        }
    }
}
impl Ftml for DocumentElement {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        match self {
            //Self::SetSectionLevel(_) |
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. } => RdfIterator::Empty(self.as_ref()),
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => {
                RdfIterator::Children(Box::new(children.iter().flat_map(Ftml::triples)))
            }
            Self::Section(s) => RdfIterator::Section(s.triples().into_iter()),
            Self::Paragraph(s) => RdfIterator::Paragraph(s.triples().into_iter()),
            Self::Problem(s) => RdfIterator::Problem(s.triples().into_iter()),
            Self::VariableDeclaration(v) => RdfIterator::Var(v.triples().into_iter()),
            Self::Notation(n) => RdfIterator::Notation(n.triples().into_iter()),
            Self::VariableNotation(n) => RdfIterator::VarNotation(n.triples().into_iter()),
            Self::Term(t) => RdfIterator::Term(t.triples().into_iter()),
        }
    }
}
impl DocumentElement {
    #[must_use]
    pub fn children_lt(&self) -> Option<&[Self]> {
        match self {
            //Self::SetSectionLevel(_) |
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::VariableDeclaration(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Term { .. } => None,
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => Some(&**children),
            Self::Section(s) => Some(&*s.children),
            Self::Paragraph(s) => Some(&*s.children),
            Self::Problem(s) => Some(&*s.children),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum DocumentElementRef<'d> {
    //SetSectionLevel(SectionLevel),
    UseModule(&'d ModuleUri),

    Module {
        range: DocumentRange,
        module: &'d ModuleUri,
        children: &'d [DocumentElement],
    },
    MathStructure {
        range: DocumentRange,
        structure: &'d SymbolUri,
        children: &'d [DocumentElement],
    },
    Extension {
        range: DocumentRange,
        extension: &'d SymbolUri,
        target: &'d SymbolUri,
        children: &'d [DocumentElement],
    },
    Morphism {
        range: DocumentRange,
        morphism: &'d SymbolUri,
        children: &'d [DocumentElement],
    },
    SymbolDeclaration(&'d SymbolUri),
    ImportModule(&'d ModuleUri),

    Section(&'d Section),
    SkipSection(&'d [DocumentElement]),
    Paragraph(&'d LogicalParagraph),
    Problem(&'d Problem),
    Slide {
        range: DocumentRange,
        uri: &'d DocumentElementUri,
        title: Option<&'d str>,
        children: &'d [DocumentElement],
    },
    DocumentReference {
        uri: &'d DocumentElementUri,
        target: &'d DocumentUri,
    },
    Notation(&'d NotationReference),
    VariableDeclaration(&'d VariableDeclaration),
    VariableNotation(&'d VariableNotationReference),
    Definiendum {
        range: DocumentRange,
        uri: &'d SymbolUri,
    },
    SymbolReference {
        range: DocumentRange,
        uri: &'d SymbolUri,
        notation: Option<&'d Id>,
    },
    VariableReference {
        range: DocumentRange,
        uri: &'d DocumentElementUri,
        notation: Option<&'d Id>,
    },
    Term(&'d DocumentTerm),
}

impl<'r> DocumentElementRef<'r> {
    pub fn children_lt(
        self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'r>> + DoubleEndedIterator {
        use super::Narrative;
        #[allow(clippy::enum_glob_use)]
        use either_of::EitherOf5::*;
        match self {
            //Self::SetSectionLevel(_) |
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::VariableDeclaration(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Term { .. } => A(std::iter::empty()),
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => B(children.iter().map(DocumentElement::as_ref)),
            Self::Section(s) => C(s.children()),
            Self::Paragraph(s) => D(s.children()),
            Self::Problem(s) => E(s.children()),
        }
    }
}

impl crate::__private::Sealed for DocumentElementRef<'_> {}
impl super::Narrative for DocumentElementRef<'_> {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        self.element_uri().map(ftml_uris::NarrativeUriRef::Element)
    }
    #[inline]
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        self.children_lt()
    }
}
impl crate::Ftml for DocumentElementRef<'_> {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        match self {
            //Self::SetSectionLevel(_)
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. } => RdfIterator::Empty(*self),
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => {
                RdfIterator::Children(Box::new(children.iter().flat_map(Ftml::triples)))
            }
            Self::Section(s) => RdfIterator::Section(s.triples().into_iter()),
            Self::Paragraph(s) => RdfIterator::Paragraph(s.triples().into_iter()),
            Self::Problem(s) => RdfIterator::Problem(s.triples().into_iter()),
            Self::VariableDeclaration(v) => RdfIterator::Var(v.triples().into_iter()),
            Self::Notation(n) => RdfIterator::Notation(n.triples().into_iter()),
            Self::VariableNotation(n) => RdfIterator::VarNotation(n.triples().into_iter()),
            Self::Term(t) => RdfIterator::Term(t.triples().into_iter()),
        }
    }
}

pub trait FlatIterable {
    type Ref: std::ops::Deref<Target = DocumentElement>;
    fn flat(self) -> impl Iterator<Item = Self::Ref>;
}

impl<'a, I: Iterator<Item = &'a DocumentElement>> FlatIterable for I {
    type Ref = &'a DocumentElement;
    fn flat(self) -> impl Iterator<Item = Self::Ref> {
        self.flat_map(|e| {
            if e.flat_children().is_empty() {
                either::Right(std::iter::once(e))
            } else {
                either::Left(e.flat_children().iter())
            }
        })
    }
}

impl DocumentElement {
    #[must_use]
    pub fn flat_children(&self) -> &[Self] {
        match self {
            //Self::SetSectionLevel(_) |
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Section(_)
            | Self::Paragraph(_)
            | Self::Problem(_)
            | Self::VariableDeclaration(_)
            | Self::Slide { .. }
            | Self::DocumentReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Term { .. } => &[],
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::SkipSection(children) => children,
        }
    }
    #[must_use]
    pub const fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(match self {
            // Self::SetSectionLevel(_) |
            Self::UseModule(_)
            | Self::Module { .. }
            | Self::MathStructure { .. }
            | Self::Extension { .. }
            | Self::Morphism { .. }
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::SkipSection(_)
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. } => return None,
            Self::Section(s) => &s.uri,
            Self::Paragraph(s) => &s.uri,
            Self::Problem(s) => &s.uri,
            Self::VariableDeclaration(s) => &s.uri,
            Self::Slide { uri, .. }
            | Self::DocumentReference { uri, .. }
            | Self::Notation(NotationReference { uri, .. })
            | Self::VariableNotation(VariableNotationReference { uri, .. })
            | Self::Term(DocumentTerm { uri, .. }) => uri,
        })
    }
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn as_ref(&self) -> DocumentElementRef<'_> {
        match self {
            //Self::SetSectionLevel(l) => DocumentElementRef::SetSectionLevel(*l),
            Self::UseModule(u) => DocumentElementRef::UseModule(u),
            Self::Module {
                range,
                module,
                children,
            } => DocumentElementRef::Module {
                range: *range,
                module,
                children,
            },
            Self::MathStructure {
                range,
                structure,
                children,
            } => DocumentElementRef::MathStructure {
                range: *range,
                structure,
                children,
            },
            Self::Extension {
                range,
                extension,
                target,
                children,
            } => DocumentElementRef::Extension {
                range: *range,
                extension,
                target,
                children,
            },
            Self::Morphism {
                range,
                morphism,
                children,
            } => DocumentElementRef::Morphism {
                range: *range,
                morphism,
                children,
            },
            Self::SymbolDeclaration(uri) => DocumentElementRef::SymbolDeclaration(uri),
            Self::ImportModule(uri) => DocumentElementRef::ImportModule(uri),
            Self::Section(s) => DocumentElementRef::Section(s),
            Self::SkipSection(e) => DocumentElementRef::SkipSection(e),
            Self::Paragraph(p) => DocumentElementRef::Paragraph(p),
            Self::Problem(p) => DocumentElementRef::Problem(p),
            Self::Slide {
                range,
                uri,
                title,
                children,
            } => DocumentElementRef::Slide {
                range: *range,
                uri,
                title: title.as_deref(),
                children,
            },
            Self::DocumentReference { uri, target } => {
                DocumentElementRef::DocumentReference { uri, target }
            }
            Self::Notation(n) => DocumentElementRef::Notation(n),
            Self::VariableDeclaration(v) => DocumentElementRef::VariableDeclaration(v),
            Self::VariableNotation(n) => DocumentElementRef::VariableNotation(n),
            Self::Definiendum { range, uri } => {
                DocumentElementRef::Definiendum { range: *range, uri }
            }
            Self::SymbolReference {
                range,
                uri,
                notation,
            } => DocumentElementRef::SymbolReference {
                range: *range,
                uri,
                notation: notation.as_ref(),
            },
            Self::VariableReference {
                range,
                uri,
                notation,
            } => DocumentElementRef::VariableReference {
                range: *range,
                uri,
                notation: notation.as_ref(),
            },
            Self::Term(t) => DocumentElementRef::Term(t),
        }
    }
}

impl<'e> DocumentElementRef<'e> {
    #[must_use]
    pub fn opaque_children(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = DocumentElementRef<'e>> + DoubleEndedIterator + use<'e>>
    {
        match self {
            //Self::SetSectionLevel(_)
            Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Section(_)
            | Self::Paragraph(_)
            | Self::Problem(_)
            | Self::VariableDeclaration(_)
            | Self::Slide { .. }
            | Self::DocumentReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Term { .. } => None,
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::SkipSection(children) => Some(children.iter().map(|e| e.as_ref())),
        }
    }
    #[must_use]
    pub const fn element_uri(self) -> Option<&'e DocumentElementUri> {
        Some(match self {
            //Self::SetSectionLevel(_)
            Self::UseModule(_)
            | Self::Module { .. }
            | Self::MathStructure { .. }
            | Self::Extension { .. }
            | Self::Morphism { .. }
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::SkipSection(_)
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. } => return None,
            Self::Section(s) => &s.uri,
            Self::Paragraph(s) => &s.uri,
            Self::Problem(s) => &s.uri,
            Self::VariableDeclaration(s) => &s.uri,
            Self::Slide { uri, .. }
            | Self::DocumentReference { uri, .. }
            | Self::Notation(NotationReference { uri, .. })
            | Self::VariableNotation(VariableNotationReference { uri, .. })
            | Self::Term(DocumentTerm { uri, .. }) => uri,
        })
    }
}

#[cfg(feature = "rdf")]
#[allow(clippy::large_enum_variant)]
enum RdfIterator<
    'e,
    S: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    Pa: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    Pr: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    V: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    VN: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    N: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    T: Iterator<Item = ulo::rdf_types::Triple> + 'e,
> {
    #[allow(dead_code)]
    Empty(DocumentElementRef<'e>),
    Children(Box<dyn Iterator<Item = ulo::rdf_types::Triple> + 'e>),
    Section(S),
    Paragraph(Pa),
    Problem(Pr),
    Notation(N),
    VarNotation(VN),
    Var(V),
    Term(T),
}
#[cfg(feature = "rdf")]
impl<
    'e,
    S: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    Pa: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    Pr: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    V: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    VN: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    N: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    T: Iterator<Item = ulo::rdf_types::Triple> + 'e,
> Iterator for RdfIterator<'e, S, Pa, Pr, V, VN, N, T>
{
    type Item = ulo::rdf_types::Triple;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Empty(_) => None,
            Self::Children(c) => c.next(),
            Self::Section(s) => s.next(),
            Self::Paragraph(p) => p.next(),
            Self::Problem(p) => p.next(),
            Self::Var(v) => v.next(),
            Self::Notation(n) => n.next(),
            Self::VarNotation(n) => n.next(),
            Self::Term(t) => t.next(),
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for DocumentElement {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => children
                .iter()
                .map(|c| std::mem::size_of_val(c) + c.deep_size_of_children(context))
                .sum::<usize>(),
            Self::Section(s) => s.deep_size_of_children(context),
            Self::Paragraph(s) => s.deep_size_of_children(context),
            Self::Problem(s) => s.deep_size_of_children(context),
            Self::VariableDeclaration(s) => s.deep_size_of_children(context),
            Self::Term(t) => t.term.deep_size_of_children(context),
            _ => 0,
        }
    }
}
