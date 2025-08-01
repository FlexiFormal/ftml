use std::{hint::unreachable_unchecked, num::NonZeroU8};

use either::Either::{self, Left, Right};
use ftml_ontology::{
    domain::declarations::{Declaration, symbols::SymbolData},
    narrative::{
        DocumentRange,
        documents::{DocumentCounter, DocumentStyle},
        elements::{
            DocumentElement,
            notations::{NotationComponent, NotationNode},
            sections::SectionLevel,
            variables::VariableData,
        },
    },
    terms::{Argument, ArgumentMode, BoundArgument, Term, Variable},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, Language, LeafUri, ModuleUri, SymbolUri};
use smallvec::SmallVec;

use crate::extraction::{FtmlExtractionError, nodes::FtmlNode};

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
    SkipSection,
    Comp,
    SymbolReference {
        uri: SymbolUri,
        notation: Option<Id>,
    },
    VariableReference {
        var: Variable,
        notation: Option<Id>,
    },
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
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
    Notation {
        uri: DocumentElementUri,
        id: Option<Id>,
        head: VarOrSym,
        prec: isize,
        argprecs: SmallVec<isize, 9>,
    },
    Argument(ArgumentPosition),
    NotationArg(ArgumentPosition),
    Type,
    Definiens,
    NotationComp,
    ArgSep,
    CurrentSectionLevel(bool),
    ImportModule(ModuleUri),
    UseModule(ModuleUri),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseFtmlElement {
    Module,
    SymbolDeclaration,
    VariableDeclaration,
    Invisible,
    Section,
    SectionTitle,
    SkipSection,
    SymbolReference,
    VariableReference,
    OMA,
    OMBIND,
    Argument,
    NotationArg,
    Type,
    Definiens,
    Notation,
    CompInNotation,
    MainCompInNotation,
    NotationComp,
    NotationOpComp,
    ArgSep,
    DocTitle,
    Comp,
}

#[derive(Debug, Clone)]
pub enum OpenDomainElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<Declaration>,
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
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenArgument>,
    },
    OMBIND {
        head: VarOrSym,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenBoundArgument>,
    },
    Argument {
        position: ArgumentPosition,
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    Type {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    Definiens {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
    Comp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenNarrativeElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        children: Vec<DocumentElement>,
    },
    VariableDeclaration {
        uri: DocumentElementUri,
        data: Box<VariableData>,
    },
    Section {
        uri: DocumentElementUri,
        title: Option<DocumentRange>,
        children: Vec<DocumentElement>,
    },
    SkipSection {
        children: Vec<DocumentElement>,
    },
    Notation {
        uri: DocumentElementUri,
        id: Option<Id>,
        head: VarOrSym,
        prec: isize,
        argprecs: SmallVec<isize, 9>,
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
    NotationArg(ArgumentPosition),
    Invisible,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaDatum {
    Style(DocumentStyle),
    Counter(DocumentCounter),
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
    SetSectionLevel(SectionLevel),
    ImportModule(ModuleUri),
    UseModule(ModuleUri),
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Split<N: FtmlNode> {
    Meta(MetaDatum),
    Open {
        domain: Option<OpenDomainElement<N>>,
        narrative: Option<OpenNarrativeElement<N>>,
    },
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum VarOrSym {
    S(SymbolUri),
    V(Variable),
}
impl std::fmt::Display for VarOrSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S(s) => s.fmt(f),
            Self::V(v) => v.fmt(f),
        }
    }
}
impl From<LeafUri> for VarOrSym {
    fn from(value: LeafUri) -> Self {
        match value {
            LeafUri::Symbol(s) => Self::S(s),
            LeafUri::Element(e) => Self::V(Variable::Ref {
                declaration: e,
                is_sequence: None,
            }),
        }
    }
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
                term: Term::Var(v),
                should_be_var: true,
            } => Some(BoundArgument::Bound(v)),
            Self::Simple { term, .. } => Some(BoundArgument::Simple(term)),
            Self::Sequence {
                terms: Left(Term::Var(v)),
                should_be_var: true,
            } => Some(BoundArgument::BoundSeq(Left(v))),
            Self::Sequence {
                terms: Right(v),
                should_be_var: true,
            } if v.iter().all(|t| matches!(t, Some(Term::Var(_)))) => {
                Some(BoundArgument::BoundSeq(Right(
                    v.into_iter()
                        .map(|v| {
                            let Some(Term::Var(v)) = v else {
                                // SAFETY: iter.all() matches above
                                unsafe { unreachable_unchecked() }
                            };
                            v
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
            Self::Simple(..) => ArgumentMode::Simple,
            Self::Sequence { .. } => ArgumentMode::Sequence,
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

impl From<SymbolUri> for VarOrSym {
    #[inline]
    fn from(value: SymbolUri) -> Self {
        Self::S(value)
    }
}
impl From<Variable> for VarOrSym {
    #[inline]
    fn from(value: Variable) -> Self {
        Self::V(value)
    }
}

/*
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PreVar {
    Resolved(DocumentElementUri),
    Unresolved(UriName),
}

impl std::fmt::Display for PreVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolved(declaration) => std::fmt::Display::fmt(declaration, f),
            Self::Unresolved(name) => std::fmt::Display::fmt(name, f),
        }
    }
}

impl PreVar {
    /*
    fn resolve<State: FtmlExtractor>(self, state: &State) -> Term {
        Term::Var(match self {
            Self::Resolved(declaration) => Variable::Ref {
                declaration,
                is_sequence: None,
            },
            // TODO can we know is_sequence yet?
            Self::Unresolved(name) => state.resolve_variable_name(name),
        })
    }
     */
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &UriName {
        match self {
            Self::Resolved(declaration) => declaration.name(),
            Self::Unresolved(name) => name,
        }
    }
}
 */

impl OpenFtmlElement {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub(crate) fn split<N: FtmlNode>(self, node: &N) -> Split<N> {
        match self {
            Self::Module {
                uri,
                meta,
                signature,
            } => Split::Open {
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
            Self::SymbolDeclaration { uri, data } => Split::Open {
                domain: Some(OpenDomainElement::SymbolDeclaration { uri, data }),
                narrative: None,
            },
            Self::VariableDeclaration { uri, data } => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::VariableDeclaration { uri, data }),
            },
            Self::Section(uri) => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Section {
                    uri,
                    title: None,
                    children: Vec::new(),
                }),
            },
            Self::SkipSection => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::SkipSection {
                    children: Vec::new(),
                }),
            },
            Self::SymbolReference { uri, notation } => Split::Open {
                domain: Some(OpenDomainElement::SymbolReference { uri, notation }),
                narrative: None,
            },
            Self::VariableReference { var, notation } => Split::Open {
                domain: Some(OpenDomainElement::VariableReference { var, notation }),
                narrative: None,
            },
            Self::OMA {
                head,
                notation,
                uri,
            } => Split::Open {
                domain: Some(OpenDomainElement::OMA {
                    head,
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
            } => Split::Open {
                domain: Some(OpenDomainElement::OMBIND {
                    head,
                    notation,
                    uri,
                    arguments: Vec::new(),
                }),
                narrative: None,
            },
            Self::Invisible => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Invisible),
            },
            Self::Argument(a) => Split::Open {
                domain: Some(OpenDomainElement::Argument {
                    position: a,
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::NotationArg(a) => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::NotationArg(a)),
            },
            Self::Type => Split::Open {
                domain: Some(OpenDomainElement::Type {
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::Definiens => Split::Open {
                domain: Some(OpenDomainElement::Definiens {
                    terms: Vec::new(),
                    node: node.clone(),
                }),
                narrative: None,
            },
            Self::Notation {
                id,
                uri,
                head,
                prec,
                argprecs,
            } => Split::Open {
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
            Self::NotationComp => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::NotationComp {
                    node: node.clone(),
                    components: Vec::new(),
                }),
            },
            Self::ArgSep => Split::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ArgSep {
                    node: node.clone(),
                    components: Vec::new(),
                }),
            },
            Self::Comp => Split::Open {
                domain: Some(OpenDomainElement::Comp),
                narrative: None,
            },
            Self::InputRef {
                target: uri,
                uri: id,
            } => Split::Meta(MetaDatum::InputRef {
                target: uri,
                uri: id,
            }),
            Self::Style(s) => Split::Meta(MetaDatum::Style(s)),
            Self::Counter(c) => Split::Meta(MetaDatum::Counter(c)),
            Self::SetSectionLevel(lvl) => Split::Meta(MetaDatum::SetSectionLevel(lvl)),
            Self::ImportModule(uri) => Split::Meta(MetaDatum::ImportModule(uri)),
            Self::UseModule(uri) => Split::Meta(MetaDatum::UseModule(uri)),
            Self::None | Self::SectionTitle | Self::CurrentSectionLevel(_) => Split::None,
        }
    }
}
