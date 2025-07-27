use ftml_ontology::{
    Ftml,
    domain::declarations::{AnyDeclaration, symbols::SymbolData},
    narrative::{
        DocumentRange,
        documents::{DocumentCounter, DocumentStyle},
        elements::{DocumentElement, sections::SectionLevel},
    },
    terms::{Argument, ArgumentMode, Term, Variable},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Language, ModuleUri, SymbolUri, UriName};

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
        notation: Option<UriName>,
    },
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
    OMA {
        head: VarOrSym,
        notation: Option<UriName>,
        uri: Option<DocumentElementUri>,
    },
    Argument(ArgumentPosition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseFtmlElement {
    Module,
    SymbolDeclaration,
    Invisible,
    Section,
    SectionTitle,
    SkipSection,
    SymbolReference,
    OMA,
    Argument,
}

#[derive(Debug, Clone)]
pub enum OpenDomainElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<AnyDeclaration>,
    },
    SymbolDeclaration {
        uri: SymbolUri,
        data: Box<SymbolData>,
    },
    SymbolReference {
        uri: SymbolUri,
        notation: Option<UriName>,
    },
    OMA {
        head: VarOrSym,
        notation: Option<UriName>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenArgument>,
    },
    Argument {
        position: ArgumentPosition,
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenNarrativeElement {
    Module {
        uri: ModuleUri,
        children: Vec<DocumentElement>,
    },
    Section {
        uri: DocumentElementUri,
        title: Option<DocumentRange>,
        children: Vec<DocumentElement>,
    },
    SkipSection {
        children: Vec<DocumentElement>,
    },
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
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Split<N: FtmlNode> {
    Meta(MetaDatum),
    Open {
        domain: Option<OpenDomainElement<N>>,
        narrative: Option<OpenNarrativeElement>,
    },
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum VarOrSym {
    S(SymbolUri),
    V(Variable),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OpenArgument {
    None,
    Simple(Argument),
    Sequence(Vec<Option<Term>>),
}
impl OpenArgument {
    pub fn close(self) -> Option<Argument> {
        use either::Either::Right;
        match self {
            Self::Simple(a) => Some(a),
            Self::Sequence(v) if v.iter().all(Option::is_some) => {
                Some(Argument::Sequence(Right(v.into_iter().flatten().collect())))
            }
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
        let idx = position.index() as usize;
        while args.len() <= idx {
            args.push(Self::None);
        }
        let arg = &mut args[idx];
        match (arg, position) {
            (r @ Self::None, ArgumentPosition::Simple(_, m)) => match m {
                ArgumentMode::Simple => *r = Self::Simple(Argument::Simple(term)),
                ArgumentMode::Sequence => *r = Self::Simple(Argument::Sequence(Left(term))),
                m => return Err(FtmlExtractionError::MismatchedArgument(m)),
            },
            (r @ Self::None, ArgumentPosition::Sequence { sequence_index, .. }) => {
                let mut v = (0..sequence_index as usize)
                    .map(|_| None)
                    .collect::<Vec<_>>();
                v.push(Some(term));
                *r = Self::Sequence(v);
            }
            (Self::Sequence(v), ArgumentPosition::Sequence { sequence_index, .. }) => {
                let idx = sequence_index as usize;
                while v.len() <= idx + 1 {
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ArgumentPosition {
    Simple(u8, ArgumentMode),
    Sequence {
        argument_number: u8,
        sequence_index: u8,
        mode: ArgumentMode,
    },
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
            Self::Simple(u, _) => *u,
            Self::Sequence {
                argument_number, ..
            } => *argument_number,
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
                (Some(a), Some(b)) if a < 256 && b < 256 => Right((a as u8, b as u8)),
                _ => return None,
            }
        } else if idx.len() == 1 {
            let a = idx
                .chars()
                .next()
                .unwrap_or_else(|| unreachable!())
                .to_digit(10)?;
            if a < 256 {
                Left(a as u8)
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
            Self::InputRef {
                target: uri,
                uri: id,
            } => Split::Meta(MetaDatum::InputRef {
                target: uri,
                uri: id,
            }),
            Self::Style(s) => Split::Meta(MetaDatum::Style(s)),
            Self::Counter(c) => Split::Meta(MetaDatum::Counter(c)),
            Self::SetSectionLevel(_) | Self::None | Self::SectionTitle | Self::Comp => Split::None,
        }
    }
}
