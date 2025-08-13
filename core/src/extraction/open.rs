use std::{hint::unreachable_unchecked, num::NonZeroU8};

use crate::extraction::{FtmlExtractionError, nodes::FtmlNode};
use either::Either::{self, Left, Right};
use ftml_ontology::{
    domain::declarations::{Declaration, structures::StructureDeclaration, symbols::SymbolData},
    narrative::{
        DocumentRange,
        documents::{DocumentCounter, DocumentStyle},
        elements::{
            DocumentElement,
            notations::{NotationComponent, NotationNode},
            paragraphs::{ParagraphFormatting, ParagraphKind},
            sections::SectionLevel,
            variables::VariableData,
        },
    },
    terms::{Argument, ArgumentMode, BoundArgument, Term, VarOrSym, Variable},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, Language, ModuleUri, SymbolUri, UriName};

pub use crate::keys::OpenFtmlElement;

/*
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenFtmlElement {
    None,
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
    },
    SymbolDeclaration {
        uri: SymbolUri,
        data: Box<SymbolData>,
    },
    MathStructure {
        uri: SymbolUri,
        macroname: Option<Id>,
    },
    VariableDeclaration {
        uri: DocumentElementUri,
        data: Box<VariableData>,
    },
    Section(DocumentElementUri),
    SetSectionLevel(SectionLevel),
    Style(DocumentStyle),
    Counter(DocumentCounter),
    Invisible,
    SectionTitle,
    ParagraphTitle,
    SkipSection,
    Comp,
    DefComp,
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
    IfInputref(bool),
    SymbolReference {
        uri: SymbolUri,
        notation: Option<Id>,
    },
    VariableReference {
        var: Variable,
        notation: Option<Id>,
    },
    OMA {
        head: VarOrSym,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
    },
    OMBIND {
        head: VarOrSym,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
    },
    ComplexTerm {
        head: VarOrSym,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
    },
    OML {
        name: UriName,
    },
    Notation {
        uri: DocumentElementUri,
        id: Option<Id>,
        head: VarOrSym,
        prec: i64,
        argprecs: Vec<i64>,
    },
    Paragraph {
        uri: DocumentElementUri,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        fors: Vec<(SymbolUri, Option<Term>)>,
    },
    Argument(ArgumentPosition),
    NotationArg(ArgumentPosition),
    Type,
    ReturnType,
    Definiens(Option<SymbolUri>),
    NotationComp,
    ArgSep,
    CurrentSectionLevel(bool),
    ImportModule(ModuleUri),
    UseModule(ModuleUri),
    Definiendum(SymbolUri),
    HeadTerm,
}
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseFtmlElement {
    Module,
    SymbolDeclaration,
    VariableDeclaration,
    Invisible,
    Section,
    SectionTitle,
    ParagraphTitle,
    SkipSection,
    SymbolReference,
    VariableReference,
    OMA,
    OMBIND,
    Argument,
    NotationArg,
    Type,
    ReturnType,
    Definiens,
    Notation,
    CompInNotation,
    MainCompInNotation,
    NotationComp,
    NotationOpComp,
    ArgSep,
    DocTitle,
    Comp,
    DefComp,
    Paragraph,
    Definiendum,
    MathStructure,
    ComplexTerm,
    HeadTerm,
    OML,
}

#[derive(Debug, Clone)]
pub enum OpenDomainElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<Declaration>,
    },
    MathStructure {
        uri: SymbolUri,
        macroname: Option<Id>,
        children: Vec<StructureDeclaration>,
    },
    SymbolDeclaration {
        uri: SymbolUri,
        data: Box<SymbolData>,
    },
    SymbolReference {
        uri: SymbolUri,
        notation: Option<Id>,
    },
    VariableReference {
        var: Variable,
        notation: Option<Id>,
    },
    OMA {
        head: VarOrSym,
        head_term: Option<Term>,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenArgument>,
    },
    OMBIND {
        head: VarOrSym,
        head_term: Option<Term>,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenBoundArgument>,
    },
    OML {
        name: UriName,
    },
    ComplexTerm {
        head: VarOrSym,
        head_term: Option<Term>,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
    },
    Argument {
        position: ArgumentPosition,
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    HeadTerm {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    Type {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    ReturnType {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    Definiens {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
        uri: Option<SymbolUri>,
    },
    Comp,
    DefComp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenNarrativeElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        children: Vec<DocumentElement>,
    },
    MathStructure {
        uri: SymbolUri,
        children: Vec<DocumentElement>,
    },
    VariableDeclaration {
        uri: DocumentElementUri,
        data: Box<VariableData>,
    },
    Section {
        uri: DocumentElementUri,
        title: Option<Box<str>>,
        children: Vec<DocumentElement>,
    },
    SkipSection {
        children: Vec<DocumentElement>,
    },
    Notation {
        uri: DocumentElementUri,
        id: Option<Id>,
        head: VarOrSym,
        prec: i64,
        argprecs: Vec<i64>,
        component: Option<NotationComponent>,
        op: Option<NotationNode>,
    },
    NotationComp {
        node: N,
        components: Vec<(NotationComponent, crate::NodePath)>,
    },
    ArgSep {
        node: N,
        components: Vec<(NotationComponent, crate::NodePath)>,
    },
    Paragraph {
        uri: DocumentElementUri,
        kind: ParagraphKind,
        fors: Vec<(SymbolUri, Option<Term>)>,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        children: Vec<DocumentElement>,
        title: Option<Box<str>>,
    },
    NotationArg(ArgumentPosition),
    Invisible,
    Definiendum(SymbolUri),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaDatum {
    Style(DocumentStyle),
    Counter(DocumentCounter),
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
    IfInputref(bool),
    SetSectionLevel(SectionLevel),
    ImportModule(ModuleUri),
    UseModule(ModuleUri),
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AnyOpen<N: FtmlNode> {
    Meta(MetaDatum),
    Open {
        domain: Option<OpenDomainElement<N>>,
        narrative: Option<OpenNarrativeElement<N>>,
    },
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OpenArgument {
    None,
    Simple(Term),
    Sequence(Either<Term, Vec<Option<Term>>>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OpenBoundArgument {
    None,
    Simple {
        term: Term,
        should_be_var: bool,
    },
    Sequence {
        terms: Either<Term, Vec<Option<Term>>>,
        should_be_var: bool,
    },
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ArgumentPosition {
    Simple(NonZeroU8, ArgumentMode),
    Sequence {
        argument_number: NonZeroU8,
        sequence_index: NonZeroU8,
        mode: ArgumentMode,
    },
}

impl OpenArgument {
    pub fn close(self) -> Option<Argument> {
        use either::Either::Right;
        match self {
            Self::Simple(a) => Some(Argument::Simple(a)),
            Self::Sequence(Right(v)) if v.iter().all(Option::is_some) => {
                Some(Argument::Sequence(Right(
                    v.into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )))
            }
            Self::Sequence(Left(t)) => Some(Argument::Sequence(Left(t))),
            Self::None | Self::Sequence(_) => None,
        }
    }

    /// ### Errors
    pub fn set(
        args: &mut Vec<Self>,
        position: ArgumentPosition,
        term: Term,
    ) -> Result<(), FtmlExtractionError> {
        use either::Either::Left;
        tracing::trace!("Setting {position:?} in {args:?} to {term:?}");
        let idx = position.index() as usize;
        while args.len() <= idx {
            args.push(Self::None);
        }
        let arg = &mut args[idx];
        match (arg, position) {
            (r @ Self::None, ArgumentPosition::Simple(_, m)) => match m {
                ArgumentMode::Simple => *r = Self::Simple(term),
                ArgumentMode::Sequence => *r = Self::Sequence(Left(term)),
                m => return Err(FtmlExtractionError::MismatchedArgument(m)),
            },
            (r @ Self::None, ArgumentPosition::Sequence { sequence_index, .. }) => {
                let mut v = (0..(sequence_index.get() - 1) as usize)
                    .map(|_| None)
                    .collect::<Vec<_>>();
                v.push(Some(term));
                *r = Self::Sequence(Right(v));
            }
            (Self::Sequence(Right(v)), ArgumentPosition::Sequence { sequence_index, .. }) => {
                let idx = (sequence_index.get() - 1) as usize;
                while v.len() <= idx {
                    v.push(None);
                }
                if v[idx].is_some() {
                    return Err(FtmlExtractionError::MismatchedArgument(position.mode()));
                }
                v[idx] = Some(term);
            }
            _ => return Err(FtmlExtractionError::MismatchedArgument(position.mode())),
        }
        Ok(())
    }
}

impl OpenBoundArgument {
    pub fn close(self) -> Option<BoundArgument> {
        use either::Either::Right;
        match self {
            Self::Simple {
                term: Term::Var { variable: v, .. },
                should_be_var: true,
            } => Some(BoundArgument::Bound(v)),
            Self::Simple { term, .. } => Some(BoundArgument::Simple(term)),
            Self::Sequence {
                terms: Left(Term::Var { variable: v, .. }),
                should_be_var: true,
            } => Some(BoundArgument::BoundSeq(Left(v))),
            Self::Sequence {
                terms: Right(v),
                should_be_var: true,
            } if v.iter().all(|t| matches!(t, Some(Term::Var { .. }))) => {
                Some(BoundArgument::BoundSeq(Right(
                    v.into_iter()
                        .map(|v| {
                            let Some(Term::Var { variable, .. }) = v else {
                                // SAFETY: iter.all() matches above
                                unsafe { unreachable_unchecked() }
                            };
                            variable
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )))
            }
            Self::Sequence {
                terms: Right(v), ..
            } if v.iter().all(Option::is_some) => Some(BoundArgument::Sequence(Right(
                v.into_iter().flatten().collect(),
            ))),
            Self::Sequence { terms: Left(a), .. } => Some(BoundArgument::Sequence(Left(a))),
            Self::None | Self::Sequence { .. } => None,
        }
    }

    /// ### Errors
    pub fn set(
        args: &mut Vec<Self>,
        position: ArgumentPosition,
        term: Term,
    ) -> Result<(), FtmlExtractionError> {
        use either::Either::Left;
        tracing::trace!("Setting {position:?} in {args:?} to {term:?}");
        let idx = position.index() as usize;
        while args.len() <= idx {
            args.push(Self::None);
        }
        let arg = &mut args[idx];
        match (arg, position) {
            (r @ Self::None, ArgumentPosition::Simple(_, m)) => match m {
                ArgumentMode::Simple => {
                    *r = Self::Simple {
                        term,
                        should_be_var: false,
                    }
                }
                ArgumentMode::Sequence => {
                    *r = Self::Sequence {
                        terms: Left(term),
                        should_be_var: false,
                    }
                }
                ArgumentMode::BoundVariable => {
                    *r = Self::Simple {
                        term,
                        should_be_var: true,
                    }
                }
                ArgumentMode::BoundVariableSequence => {
                    *r = Self::Sequence {
                        terms: Left(term),
                        should_be_var: true,
                    }
                }
            },
            (
                r @ Self::None,
                ArgumentPosition::Sequence {
                    sequence_index,
                    mode,
                    ..
                },
            ) => {
                let mut v = (0..(sequence_index.get() - 1) as usize)
                    .map(|_| None)
                    .collect::<Vec<_>>();
                v.push(Some(term));
                let should_be_var = matches!(
                    mode,
                    ArgumentMode::BoundVariable | ArgumentMode::BoundVariableSequence
                );
                *r = Self::Sequence {
                    terms: Right(v),
                    should_be_var,
                };
            }
            (
                Self::Sequence {
                    terms: Right(v),
                    should_be_var: _,
                },
                ArgumentPosition::Sequence {
                    sequence_index,
                    mode: ArgumentMode::Sequence | ArgumentMode::BoundVariableSequence,
                    ..
                },
            ) => {
                let idx = (sequence_index.get() - 1) as usize;
                while v.len() <= idx {
                    v.push(None);
                }
                if v[idx].is_some() {
                    return Err(FtmlExtractionError::MismatchedArgument(position.mode()));
                }
                v[idx] = Some(term);
            }
            _ => return Err(FtmlExtractionError::MismatchedArgument(position.mode())),
        }
        Ok(())
    }
}

impl ArgumentPosition {
    #[inline]
    #[must_use]
    pub const fn mode(&self) -> ArgumentMode {
        match self {
            Self::Simple(_, m) => *m,
            Self::Sequence { mode, .. } => *mode,
        }
    }
    #[inline]
    #[must_use]
    pub const fn index(&self) -> u8 {
        match self {
            Self::Simple(u, _) => u.get() - 1,
            Self::Sequence {
                argument_number, ..
            } => argument_number.get() - 1,
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn from_strs(idx: &str, mode: Option<ArgumentMode>) -> Option<Self> {
        use either::Either::{Left, Right};
        use std::str::FromStr;
        let index = if idx.chars().count() > 1 {
            let a = idx
                .chars()
                .next()
                .unwrap_or_else(|| unreachable!())
                .to_digit(10);
            let b = u32::from_str(&idx[1..]).ok();
            match (a, b) {
                (Some(a), Some(b)) if a < 256 && b < 256 => {
                    Right(((a as u8).try_into().ok()?, (b as u8).try_into().ok()?))
                }
                _ => return None,
            }
        } else if idx.len() == 1 {
            let a = idx
                .chars()
                .next()
                .unwrap_or_else(|| unreachable!())
                .to_digit(10)?;
            if a < 256 {
                Left((a as u8).try_into().ok()?)
            } else {
                return None;
            }
        } else {
            return None;
        };
        Some(match index {
            Left(i) => Self::Simple(i, mode.unwrap_or_default()),
            Right((a, b)) => Self::Sequence {
                argument_number: a,
                sequence_index: b,
                mode: mode.unwrap_or_default(),
            },
        })
    }
}

impl OpenFtmlElement {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub(crate) fn split<N: FtmlNode>(self, node: &N) -> AnyOpen<N> {
        match self {
            Self::Module {
                uri,
                meta,
                signature,
            } => AnyOpen::Open {
                domain: Some(OpenDomainElement::Module {
                    uri: uri.clone(),
                    meta,
                    signature,
                    children: Vec::new(),
                }),
                narrative: Some(OpenNarrativeElement::Module {
                    uri,
                    children: Vec::new(),
                }),
            },
            Self::MathStructure { uri, macroname } => AnyOpen::Open {
                domain: Some(OpenDomainElement::MathStructure {
                    uri: uri.clone(),
                    macroname,
                    children: Vec::new(),
                }),
                narrative: Some(OpenNarrativeElement::MathStructure {
                    uri,
                    children: Vec::new(),
                }),
            },
            Self::SymbolDeclaration { uri, data } => AnyOpen::Open {
                domain: Some(OpenDomainElement::SymbolDeclaration { uri, data }),
                narrative: None,
            },
            Self::VariableDeclaration { uri, data } => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::VariableDeclaration { uri, data }),
            },
            Self::Section(uri) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Section {
                    uri,
                    title: None,
                    children: Vec::new(),
                }),
            },
            Self::SkipSection => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::SkipSection {
                    children: Vec::new(),
                }),
            },
            Self::SymbolReference { uri, notation } => AnyOpen::Open {
                domain: Some(OpenDomainElement::SymbolReference { uri, notation }),
                narrative: None,
            },
            Self::VariableReference { var, notation } => AnyOpen::Open {
                domain: Some(OpenDomainElement::VariableReference { var, notation }),
                narrative: None,
            },
            Self::OMA {
                head,
                notation,
                uri,
            } => AnyOpen::Open {
                domain: Some(OpenDomainElement::OMA {
                    head,
                    head_term: None,
                    notation,
                    uri,
                    arguments: Vec::new(),
                }),
                narrative: None,
            },
            Self::OMBIND {
                head,
                notation,
                uri,
            } => AnyOpen::Open {
                domain: Some(OpenDomainElement::OMBIND {
                    head,
                    head_term: None,
                    notation,
                    uri,
                    arguments: Vec::new(),
                }),
                narrative: None,
            },
            Self::OML { name } => AnyOpen::Open {
                domain: Some(OpenDomainElement::OML { name }),
                narrative: None,
            },
            Self::ComplexTerm {
                head,
                notation,
                uri,
            } => AnyOpen::Open {
                domain: Some(OpenDomainElement::ComplexTerm {
                    head,
                    head_term: None,
                    notation,
                    uri,
                }),
                narrative: None,
            },
            Self::Invisible => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Invisible),
            },
            Self::Argument(a) => AnyOpen::Open {
                domain: Some(OpenDomainElement::Argument {
                    position: a,
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::HeadTerm => AnyOpen::Open {
                domain: Some(OpenDomainElement::HeadTerm {
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::NotationArg(a) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::NotationArg(a)),
            },
            Self::Type => AnyOpen::Open {
                domain: Some(OpenDomainElement::Type {
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::ReturnType => AnyOpen::Open {
                domain: Some(OpenDomainElement::ReturnType {
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::Definiens(uri) => AnyOpen::Open {
                domain: Some(OpenDomainElement::Definiens {
                    terms: Vec::new(),
                    node: node.clone(),
                    uri,
                }),
                narrative: None,
            },
            Self::Notation {
                id,
                uri,
                head,
                prec,
                argprecs,
            } => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Notation {
                    id,
                    uri,
                    head,
                    prec,
                    argprecs,
                    component: None,
                    op: None,
                }),
            },
            Self::Paragraph {
                uri,
                kind,
                formatting,
                styles,
                fors,
            } => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Paragraph {
                    uri,
                    kind,
                    fors,
                    formatting,
                    styles,
                    title: None,
                    children: Vec::new(),
                }),
            },
            Self::Definiendum(s) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Definiendum(s)),
            },
            Self::NotationComp => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::NotationComp {
                    node: node.clone(),
                    components: Vec::new(),
                }),
            },
            Self::ArgSep => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ArgSep {
                    node: node.clone(),
                    components: Vec::new(),
                }),
            },
            Self::Comp => AnyOpen::Open {
                domain: Some(OpenDomainElement::Comp),
                narrative: None,
            },
            Self::DefComp => AnyOpen::Open {
                domain: Some(OpenDomainElement::DefComp),
                narrative: None,
            },
            Self::InputRef {
                target: uri,
                uri: id,
            } => AnyOpen::Meta(MetaDatum::InputRef {
                target: uri,
                uri: id,
            }),
            Self::IfInputref(b) => AnyOpen::Meta(MetaDatum::IfInputref(b)),
            Self::Style(s) => AnyOpen::Meta(MetaDatum::Style(s)),
            Self::Counter(c) => AnyOpen::Meta(MetaDatum::Counter(c)),
            Self::SetSectionLevel(lvl) => AnyOpen::Meta(MetaDatum::SetSectionLevel(lvl)),
            Self::ImportModule(uri) => AnyOpen::Meta(MetaDatum::ImportModule(uri)),
            Self::UseModule(uri) => AnyOpen::Meta(MetaDatum::UseModule(uri)),
            Self::None
            | Self::SectionTitle
            | Self::ParagraphTitle
            | Self::CurrentSectionLevel(_) => AnyOpen::None,
        }
    }
}
