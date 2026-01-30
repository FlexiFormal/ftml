use crate::extraction::{FtmlExtractionError, nodes::FtmlNode};
pub use crate::keys::OpenFtmlElement;
use either::Either::{self, Left, Right};
use ftml_ontology::{
    domain::declarations::{
        Declaration, morphisms::Assignment, structures::StructureDeclaration, symbols::SymbolData,
    },
    narrative::{
        DataRef,
        documents::{DocumentCounter, DocumentKind, DocumentStyle},
        elements::{
            DocumentElement,
            notations::{NotationComponent, NotationNode},
            paragraphs::{ParagraphFormatting, ParagraphKind},
            problems::{
                AnswerClass, AnswerKind, Choice, ChoiceBlockStyle, CognitiveDimension,
                FillInSolOption, GradingNote, SolutionData,
            },
            sections::SectionLevel,
            variables::VariableData,
        },
    },
    terms::{
        Argument, ArgumentMode, BoundArgument, ComponentVar, MaybeSequence, Term, VarOrSym,
        Variable,
    },
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, Id, Language, ModuleUri, SimpleUriName, SymbolUri, UriName,
};
use std::{hint::unreachable_unchecked, num::NonZeroU8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseFtmlElement {
    Module,
    SymbolDeclaration,
    VariableDeclaration,
    Invisible,
    Section,
    SectionTitle,
    ParagraphTitle,
    SlideTitle,
    ProblemTitle,
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
    Morphism,
    Assign,
    Slide,
    Problem,
    Solution,
    FillinSol,
    ProblemHint,
    ProblemExNote,
    ProblemGradingNote,
    AnswerClass,
    ChoiceBlock,
    ProblemChoice,
    ProblemChoiceVerdict,
    ProblemChoiceFeedback,
    ArgTypes,
    FillinSolCase,
    Rule,
}

#[derive(Debug, Clone)]
pub enum OpenDomainElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<Declaration>,
    },
    Morphism {
        uri: SymbolUri,
        domain: ModuleUri,
        total: bool,
        children: Vec<Assignment>,
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
        tp: Option<Term>,
        df: Option<Term>,
    },
    InferenceRule {
        rule: Id,
        parameters: Vec<Term>,
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
    ArgTypes(Vec<Term>),
    Definiens {
        terms: Vec<(Term, crate::NodePath)>,
        node: N,
        uri: Option<SymbolUri>,
    },
    Comp,
    DefComp,
    Assign {
        source: SymbolUri,
        refined_type: Option<Term>,
        definiens: Option<Term>,
    },
}

#[derive(Debug, Clone)]
pub enum OpenNarrativeElement<N: FtmlNode> {
    Module {
        uri: ModuleUri,
        children: Vec<DocumentElement>,
    },
    MathStructure {
        uri: SymbolUri,
        children: Vec<DocumentElement>,
    },
    Morphism {
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
    Problem {
        uri: DocumentElementUri,
        children: Vec<DocumentElement>,
        title: Option<Box<str>>,
        sub_problem: bool,
        autogradable: bool,
        points: Option<f32>,
        minutes: Option<f32>,
        solutions: Vec<SolutionData>,
        gnotes: Vec<DataRef<GradingNote>>,
        hints: Vec<DataRef<Box<str>>>,
        notes: Vec<DataRef<Box<str>>>,
        styles: Box<[Id]>,
        preconditions: Vec<(CognitiveDimension, SymbolUri)>,
        objectives: Vec<(CognitiveDimension, SymbolUri)>,
    },
    Solution(Option<Id>),
    NotationArg(ArgumentPosition),
    Invisible,
    Definiendum(SymbolUri),
    Slide {
        uri: DocumentElementUri,
        children: Vec<DocumentElement>,
        title: Option<Box<str>>,
    },
    FillinSol {
        width: Option<f32>,
        cases: Vec<FillInSolOption>,
        nodes: Vec<N>,
    },
    ProblemHint,
    ProblemExNote,
    ProblemGradingNote(Vec<AnswerClass>),
    AnswerClass {
        id: Id,
        kind: AnswerKind,
        feedback: Box<str>,
        nodes: Vec<N>,
    },
    ChoiceBlock {
        styles: Box<[Id]>,
        block_style: ChoiceBlockStyle,
        multiple: bool,
        choices: Vec<Choice>,
    },
    ProblemChoice {
        correct: bool,
        verdict: Option<Box<str>>,
        feedback: Box<str>,
        nodes: Vec<N>,
    },
    ProblemChoiceVerdict,
    ProblemChoiceFeedback,
    FillinSolCase(FillInSolOption),
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
    Rename {
        source: SymbolUri,
        name: Option<SimpleUriName>,
        macroname: Option<Id>,
    },
    DocumentKind(DocumentKind),
    DocumentUri(DocumentUri),
    Precondition(SymbolUri, CognitiveDimension),
    Objective(SymbolUri, CognitiveDimension),
    AnswerClassFeedback,
    ProofBody,
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
                Some(Argument::Sequence(MaybeSequence::Seq(
                    v.into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )))
            }
            Self::Sequence(Left(t)) => Some(Argument::Sequence(MaybeSequence::One(t))),
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
                ArgumentMode::Simple | ArgumentMode::BoundVariable => *r = Self::Simple(term),
                ArgumentMode::Sequence | ArgumentMode::BoundVariableSequence => {
                    *r = Self::Sequence(Left(term));
                }
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
                if v[idx].as_ref().is_some_and(|t| *t != term) {
                    return Err(FtmlExtractionError::MismatchedArgument {
                        pos: position,
                        //t: term,
                        //args: args.clone(),
                    });
                }
                v[idx] = Some(term);
            }
            (Self::Simple(t), ArgumentPosition::Simple(_, _)) if *t == term => (),
            _ => {
                return Err(FtmlExtractionError::MismatchedArgument {
                    pos: position,
                    //t: term,
                    //args: args.clone(),
                });
            }
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
            } => Some(BoundArgument::Bound(ComponentVar {
                var: v,
                tp: None,
                df: None,
            })),
            Self::Simple { term, .. } => Some(BoundArgument::Simple(term)),
            Self::Sequence {
                terms: Left(Term::Var { variable: v, .. }),
                should_be_var: true,
            } => Some(BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                var: v,
                tp: None,
                df: None,
            }))),
            Self::Sequence {
                terms: Right(v),
                should_be_var: true,
            } if v.iter().all(|t| matches!(t, Some(Term::Var { .. }))) => {
                Some(BoundArgument::BoundSeq(MaybeSequence::Seq(
                    v.into_iter()
                        .map(|v| {
                            let Some(Term::Var { variable, .. }) = v else {
                                // SAFETY: iter.all() matches above
                                unsafe { unreachable_unchecked() }
                            };
                            ComponentVar {
                                var: variable,
                                tp: None,
                                df: None,
                            }
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )))
            }
            Self::Sequence {
                terms: Right(v), ..
            } if v.iter().all(Option::is_some) => Some(BoundArgument::Sequence(
                MaybeSequence::Seq(v.into_iter().flatten().collect()),
            )),
            Self::Sequence { terms: Left(a), .. } => {
                Some(BoundArgument::Sequence(MaybeSequence::One(a)))
            }
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
                if v[idx].as_ref().is_some_and(|t| *t != term) {
                    return Err(FtmlExtractionError::MismatchedBoundArgument {
                        pos: position,
                        //t: term,
                        //args: args.clone(),
                    });
                }
                v[idx] = Some(term);
            }
            (Self::Simple { term: t, .. }, ArgumentPosition::Simple(_, _)) if *t == term => (),
            _ => {
                return Err(FtmlExtractionError::MismatchedBoundArgument {
                    pos: position,
                    //t: term,
                    //args: args.clone(),
                });
            }
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
            Self::DocumentUri(uri) => AnyOpen::Meta(MetaDatum::DocumentUri(uri)),
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
            Self::Morphism { uri, domain, total } => AnyOpen::Open {
                domain: Some(OpenDomainElement::Morphism {
                    uri: uri.clone(),
                    domain,
                    total,
                    children: Vec::new(),
                }),
                narrative: Some(OpenNarrativeElement::Morphism {
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
            Self::Assign(uri) => AnyOpen::Open {
                domain: Some(OpenDomainElement::Assign {
                    source: uri,
                    refined_type: None,
                    definiens: None,
                }),
                narrative: None,
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
            Self::Rule(id) => AnyOpen::Open {
                domain: Some(OpenDomainElement::InferenceRule {
                    rule: id,
                    parameters: Vec::new(),
                }),
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
                domain: Some(OpenDomainElement::OML {
                    name,
                    tp: None,
                    df: None,
                }),
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
            Self::Slide(uri) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Slide {
                    uri,
                    children: Vec::new(),
                    title: None,
                }),
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
            Self::ArgTypes => AnyOpen::Open {
                domain: Some(OpenDomainElement::ArgTypes(Vec::new())),
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
            Self::Problem {
                is_subproblem,
                styles,
                uri,
                autogradable,
                points,
                minutes,
            } => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Problem {
                    uri,
                    children: Vec::new(),
                    title: None,
                    sub_problem: is_subproblem,
                    autogradable,
                    points,
                    minutes,
                    solutions: Vec::new(),
                    gnotes: Vec::new(),
                    hints: Vec::new(),
                    notes: Vec::new(),
                    styles,
                    preconditions: Vec::new(),
                    objectives: Vec::new(),
                }),
            },
            Self::FillinSol(wd) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::FillinSol {
                    width: wd,
                    cases: Vec::new(),
                    nodes: Vec::new(),
                }),
            },
            Self::FillinSolCase(case) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::FillinSolCase(case)),
            },
            Self::Precondition { uri, dim } => AnyOpen::Meta(MetaDatum::Precondition(uri, dim)),
            Self::Objective { uri, dim } => AnyOpen::Meta(MetaDatum::Objective(uri, dim)),
            Self::ProblemHint => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ProblemHint),
            },
            Self::ProblemExNote => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ProblemExNote),
            },
            Self::ProblemGradingNote => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ProblemGradingNote(Vec::new())),
            },
            Self::AnswerClass(id, kind) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::AnswerClass {
                    id,
                    kind,
                    feedback: Box::default(),
                    nodes: Vec::new(),
                }),
            },
            Self::ChoiceBlock {
                styles,
                block_style,
                multiple,
            } => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ChoiceBlock {
                    styles,
                    block_style,
                    multiple,
                    choices: Vec::new(),
                }),
            },
            Self::ProblemChoice(correct) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ProblemChoice {
                    correct,
                    verdict: None,
                    feedback: Box::default(),
                    nodes: Vec::new(),
                }),
            },
            Self::ProblemChoiceVerdict => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ProblemChoiceVerdict),
            },
            Self::ProblemChoiceFeedback => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::ProblemChoiceFeedback),
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
            Self::Solution(id) => AnyOpen::Open {
                domain: None,
                narrative: Some(OpenNarrativeElement::Solution(id)),
            },
            Self::InputRef {
                target: uri,
                uri: id,
            } => AnyOpen::Meta(MetaDatum::InputRef {
                target: uri,
                uri: id,
            }),
            Self::ProofBody => AnyOpen::Meta(MetaDatum::ProofBody),
            Self::AnswerClassFeedback => AnyOpen::Meta(MetaDatum::AnswerClassFeedback),
            Self::DocumentKind(k) => AnyOpen::Meta(MetaDatum::DocumentKind(k)),
            Self::IfInputref(b) => AnyOpen::Meta(MetaDatum::IfInputref(b)),
            Self::Style(s) => AnyOpen::Meta(MetaDatum::Style(s)),
            Self::Counter(c) => AnyOpen::Meta(MetaDatum::Counter(c)),
            Self::SetSectionLevel(lvl) => AnyOpen::Meta(MetaDatum::SetSectionLevel(lvl)),
            Self::ImportModule(uri) => AnyOpen::Meta(MetaDatum::ImportModule(uri)),
            Self::UseModule(uri) => AnyOpen::Meta(MetaDatum::UseModule(uri)),
            Self::Rename {
                source,
                name,
                macroname,
            } => AnyOpen::Meta(MetaDatum::Rename {
                source,
                name,
                macroname,
            }),
            Self::None
            | Self::SectionTitle
            | Self::ParagraphTitle
            | Self::ProblemTitle
            | Self::SlideTitle
            | Self::SlideNumber
            | Self::CurrentSectionLevel(_) => AnyOpen::None,
        }
    }
}
