pub mod notations;
pub mod paragraphs;
pub mod problems;
pub mod sections;
pub mod variables;

use crate::{
    Ftml,
    expressions::Expr,
    narrative::{DataRef, DocumentRange},
};
use ftml_uris::{DocumentElementUri, DocumentUri, ModuleUri, SymbolUri, UriName};
pub use notations::Notation;
pub use paragraphs::LogicalParagraph;
pub use problems::Problem;
pub use sections::{Section, SectionLevel};
pub use variables::VariableDeclaration;

pub trait IsDocumentElement: super::Narrative {
    fn as_ref(&self) -> DocumentElementRef<'_>;
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized;

    fn element_uri(&self) -> Option<&DocumentElementUri>;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DocumentElement {
    SetSectionLevel(SectionLevel),
    UseModule(ModuleUri),

    Module {
        range: DocumentRange,
        module: ModuleUri,
        children: Box<[Self]>,
    },
    MathStructure {
        range: DocumentRange,
        structure: SymbolUri,
        children: Box<[Self]>,
    },
    Extension {
        range: DocumentRange,
        extension: SymbolUri,
        target: SymbolUri,
        children: Box<[Self]>,
    },
    Morphism {
        range: DocumentRange,
        morphism: SymbolUri,
        children: Box<[Self]>,
    },
    SymbolDeclaration(SymbolUri),
    ImportModule(ModuleUri),

    Section(Section),
    SkipSection(Box<[Self]>),
    Paragraph(LogicalParagraph),
    Problem(Problem),
    Slide {
        range: DocumentRange,
        uri: DocumentElementUri,
        children: Box<[Self]>,
    },
    DocumentReference {
        uri: DocumentElementUri,
        target: DocumentUri,
    },
    Notation {
        symbol: SymbolUri,
        uri: DocumentElementUri,
        notation: DataRef<Notation>,
    },
    VariableDeclaration(VariableDeclaration),
    VariableNotation {
        variable: DocumentElementUri,
        uri: DocumentElementUri,
        notation: DataRef<Notation>,
    },
    Definiendum {
        range: DocumentRange,
        uri: SymbolUri,
    },
    SymbolReference {
        range: DocumentRange,
        uri: SymbolUri,
        notation: Option<UriName>,
    },
    VariableReference {
        range: DocumentRange,
        uri: DocumentElementUri,
        notation: Option<UriName>,
    },
    Expr {
        uri: DocumentElementUri,
        term: Expr,
    },
}

impl crate::__private::Sealed for DocumentElement {}
impl super::Narrative for DocumentElement {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        self.element_uri().map(ftml_uris::NarrativeUriRef::Element)
    }
    fn children(&self) -> &[DocumentElement] {
        match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::VariableDeclaration(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Expr { .. } => &[],
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => children,
            Self::Section(s) => &s.children,
            Self::Paragraph(s) => &s.children,
            Self::Problem(s) => &s.children,
        }
    }
}
impl Ftml for DocumentElement {
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
        match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
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
            Self::Notation { symbol, uri, .. } => {
                let iri = uri.to_iri();
                RdfIterator::Not(
                    [
                        triple!(<(iri.clone())>: ulo:notation),
                        triple!(<(iri)> ulo:notation_for <(symbol.to_iri())>),
                    ]
                    .into_iter(),
                )
            }
            Self::VariableNotation { variable, uri, .. } => {
                let iri = uri.to_iri();
                RdfIterator::Not(
                    [
                        triple!(<(iri.clone())>: ulo:notation),
                        triple!(<(iri)> ulo:notation_for <(variable.to_iri())>),
                    ]
                    .into_iter(),
                )
            }
            Self::Expr { uri, term } => {
                let iri = uri.to_iri();
                let iri2 = iri.clone();
                RdfIterator::Term(
                    syms!(iri term).chain(std::iter::once(triple!(<(iri2)>: ulo:term))),
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DocumentElementRef<'d> {
    SetSectionLevel(SectionLevel),
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
        children: &'d [DocumentElement],
    },
    DocumentReference {
        uri: &'d DocumentElementUri,
        target: &'d DocumentUri,
    },
    Notation {
        symbol: &'d SymbolUri,
        uri: &'d DocumentElementUri,
        notation: DataRef<Notation>,
    },
    VariableDeclaration(&'d VariableDeclaration),
    VariableNotation {
        variable: &'d DocumentElementUri,
        uri: &'d DocumentElementUri,
        notation: DataRef<Notation>,
    },
    Definiendum {
        range: DocumentRange,
        uri: &'d SymbolUri,
    },
    SymbolReference {
        range: DocumentRange,
        uri: &'d SymbolUri,
        notation: Option<&'d UriName>,
    },
    VariableReference {
        range: DocumentRange,
        uri: &'d DocumentElementUri,
        notation: Option<&'d UriName>,
    },
    Expr {
        uri: &'d DocumentElementUri,
        term: &'d Expr,
    },
}

impl crate::__private::Sealed for DocumentElementRef<'_> {}
impl super::Narrative for DocumentElementRef<'_> {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        self.element_uri().map(ftml_uris::NarrativeUriRef::Element)
    }
    fn children(&self) -> &[DocumentElement] {
        match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
            | Self::SymbolDeclaration(_)
            | Self::ImportModule(_)
            | Self::VariableDeclaration(_)
            | Self::DocumentReference { .. }
            | Self::Definiendum { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Notation { .. }
            | Self::VariableNotation { .. }
            | Self::Expr { .. } => &[],
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::Slide { children, .. }
            | Self::SkipSection(children) => children,
            Self::Section(s) => &s.children,
            Self::Paragraph(s) => &s.children,
            Self::Problem(s) => &s.children,
        }
    }
}
impl crate::Ftml for DocumentElementRef<'_> {
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
        match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
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
            Self::Notation { symbol, uri, .. } => {
                let iri = uri.to_iri();
                RdfIterator::Not(
                    [
                        triple!(<(iri.clone())>: ulo:notation),
                        triple!(<(iri)> ulo:notation_for <(symbol.to_iri())>),
                    ]
                    .into_iter(),
                )
            }
            Self::VariableNotation { variable, uri, .. } => {
                let iri = uri.to_iri();
                RdfIterator::Not(
                    [
                        triple!(<(iri.clone())>: ulo:notation),
                        triple!(<(iri)> ulo:notation_for <(variable.to_iri())>),
                    ]
                    .into_iter(),
                )
            }
            Self::Expr { uri, term } => {
                let iri = uri.to_iri();
                let iri2 = iri.clone();
                RdfIterator::Term(
                    syms!(iri term).chain(std::iter::once(triple!(<(iri2)>: ulo:term))),
                )
            }
        }
    }
}

impl DocumentElement {
    #[must_use]
    pub fn opaque_children(&self) -> &[Self] {
        match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
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
            | Self::Expr { .. } => &[],
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
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
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
            | Self::Notation { uri, .. }
            | Self::VariableNotation { uri, .. }
            | Self::Expr { uri, .. } => uri,
        })
    }
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn as_ref(&self) -> DocumentElementRef<'_> {
        match self {
            Self::SetSectionLevel(l) => DocumentElementRef::SetSectionLevel(*l),
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
                children,
            } => DocumentElementRef::Slide {
                range: *range,
                uri,
                children,
            },
            Self::DocumentReference { uri, target } => {
                DocumentElementRef::DocumentReference { uri, target }
            }
            Self::Notation {
                symbol,
                uri,
                notation,
            } => DocumentElementRef::Notation {
                symbol,
                uri,
                notation: *notation,
            },
            Self::VariableDeclaration(v) => DocumentElementRef::VariableDeclaration(v),
            Self::VariableNotation {
                variable,
                uri,
                notation,
            } => DocumentElementRef::VariableNotation {
                variable,
                uri,
                notation: *notation,
            },
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
            Self::Expr { uri, term } => DocumentElementRef::Expr { uri, term },
        }
    }
}

impl<'e> DocumentElementRef<'e> {
    #[must_use]
    pub const fn opaque_children(&self) -> &[DocumentElement] {
        match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
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
            | Self::Expr { .. } => &[],
            Self::Module { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::Morphism { children, .. }
            | Self::SkipSection(children) => children,
        }
    }
    #[must_use]
    pub const fn element_uri(self) -> Option<&'e DocumentElementUri> {
        Some(match self {
            Self::SetSectionLevel(_)
            | Self::UseModule(_)
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
            | Self::Notation { uri, .. }
            | Self::VariableNotation { uri, .. }
            | Self::Expr { uri, .. } => uri,
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
    T: Iterator<Item = ulo::rdf_types::Triple> + 'e,
> {
    #[allow(dead_code)]
    Empty(DocumentElementRef<'e>),
    Children(Box<dyn Iterator<Item = ulo::rdf_types::Triple> + 'e>),
    Section(S),
    Paragraph(Pa),
    Problem(Pr),
    Var(V),
    Not(std::array::IntoIter<ulo::rdf_types::Triple, 2>),
    Term(T),
}
#[cfg(feature = "rdf")]
impl<
    'e,
    S: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    Pa: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    Pr: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    V: Iterator<Item = ulo::rdf_types::Triple> + 'e,
    T: Iterator<Item = ulo::rdf_types::Triple> + 'e,
> Iterator for RdfIterator<'e, S, Pa, Pr, V, T>
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
            Self::Not(n) => n.next(),
            Self::Term(t) => t.next(),
        }
    }
}
