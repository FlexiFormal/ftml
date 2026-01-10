use crate::{
    FtmlKey,
    extraction::{
        AnyOpen, ArgumentPosition, CloseFtmlElement, FtmlExtractionError, MetaDatum, OpenArgument,
        OpenBoundArgument, OpenDomainElement, OpenFtmlElement, OpenNarrativeElement,
        nodes::FtmlNode,
    },
};
use ftml_ontology::{
    domain::{
        declarations::{
            Declaration,
            morphisms::{Assignment, Morphism},
            structures::{MathStructure, StructureDeclaration, StructureExtension},
            symbols::{Symbol, SymbolData},
        },
        modules::{Module, ModuleData, NestedModule},
    },
    narrative::{
        DataBuffer, DocumentRange,
        documents::{
            Document, DocumentCounter, DocumentData, DocumentKind, DocumentStyle, DocumentStyles,
        },
        elements::{
            DocumentElement, DocumentTerm, LogicalParagraph, Notation, Problem, Section,
            SectionLevel, Slide, VariableDeclaration,
            notations::{
                NotationComponent, NotationNode, NotationReference, VariableNotationReference,
            },
            paragraphs::{ParagraphFormatting, ParagraphKind},
            problems::{
                AnswerClass, AnswerKind, Choice, ChoiceBlock, ChoiceBlockStyle, FillInSol,
                FillInSolOption, GradingNote, ProblemData, SolutionData, Solutions,
            },
            variables::VariableData,
        },
    },
    terms::{ApplicationTerm, BindingTerm, Term, TermContainer, VarOrSym, Variable},
    utils::SourceRange,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, DomainUriRef, Id, IsDomainUri, Language, LeafUri, ModuleUri,
    SymbolUri,
};
use std::{borrow::Cow, hint::unreachable_unchecked};

#[derive(Debug)]
pub struct IdCounter {
    inner: rustc_hash::FxHashMap<Cow<'static, str>, u32>,
    forced: std::sync::Mutex<Option<DocumentElementUri>>,
}
impl Default for IdCounter {
    fn default() -> Self {
        let mut inner = rustc_hash::FxHashMap::default();
        inner.insert("EXTSTRUCT".into(), 0);
        Self {
            inner,
            forced: std::sync::Mutex::new(None),
        }
    }
}
impl Clone for IdCounter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            forced: std::sync::Mutex::new(self.forced.lock().ok().and_then(|mut e| e.take())),
        }
    }
}
impl IdCounter {
    pub fn forced(&mut self) -> Option<DocumentElementUri> {
        self.forced.lock().ok().and_then(|mut e| e.take())
    }
    pub fn new_id(&mut self, prefix: impl Into<Cow<'static, str>>) -> String {
        use std::collections::hash_map::Entry;
        let prefix = prefix.into();
        match self.inner.entry(prefix) {
            Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
                format!("{}_{}", e.key(), e.get())
            }
            Entry::Vacant(e) => {
                let r = e.key().to_string();
                e.insert(0);
                r
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackVec<T> {
    last: Option<T>,
    rest: Vec<T>,
}
impl<T> Default for StackVec<T> {
    fn default() -> Self {
        Self {
            last: None,
            rest: Vec::new(),
        }
    }
}
impl<T> StackVec<T> {
    #[inline]
    const fn last_mut(&mut self) -> Option<&mut T> {
        self.last.as_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        use either::Either::{Left, Right};
        self.last.as_ref().map_or_else(
            || Right(std::iter::empty()),
            |l| Left(std::iter::once(l).chain(self.rest.iter().rev())),
        )
    }
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        use either::Either::{Left, Right};
        if let Some(l) = &mut self.last {
            Left(std::iter::once(l).chain(self.rest.iter_mut().rev()))
        } else {
            Right(std::iter::empty())
        }
    }
    #[inline]
    const fn is_empty(&self) -> bool {
        self.last.is_none()
    }

    pub fn push(&mut self, e: T) {
        if let Some(e) = self.last.replace(e) {
            self.rest.push(e);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        std::mem::replace(&mut self.last, self.rest.pop())
    }
}

pub struct ExtractorState<N: FtmlNode + std::fmt::Debug> {
    pub document: DocumentUri,
    pub top: Vec<DocumentElement>,
    pub modules: Vec<ModuleData>,
    pub counters: Vec<DocumentCounter>,
    pub styles: Vec<DocumentStyle>,
    pub buffer: DataBuffer,
    pub title: Option<Box<str>>,
    pub notations: Vec<(LeafUri, DocumentElementUri, Notation)>,
    pub solutions: Vec<(DocumentElementUri, Solutions)>,
    pub domain: StackVec<OpenDomainElement<N>>,
    pub narrative: StackVec<OpenNarrativeElement<N>>,
    pub kind: DocumentKind,
    pub current_source_range: SourceRange,
    top_section_level: Option<SectionLevel>,
    pub(crate) ids: IdCounter,
    #[allow(dead_code)]
    do_rdf: bool,
    #[cfg(feature = "rdf")]
    rdf: Vec<ulo::rdf_types::Triple>,
}

#[derive(Debug)]
pub struct ExtractionResult {
    pub document: Document,
    pub modules: Vec<Module>,
    pub data: Box<[u8]>,
    #[cfg(feature = "rdf")]
    pub triples: Vec<ulo::rdf_types::Triple>,
    pub notations: Vec<(LeafUri, DocumentElementUri, Notation)>,
}

#[allow(unused_variables)]
impl<N: FtmlNode + std::fmt::Debug> ExtractorState<N> {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(document: DocumentUri, do_rdf: bool) -> Self {
        Self {
            do_rdf,
            document,
            top_section_level: None,
            title: None,
            ids: IdCounter::default(),
            counters: Vec::new(),
            styles: Vec::new(),
            notations: Vec::new(),
            solutions: Vec::new(),
            buffer: DataBuffer::default(),
            top: Vec::new(),
            modules: Vec::new(),
            kind: DocumentKind::default(),
            domain: StackVec::default(),
            narrative: StackVec::default(),
            current_source_range: SourceRange::DEFAULT,
            #[cfg(feature = "rdf")]
            rdf: Vec::new(),
        }
    }

    pub fn set_next_uri(&mut self, uri: DocumentElementUri) {
        if let Ok(mut e) = self.ids.forced.lock() {
            *e = Some(uri);
        }
    }

    pub fn finish(&mut self) -> ExtractionResult {
        use std::mem::take;
        let document = DocumentData {
            uri: self.document.clone(),
            title: take(&mut self.title), // todo
            elements: take(&mut self.top).into_boxed_slice(),
            top_section_level: self.top_section_level.unwrap_or_default(),
            kind: self.kind.clone(),
            styles: DocumentStyles {
                // clone instead of take because DomExtractor
                // still needs them
                counters: self.counters.clone().into_boxed_slice(),
                styles: self.styles.clone().into_boxed_slice(),
            },
        }
        .close();
        #[cfg(feature = "rdf")]
        {
            use ftml_ontology::Ftml;
            self.rdf.extend(document.triples());
        }
        tracing::info!("Finished document {document:#?}");
        let modules = take(&mut self.modules)
            .into_iter()
            .map(|m| {
                tracing::info!("Found module {m:#?}");
                m.close()
            })
            .collect();
        let data = take(&mut self.buffer).take();
        #[cfg(feature = "rdf")]
        let triples = take(&mut self.rdf);
        ExtractionResult {
            document,
            modules,
            data,
            #[cfg(feature = "rdf")]
            triples,
            notations: take(&mut self.notations),
        }
    }

    #[inline]
    /// ### Errors
    pub fn new_id(
        &mut self,
        key: FtmlKey,
        prefix: impl Into<Cow<'static, str>>,
    ) -> super::Result<Id> {
        Ok(self.ids.new_id(prefix).parse().map_err(|e| (key, e))?)
    }

    #[inline]
    #[must_use]
    pub const fn in_document(&self) -> &DocumentUri {
        &self.document
    }
    #[inline]
    pub fn domain(&self) -> impl Iterator<Item = &OpenDomainElement<N>> {
        self.domain.iter()
    }
    #[inline]
    pub fn narrative(&self) -> impl Iterator<Item = &OpenNarrativeElement<N>> {
        self.narrative.iter()
    }

    #[allow(clippy::too_many_lines)]
    fn do_meta(&mut self, m: MetaDatum, node: &N) -> Result<(), FtmlExtractionError> {
        match m {
            MetaDatum::DocumentKind(k) => self.kind = k,
            MetaDatum::Style(s) => self.styles.push(s),
            MetaDatum::Counter(c) => self.counters.push(c),
            MetaDatum::InputRef { target, uri } => {
                self.push_elem(DocumentElement::DocumentReference {
                    uri,
                    target,
                    source: self.current_source_range,
                });
            }
            MetaDatum::SetSectionLevel(lvl) => {
                self.top_section_level = Some(lvl);
            }
            MetaDatum::UseModule(uri) => self.push_elem(DocumentElement::UseModule {
                uri,
                source: self.current_source_range,
            }),
            MetaDatum::ImportModule(uri) => {
                let source = self.current_source_range;
                self.push_domain(
                    uri.clone(),
                    |uri| Declaration::Import { uri, source },
                    |uri| Ok(StructureDeclaration::Import { uri, source }),
                )?;
                self.push_elem(DocumentElement::ImportModule(uri));
            }
            MetaDatum::Rename {
                source,
                name,
                macroname,
            } => {
                if let Some(OpenDomainElement::Morphism { children, uri, .. }) =
                    self.domain.last_mut()
                {
                    if let Some(e) = children.iter_mut().find(|e| e.original == source) {
                        e.new_name = name;
                        e.macroname = macroname;
                    } else {
                        children.push(Assignment {
                            original: source,
                            morphism: uri.clone(),
                            definiens: None,
                            refined_type: None,
                            new_name: name,
                            macroname,
                            source: self.current_source_range,
                        });
                    }
                } else {
                    return Err(FtmlExtractionError::InvalidIn(
                        FtmlKey::Rename,
                        "outside of morphisms",
                    ));
                }
            }
            MetaDatum::Precondition(uri, dim) => {
                if let Some(OpenNarrativeElement::Problem { preconditions, .. }) =
                    self.narrative.last_mut()
                {
                    preconditions.push((dim, uri));
                } else {
                    return Err(FtmlExtractionError::InvalidIn(
                        FtmlKey::PreconditionSymbol,
                        "outside of (sub)problems",
                    ));
                }
            }
            MetaDatum::Objective(uri, dim) => {
                if let Some(OpenNarrativeElement::Problem { objectives, .. }) =
                    self.narrative.last_mut()
                {
                    objectives.push((dim, uri));
                } else {
                    return Err(FtmlExtractionError::InvalidIn(
                        FtmlKey::ObjectiveSymbol,
                        "outside of (sub)problems",
                    ));
                }
            }
            MetaDatum::AnswerClassFeedback => {
                if let Some(nodes) = self.narrative.iter_mut().find_map(|e| {
                    if let OpenNarrativeElement::AnswerClass { nodes, .. } = e {
                        Some(nodes)
                    } else {
                        None
                    }
                }) {
                    nodes.push(node.clone());
                } else {
                    return Err(FtmlExtractionError::NotIn(
                        FtmlKey::AnswerclassFeedback,
                        "answer classes",
                    ));
                }
            }
            MetaDatum::IfInputref(_) | MetaDatum::ProofBody => (),
        }
        Ok(())
    }

    /// ### Errors
    pub fn add(&mut self, e: OpenFtmlElement, node: &N) -> Result<(), FtmlExtractionError> {
        match e.split(node) {
            AnyOpen::Open { domain, narrative } => {
                if let Some(dom) = domain {
                    self.domain.push(dom);
                }
                if let Some(narr) = narrative {
                    self.narrative.push(narr);
                }
            }
            AnyOpen::Meta(m) => self.do_meta(m, node)?,
            AnyOpen::None => (),
        }
        Ok(())
    }

    /// ### Errors
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    pub fn close(&mut self, elem: CloseFtmlElement, node: &N) -> super::Result<()> {
        tracing::trace!("Closing: {elem:?} in {:?}", self.domain);
        match elem {
            CloseFtmlElement::Module => match self.domain.pop() {
                Some(OpenDomainElement::Module {
                    uri,
                    meta,
                    signature,
                    children,
                }) => self.close_any_module(uri, meta, signature, children, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Module)),
            },
            CloseFtmlElement::MathStructure => match self.domain.pop() {
                Some(OpenDomainElement::MathStructure {
                    uri,
                    macroname,
                    children,
                }) => self.close_structure(uri, macroname, children, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::MathStructure)),
            },
            CloseFtmlElement::Morphism => match self.domain.pop() {
                Some(OpenDomainElement::Morphism {
                    uri,
                    domain,
                    total,
                    children,
                }) => self.close_morphism(uri, domain, total, children, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Morphism)),
            },
            CloseFtmlElement::Rule => match self.domain.pop() {
                Some(OpenDomainElement::InferenceRule { rule, parameters }) => {
                    self.close_inferencerule(rule, parameters)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::InferenceRule)),
            },
            CloseFtmlElement::Comp => match self.domain.pop() {
                Some(OpenDomainElement::Comp) => Ok(()),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Comp)),
            },
            CloseFtmlElement::DefComp => match self.domain.pop() {
                Some(OpenDomainElement::DefComp) => Ok(()),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::DefComp)),
            },
            CloseFtmlElement::SymbolDeclaration => match self.domain.pop() {
                Some(OpenDomainElement::SymbolDeclaration { uri, data }) => {
                    self.close_symbol(uri, data)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Symdecl)),
            },
            CloseFtmlElement::VariableDeclaration => match self.narrative.pop() {
                Some(OpenNarrativeElement::VariableDeclaration { uri, data }) => {
                    self.close_vardecl(uri, data);
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Vardef)),
            },
            CloseFtmlElement::Definiendum => match self.narrative.pop() {
                Some(OpenNarrativeElement::Definiendum(uri)) => {
                    self.close_definiendum(uri, node.range())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiendum)),
            },
            CloseFtmlElement::Section => match self.narrative.pop() {
                Some(OpenNarrativeElement::Section {
                    uri,
                    title,
                    children,
                }) => {
                    self.close_section(uri, title, node.range(), children);
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Section)),
            },
            CloseFtmlElement::Paragraph => match self.narrative.pop() {
                Some(OpenNarrativeElement::Paragraph {
                    uri,
                    kind,
                    formatting,
                    styles,
                    children,
                    fors,
                    title,
                }) => {
                    self.close_paragraph(
                        uri,
                        kind,
                        fors,
                        formatting,
                        styles,
                        children,
                        title,
                        node.range(),
                    );
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Paragraph)),
            },
            CloseFtmlElement::Slide => match self.narrative.pop() {
                Some(OpenNarrativeElement::Slide {
                    uri,
                    children,
                    title,
                }) => {
                    self.close_slide(uri, children, title, node.range());
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Slide)),
            },
            CloseFtmlElement::Assign => match self.domain.pop() {
                Some(OpenDomainElement::Assign {
                    source,
                    refined_type,
                    definiens,
                }) => self.close_assignment(source, refined_type, definiens),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Assign)),
            },
            CloseFtmlElement::SkipSection => match self.narrative.pop() {
                Some(OpenNarrativeElement::SkipSection { children }) => {
                    self.push_elem(DocumentElement::SkipSection(children.into_boxed_slice()));
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::SkipSection)),
            },
            CloseFtmlElement::Notation => match self.narrative.pop() {
                Some(OpenNarrativeElement::Notation {
                    uri,
                    id,
                    head,
                    prec,
                    argprecs,
                    component,
                    op,
                }) => self.close_notation(uri, id, head, prec, argprecs, component, op),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Notation)),
            },
            CloseFtmlElement::NotationComp => match self.narrative.pop() {
                Some(OpenNarrativeElement::NotationComp { node, components }) => {
                    self.close_notation_component(&node, components)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::NotationComp)),
            },
            CloseFtmlElement::ArgSep => match self.narrative.pop() {
                Some(OpenNarrativeElement::ArgSep { node, components }) => {
                    self.close_argsep(&node, components)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ArgSep)),
            },
            CloseFtmlElement::NotationArg => match self.narrative.pop() {
                Some(OpenNarrativeElement::NotationArg(arg)) => self.close_notation_arg(node, arg),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Arg)),
            },
            CloseFtmlElement::NotationOpComp => self.close_notation_op(node),
            CloseFtmlElement::CompInNotation => self.close_comp_in_notation(node, false),
            CloseFtmlElement::MainCompInNotation => self.close_comp_in_notation(node, true),
            CloseFtmlElement::SymbolReference => {
                if let Some(OpenDomainElement::SymbolReference { uri, notation }) =
                    self.domain.pop()
                {
                    self.close_oms(uri, notation, node)
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::VariableReference => {
                if let Some(OpenDomainElement::VariableReference { var, notation }) =
                    self.domain.pop()
                {
                    self.close_omv(var, notation, node)
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::OMA => {
                if let Some(OpenDomainElement::OMA {
                    head,
                    notation: _,
                    head_term,
                    arguments,
                    uri,
                }) = self.domain.pop()
                {
                    self.close_oma(head, head_term, uri, arguments, node)
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::OMBIND => {
                if let Some(OpenDomainElement::OMBIND {
                    head,
                    notation: _,
                    head_term,
                    arguments,
                    uri,
                }) = self.domain.pop()
                {
                    self.close_ombind(head, head_term, uri, arguments, node)
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::OML => {
                if let Some(OpenDomainElement::OML { name, df, tp }) = self.domain.pop() {
                    self.close_term(
                        Term::Label {
                            name,
                            df: df.map(Box::new),
                            tp: tp.map(Box::new),
                        },
                        node,
                        |_, _| Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term)),
                    )
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::ComplexTerm => {
                if let Some(OpenDomainElement::ComplexTerm {
                    head,
                    head_term,
                    notation: _,
                    uri,
                }) = self.domain.pop()
                {
                    self.close_complex(head, head_term, uri, node)
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::Argument => match self.domain.pop() {
                Some(OpenDomainElement::Argument {
                    position,
                    terms,
                    node: n,
                }) => {
                    //debug_assert_eq!(node,n);
                    self.close_argument(position, terms, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Arg)),
            },
            CloseFtmlElement::HeadTerm => match self.domain.pop() {
                Some(OpenDomainElement::HeadTerm { terms, node }) => {
                    self.close_head_term(terms, &node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::HeadTerm)),
            },
            CloseFtmlElement::Type => match self.domain.pop() {
                Some(OpenDomainElement::Type { terms, node: n }) => {
                    //debug_assert_eq!(node,n);
                    self.close_type(terms, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type)),
            },
            CloseFtmlElement::ArgTypes => match self.domain.pop() {
                Some(OpenDomainElement::ArgTypes(terms)) => {
                    //debug_assert_eq!(node,n);
                    self.close_argtypes(terms, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type)),
            },
            CloseFtmlElement::ReturnType => match self.domain.pop() {
                Some(OpenDomainElement::ReturnType { terms, node: n }) => {
                    //debug_assert_eq!(node,n);
                    self.close_return_type(terms, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type)),
            },
            CloseFtmlElement::Definiens => match self.domain.pop() {
                Some(OpenDomainElement::Definiens {
                    terms,
                    node: n,
                    uri,
                }) => {
                    //debug_assert_eq!(node,n);
                    self.close_definiens(terms, uri, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiens)),
            },
            CloseFtmlElement::Problem => match self.narrative.pop() {
                Some(OpenNarrativeElement::Problem {
                    uri,
                    children,
                    title,
                    sub_problem,
                    autogradable,
                    points,
                    minutes,
                    solutions,
                    gnotes,
                    hints,
                    notes,
                    styles,
                    preconditions,
                    objectives,
                }) => {
                    let solutions = Solutions::from_solutions(solutions.into_boxed_slice());
                    let solref = self.buffer.push(&solutions).map_err(|e| {
                        FtmlExtractionError::EncodingError(FtmlKey::Problem, e.to_string())
                    })?;
                    self.solutions.push((uri.clone(), solutions));
                    let data = Box::new(ProblemData {
                        sub_problem,
                        autogradable,
                        points,
                        minutes,
                        solutions: solref,
                        gnotes: gnotes.into_boxed_slice(),
                        hints: hints.into_boxed_slice(),
                        notes: notes.into_boxed_slice(),
                        styles,
                        title,
                        preconditions: preconditions.into_boxed_slice(),
                        objectives: objectives.into_boxed_slice(),
                        source: self.current_source_range,
                    });
                    self.push_elem(DocumentElement::Problem(Problem {
                        uri,
                        range: node.range(),
                        children: children.into_boxed_slice(),
                        data,
                    }));
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Problem)),
            },
            CloseFtmlElement::FillinSol => match self.narrative.pop() {
                Some(OpenNarrativeElement::FillinSol {
                    width,
                    cases,
                    nodes,
                }) => {
                    for n in nodes {
                        n.delete();
                    }
                    self.close_fillinsol(width, cases, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(
                    FtmlKey::ProblemFillinsol,
                )),
            },
            CloseFtmlElement::Solution => match self.narrative.pop() {
                Some(OpenNarrativeElement::Solution(id)) => self.close_solution(id, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(
                    FtmlKey::ProblemSolution,
                )),
            },
            CloseFtmlElement::ProblemHint => match self.narrative.pop() {
                Some(OpenNarrativeElement::ProblemHint) => self.close_hint(node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ProblemHint)),
            },
            CloseFtmlElement::ProblemExNote => match self.narrative.pop() {
                Some(OpenNarrativeElement::ProblemExNote) => self.close_exnote(node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ProblemNote)),
            },
            CloseFtmlElement::ProblemGradingNote => match self.narrative.pop() {
                Some(OpenNarrativeElement::ProblemGradingNote(v)) => self.close_gnote(v, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(
                    FtmlKey::ProblemGradingNote,
                )),
            },
            CloseFtmlElement::AnswerClass => match self.narrative.pop() {
                Some(OpenNarrativeElement::AnswerClass {
                    id,
                    kind,
                    feedback,
                    nodes,
                }) => {
                    for n in nodes {
                        n.delete();
                    }
                    self.close_answerclass(id, kind, feedback, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::AnswerClass)),
            },
            CloseFtmlElement::ChoiceBlock => match self.narrative.pop() {
                Some(OpenNarrativeElement::ChoiceBlock {
                    styles,
                    block_style,
                    multiple,
                    choices,
                }) => self.close_choice_block(styles, block_style, multiple, choices, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(
                    FtmlKey::ProblemMultipleChoiceBlock,
                )),
            },
            CloseFtmlElement::ProblemChoice => match self.narrative.pop() {
                Some(OpenNarrativeElement::ProblemChoice {
                    correct,
                    verdict,
                    feedback,
                    nodes,
                }) => {
                    for n in nodes {
                        n.delete();
                    }
                    self.close_choice(correct, verdict, feedback)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ProblemChoice)),
            },
            CloseFtmlElement::FillinSolCase => {
                let Some(OpenNarrativeElement::FillinSolCase(mut opt)) = self.narrative.pop()
                else {
                    return Err(FtmlExtractionError::UnexpectedEndOf(
                        FtmlKey::ProblemFillinsolCase,
                    ));
                };
                match self.narrative.last_mut() {
                    Some(OpenNarrativeElement::FillinSol { cases, nodes, .. }) => {
                        let fb = node.inner_string().into_owned().into_boxed_str();
                        match &mut opt {
                            FillInSolOption::Exact { feedback, .. }
                            | FillInSolOption::Regex { feedback, .. }
                            | FillInSolOption::NumericalRange { feedback, .. } => *feedback = fb,
                        }
                        nodes.push(node.clone());
                        cases.push(opt);
                    }
                    _ => {
                        return Err(FtmlExtractionError::NotIn(
                            FtmlKey::ProblemFillinsolCase,
                            "fill-in-solutions",
                        ));
                    }
                }
                Ok(())
            }
            CloseFtmlElement::ProblemChoiceVerdict => {
                match self.narrative.pop() {
                    Some(OpenNarrativeElement::ProblemChoiceVerdict) => (),
                    _ => {
                        return Err(FtmlExtractionError::UnexpectedEndOf(
                            FtmlKey::ProblemChoiceVerdict,
                        ));
                    }
                }
                if let Some((nodes, verdict)) = self.narrative.iter_mut().find_map(|e| {
                    if let OpenNarrativeElement::ProblemChoice { nodes, verdict, .. } = e {
                        Some((nodes, verdict))
                    } else {
                        None
                    }
                }) {
                    *verdict = Some(node.inner_string().into_owned().into_boxed_str());
                    nodes.push(node.clone());
                    Ok(())
                } else {
                    Err(FtmlExtractionError::NotIn(
                        FtmlKey::ProblemChoiceVerdict,
                        "problem choices",
                    ))
                }
            }
            CloseFtmlElement::ProblemChoiceFeedback => {
                match self.narrative.pop() {
                    Some(OpenNarrativeElement::ProblemChoiceFeedback) => (),
                    _ => {
                        return Err(FtmlExtractionError::UnexpectedEndOf(
                            FtmlKey::ProblemChoiceFeedback,
                        ));
                    }
                }
                if let Some((nodes, feedback)) = self.narrative.iter_mut().find_map(|e| {
                    if let OpenNarrativeElement::ProblemChoice {
                        nodes, feedback, ..
                    } = e
                    {
                        Some((nodes, feedback))
                    } else {
                        None
                    }
                }) {
                    *feedback = node.inner_string().into_owned().into_boxed_str();
                    nodes.push(node.clone());
                    Ok(())
                } else {
                    Err(FtmlExtractionError::NotIn(
                        FtmlKey::ProblemChoiceFeedback,
                        "problem choices",
                    ))
                }
            }
            CloseFtmlElement::SectionTitle => self.close_section_title(node),
            CloseFtmlElement::ParagraphTitle => self.close_paragraph_title(node),
            CloseFtmlElement::ProblemTitle => self.close_problem_title(node),
            CloseFtmlElement::SlideTitle => self.close_slide_title(node),
            CloseFtmlElement::DocTitle => {
                self.title = Some(node.inner_string().into_owned().into_boxed_str());
                Ok(())
            }
            CloseFtmlElement::Invisible => {
                match self.narrative.pop() {
                    Some(OpenNarrativeElement::Invisible) => (),
                    e => return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Invisible)),
                }
                if !self
                    .narrative
                    .iter()
                    .any(|e| matches!(e, OpenNarrativeElement::Notation { .. }))
                    && !self.domain.iter().any(|e| {
                        matches!(
                            e,
                            OpenDomainElement::OMA { .. } | OpenDomainElement::OMBIND { .. }
                        )
                    })
                {
                    node.delete();
                }
                Ok(())
            }
        }
    }

    fn push_domain<D>(
        &mut self,
        data: D,
        module: impl FnOnce(D) -> Declaration,
        structure: impl FnOnce(D) -> Result<StructureDeclaration, FtmlExtractionError>,
    ) -> Result<DomainUriRef<'_>, FtmlExtractionError> {
        for e in self.domain.iter_mut() {
            let uri = match e {
                OpenDomainElement::Module { uri, children, .. } => {
                    let elem = module(data);
                    #[cfg(feature = "rdf")]
                    {
                        use ftml_ontology::Ftml;
                        self.rdf.extend(elem.triples());
                    }
                    children.push(elem);
                    DomainUriRef::Module(uri)
                }
                OpenDomainElement::MathStructure { uri, children, .. } => {
                    let elem = structure(data)?;
                    #[cfg(feature = "rdf")]
                    {
                        use ftml_ontology::Ftml;
                        self.rdf.extend(elem.triples());
                    }
                    children.push(elem);
                    DomainUriRef::Symbol(uri)
                }
                _ => continue,
            };
            return Ok(uri);
        }

        Err(FtmlExtractionError::NotIn(
            FtmlKey::Symdecl,
            "a module or structure (or inside of a declaration)",
        ))
    }

    fn close_fillinsol(
        &mut self,
        width: Option<f32>,
        mut opts: Vec<FillInSolOption>,
        node: &N,
    ) -> Result<(), FtmlExtractionError> {
        let exact = node.inner_string().into_owned().into_boxed_str();
        opts.insert(
            0,
            FillInSolOption::Exact {
                value: exact,
                verdict: true,
                feedback: Box::default(),
            },
        );
        for d in self.narrative.iter_mut() {
            if let OpenNarrativeElement::Problem { solutions, .. } = d {
                solutions.push(SolutionData::FillInSol(FillInSol { width, opts }));
                return Ok(());
            }
        }
        Err(FtmlExtractionError::NotIn(
            FtmlKey::ProblemFillinsol,
            "problems",
        ))
    }

    fn push_elem(&mut self, e: DocumentElement) {
        #[cfg(feature = "rdf")]
        {
            use ftml_ontology::Ftml;
            self.rdf.extend(e.triples());
        }
        for d in self.narrative.iter_mut() {
            match d {
                OpenNarrativeElement::Module { children, .. }
                | OpenNarrativeElement::MathStructure { children, .. }
                | OpenNarrativeElement::Morphism { children, .. }
                | OpenNarrativeElement::Section { children, .. }
                | OpenNarrativeElement::Paragraph { children, .. }
                | OpenNarrativeElement::Slide { children, .. }
                | OpenNarrativeElement::Problem { children, .. }
                | OpenNarrativeElement::SkipSection { children } => {
                    children.push(e);
                    return;
                }
                OpenNarrativeElement::Invisible
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::Solution(_)
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::NotationArg(_) => (),
            }
        }
        self.top.push(e);
    }

    fn close_extension(
        &mut self,
        uri: SymbolUri,
        mut children: Vec<StructureDeclaration>,
        node: &N,
    ) -> Result<(), FtmlExtractionError> {
        let Some((i,_)) = children.iter().enumerate().find(|(_,i)|
            matches!(i,StructureDeclaration::Import{uri:i,..} if !i.module_name().last().starts_with("EXTSTRUCT"))
        ) else {
            return Err(FtmlExtractionError::MissingArgument(0))
        };
        let StructureDeclaration::Import { uri: i, .. } = children.remove(i) else {
            // SAFETY: match above
            unsafe { unreachable_unchecked() }
        };
        let Some(target) = i.into_symbol() else {
            return Err(FtmlExtractionError::InvalidValue(FtmlKey::ImportModule));
        };
        let ext = StructureExtension {
            uri,
            target,
            elements: children.into_boxed_slice(),
            source: self.current_source_range,
        };

        let Some(OpenNarrativeElement::MathStructure { uri, children }) = self.narrative.pop()
        else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::MathStructure));
        };
        //add_triples!(DOM self,uri -> parent_uri);
        self.push_elem(DocumentElement::Extension {
            range: node.range(),
            extension: ext.uri.clone(),
            target: ext.target.clone(),
            children: children.into_boxed_slice(),
        });

        tracing::trace!("New {ext:?}");
        let parent_uri = self.push_domain(ext, Declaration::Extension, |s| {
            Err(FtmlExtractionError::InvalidIn(
                FtmlKey::MathStructure,
                "other structures",
            ))
        })?;

        Ok(())
    }

    fn close_morphism(
        &mut self,
        uri: SymbolUri,
        domain: ModuleUri,
        total: bool,
        children: Vec<Assignment>,
        node: &N,
    ) -> Result<(), FtmlExtractionError> {
        let morphism = Morphism {
            uri,
            domain,
            total,
            elements: children.into_boxed_slice(),
            source: self.current_source_range,
        };
        tracing::trace!("New morphism {morphism:?}");
        let parent_uri = self.push_domain(morphism, Declaration::Morphism, |m| {
            Ok(StructureDeclaration::Morphism(m))
        })?;

        let Some(OpenNarrativeElement::Morphism { uri, children }) = self.narrative.pop() else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Morphism));
        };
        //add_triples!(DOM self,uri -> parent_uri);
        self.push_elem(DocumentElement::Morphism {
            range: node.range(),
            morphism: uri,
            children: children.into_boxed_slice(),
        });
        Ok(())
    }

    fn close_assignment(
        &mut self,
        source: SymbolUri,
        refined_type: Option<Term>,
        definiens: Option<Term>,
    ) -> Result<(), FtmlExtractionError> {
        for e in self.domain.iter_mut() {
            if let OpenDomainElement::Morphism { children, uri, .. } = e {
                if let Some(e) = children.iter_mut().find(|e| e.original == source) {
                    if let Some(d) = definiens {
                        e.definiens = Some(d);
                    }
                    if let Some(t) = refined_type {
                        e.refined_type = Some(t);
                    }
                } else {
                    children.push(Assignment {
                        original: source,
                        morphism: uri.clone(),
                        definiens,
                        refined_type,
                        new_name: None,
                        macroname: None,
                        source: self.current_source_range,
                    });
                }
                return Ok(());
            }
        }
        Err(FtmlExtractionError::InvalidIn(
            FtmlKey::Assign,
            "outside of morphisms",
        ))
    }

    fn close_structure(
        &mut self,
        uri: SymbolUri,
        macroname: Option<Id>,
        children: Vec<StructureDeclaration>,
        node: &N,
    ) -> Result<(), FtmlExtractionError> {
        if uri.name().last().starts_with("EXTSTRUCT") {
            return self.close_extension(uri, children, node);
        }
        //get_module!(parent,parent_uri <- self);
        let structure = MathStructure {
            // SAFETY: uri is not is_top() verified above
            uri,
            elements: children.into_boxed_slice(),
            macroname,
            source: self.current_source_range,
        };
        tracing::trace!("New structure {structure:?}");
        let parent_uri = self.push_domain(structure, Declaration::MathStructure, |s| {
            Err(FtmlExtractionError::InvalidIn(
                FtmlKey::MathStructure,
                "other structures",
            ))
        })?;

        let Some(OpenNarrativeElement::MathStructure { uri, children }) = self.narrative.pop()
        else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::MathStructure));
        };
        //add_triples!(DOM self,uri -> parent_uri);
        self.push_elem(DocumentElement::MathStructure {
            range: node.range(),
            structure: uri,
            children: children.into_boxed_slice(),
        });
        Ok(())
    }

    fn close_any_module(
        &mut self,
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<Declaration>,
        node: &N,
    ) -> Result<(), FtmlExtractionError> {
        if uri.is_top() {
            self.close_module(uri, meta, signature, children)?;
        } else {
            self.close_nested_module(uri, children)?;
        }
        let Some(OpenNarrativeElement::Module { uri, children }) = self.narrative.pop() else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Module));
        };
        self.push_elem(DocumentElement::Module {
            range: node.range(),
            module: uri,
            children: children.into_boxed_slice(),
        });
        Ok(())
    }

    fn close_module(
        &mut self,
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<Declaration>,
    ) -> super::Result<()> {
        if !self.domain.is_empty() {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Module));
        }
        let module = ModuleData {
            uri,
            meta_module: meta,
            signature,
            declarations: children.into_boxed_slice(),
            source: self.current_source_range,
        };
        #[cfg(feature = "rdf")]
        {
            use ftml_ontology::Ftml;
            self.rdf.extend(module.triples());
        }
        //add_triples!(NARR self,module -> (self.iri.clone()) ulo:contains);
        self.modules.push(module);
        Ok(())
    }

    fn close_nested_module(
        &mut self,
        uri: ModuleUri,
        children: Vec<Declaration>,
    ) -> super::Result<()> {
        //get_module!(parent,parent_uri <- self);
        let module = NestedModule {
            // SAFETY: uri is not is_top() verified above
            uri: unsafe { uri.into_symbol().unwrap_unchecked() },
            declarations: children.into_boxed_slice(),
            source: self.current_source_range,
        };
        self.push_domain(module, Declaration::NestedModule, |m| {
            Err(FtmlExtractionError::InvalidIn(
                FtmlKey::Module,
                "structures",
            ))
        })?;
        //add_triples!(DOM self,module -> parent_uri);
        //parent.push(Declaration::NestedModule(module));
        Ok(())
    }

    fn close_symbol(&mut self, uri: SymbolUri, data: Box<SymbolData>) -> super::Result<()> {
        let uricl = uri.clone();
        let symbol = Symbol { uri, data };
        tracing::info!("New symbol {symbol:#?}");
        self.push_domain(symbol, Declaration::Symbol, |s| {
            Ok(StructureDeclaration::Symbol(s))
        })?;
        self.push_elem(DocumentElement::SymbolDeclaration(uricl));
        Ok(())
    }

    fn close_inferencerule(&mut self, id: Id, parameters: Vec<Term>) -> super::Result<()> {
        let source = self.current_source_range;
        self.push_domain(
            (id, parameters.into_boxed_slice()),
            |(id, parameters)| Declaration::Rule {
                id,
                parameters,
                source,
            },
            |(id, parameters)| {
                Ok(StructureDeclaration::Rule {
                    id,
                    parameters,
                    source,
                })
            },
        )
        .map(|_| ())
    }

    fn close_vardecl(&mut self, uri: DocumentElementUri, data: Box<VariableData>) {
        let var = VariableDeclaration { uri, data };
        tracing::info!("New variable {var:#?}");
        self.push_elem(DocumentElement::VariableDeclaration(var));
    }

    fn close_definiendum(
        &mut self,
        uri: SymbolUri,
        range: DocumentRange,
    ) -> Result<(), FtmlExtractionError> {
        let mut iter = self.narrative.iter_mut();
        while let Some(e) = iter.next() {
            if let OpenNarrativeElement::Paragraph { fors, .. } = e {
                if !fors.iter().any(|p| p.0 == uri) {
                    fors.push((uri.clone(), None));
                }
                drop(iter);
                self.push_elem(DocumentElement::Definiendum {
                    range,
                    uri,
                    source: self.current_source_range,
                });
                return Ok(());
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiendum))
    }

    fn close_section(
        &mut self,
        uri: DocumentElementUri,
        title: Option<Box<str>>,
        range: DocumentRange,
        children: Vec<DocumentElement>,
    ) {
        let sec = Section {
            uri,
            range,
            title,
            children: children.into_boxed_slice(),
            source: self.current_source_range,
        };
        self.push_elem(DocumentElement::Section(sec));
    }

    fn close_solution(&mut self, id: Option<Id>, node: &N) -> super::Result<()> {
        self.narrative
            .iter_mut()
            .find_map(|e| {
                if let OpenNarrativeElement::Problem { solutions, .. } = e {
                    Some(solutions)
                } else {
                    None
                }
            })
            .map_or(
                Err(FtmlExtractionError::UnexpectedEndOf(
                    FtmlKey::ProblemSolution,
                )),
                |s| {
                    s.push(SolutionData::Solution {
                        html: node.string().into_owned().into_boxed_str(),
                        answer_class: id,
                    });
                    node.children().for_each(|e| {
                        if let Some(either::Either::Left(e)) = e {
                            e.delete();
                        }
                    });
                    Ok(())
                },
            )
    }

    fn close_hint(&mut self, node: &N) -> super::Result<()> {
        self.narrative
            .iter_mut()
            .find_map(|e| {
                if let OpenNarrativeElement::Problem { hints, .. } = e {
                    Some(hints)
                } else {
                    None
                }
            })
            .map_or(
                Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ProblemHint)),
                |s| {
                    let rf = self
                        .buffer
                        .push(&node.string().into_owned().into_boxed_str())
                        .map_err(|e| {
                            FtmlExtractionError::EncodingError(FtmlKey::ProblemHint, e.to_string())
                        })?;
                    s.push(rf);
                    Ok(())
                },
            )
    }

    fn close_exnote(&mut self, node: &N) -> super::Result<()> {
        self.narrative
            .iter_mut()
            .find_map(|e| {
                if let OpenNarrativeElement::Problem { notes, .. } = e {
                    Some(notes)
                } else {
                    None
                }
            })
            .map_or(
                Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ProblemNote)),
                |s| {
                    let rf = self
                        .buffer
                        .push(&node.string().into_owned().into_boxed_str())
                        .map_err(|e| {
                            FtmlExtractionError::EncodingError(FtmlKey::ProblemNote, e.to_string())
                        })?;
                    s.push(rf);
                    node.children().for_each(|e| {
                        if let Some(either::Either::Left(e)) = e {
                            e.delete();
                        }
                    });
                    Ok(())
                },
            )
    }

    fn close_gnote(&mut self, answer_classes: Vec<AnswerClass>, node: &N) -> super::Result<()> {
        self.narrative
            .iter_mut()
            .find_map(|e| {
                if let OpenNarrativeElement::Problem { gnotes, .. } = e {
                    Some(gnotes)
                } else {
                    None
                }
            })
            .map_or(
                Err(FtmlExtractionError::UnexpectedEndOf(
                    FtmlKey::ProblemGradingNote,
                )),
                |s| {
                    let gn = GradingNote {
                        html: node.string().into_owned().into_boxed_str(),
                        answer_classes: answer_classes.into_boxed_slice(),
                    };
                    let rf = self.buffer.push(&gn).map_err(|e| {
                        FtmlExtractionError::EncodingError(
                            FtmlKey::ProblemGradingNote,
                            e.to_string(),
                        )
                    })?;
                    s.push(rf);
                    node.children().for_each(|e| {
                        if let Some(either::Either::Left(e)) = e {
                            e.delete();
                        }
                    });
                    Ok(())
                },
            )
    }

    fn close_answerclass(
        &mut self,
        id: Id,
        kind: AnswerKind,
        feedback: Box<str>,
        node: &N,
    ) -> super::Result<()> {
        let description = node.inner_string().into_owned().into_boxed_str();
        if let Some(OpenNarrativeElement::ProblemGradingNote(acs)) = self.narrative.last_mut() {
            acs.push(AnswerClass {
                id,
                feedback,
                kind,
                description,
            });
            return Ok(());
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::AnswerClass))
    }

    fn close_choice_block(
        &mut self,
        styles: Box<[Id]>,
        block_style: ChoiceBlockStyle,
        multiple: bool,
        choices: Vec<Choice>,
        node: &N,
    ) -> super::Result<()> {
        let key = if multiple {
            FtmlKey::ProblemMultipleChoiceBlock
        } else {
            FtmlKey::ProblemSingleChoiceBlock
        };
        self.narrative
            .iter_mut()
            .find_map(|e| {
                if let OpenNarrativeElement::Problem { solutions, .. } = e {
                    Some(solutions)
                } else {
                    None
                }
            })
            .map_or(Err(FtmlExtractionError::UnexpectedEndOf(key)), |s| {
                let block = SolutionData::ChoiceBlock(ChoiceBlock {
                    multiple,
                    block_style,
                    range: node.range(),
                    styles,
                    choices: choices.into_boxed_slice(),
                });
                s.push(block);
                Ok(())
            })
    }

    fn close_choice(
        &mut self,
        correct: bool,
        verdict: Option<Box<str>>,
        feedback: Box<str>,
    ) -> super::Result<()> {
        if let Some(choices) = self.narrative.iter_mut().find_map(|e| {
            if let OpenNarrativeElement::ChoiceBlock { choices, .. } = e {
                Some(choices)
            } else {
                None
            }
        }) {
            let verdict = match verdict {
                None => (if correct { "correct" } else { "wrong" })
                    .to_string()
                    .into_boxed_str(),
                Some(s) if s.is_empty() => (if correct { "correct" } else { "wrong" })
                    .to_string()
                    .into_boxed_str(),
                Some(s) => s,
            };
            choices.push(Choice {
                correct,
                verdict,
                feedback,
            });
            return Ok(());
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ProblemChoice))
    }

    #[allow(clippy::too_many_arguments)]
    fn close_paragraph(
        &mut self,
        uri: DocumentElementUri,
        kind: ParagraphKind,
        fors: Vec<(SymbolUri, Option<Term>)>,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        children: Vec<DocumentElement>,
        title: Option<Box<str>>,
        range: DocumentRange,
    ) {
        let p = LogicalParagraph {
            kind,
            uri,
            formatting,
            title,
            range,
            styles,
            children: children.into_boxed_slice(),
            fors: fors.into_boxed_slice(),
            source: self.current_source_range,
        };
        if self
            .narrative
            .iter()
            .any(|e| matches!(e, OpenNarrativeElement::Solution(_)))
        {
            tracing::info!("Skipping paragraph in solutions block");
            return;
        }
        tracing::info!("Adding paragraph {p:#?}");
        self.push_elem(DocumentElement::Paragraph(p));
    }

    #[allow(clippy::too_many_arguments)]
    fn close_slide(
        &mut self,
        uri: DocumentElementUri,
        children: Vec<DocumentElement>,
        title: Option<Box<str>>,
        range: DocumentRange,
    ) {
        let p = DocumentElement::Slide(Slide {
            uri,
            title,
            range,
            children: children.into_boxed_slice(),
            source: self.current_source_range,
        });
        tracing::info!("Adding slide {p:#?}");
        self.push_elem(p);
    }

    #[allow(clippy::too_many_arguments)]
    fn close_notation(
        &mut self,
        uri: DocumentElementUri,
        id: Option<Id>,
        head: VarOrSym,
        prec: i64,
        argprecs: Vec<i64>,
        component: Option<NotationComponent>,
        op: Option<NotationNode>,
    ) -> super::Result<()> {
        let Some(component) = component else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Notation));
        };
        let not = Notation {
            id,
            precedence: prec,
            argprecs,
            component,
            op,
        };
        tracing::info!("New notation for {head:?}: {not:#?}");
        let notation = self
            .buffer
            .push(&not)
            .map_err(|e| FtmlExtractionError::EncodingError(FtmlKey::Notation, e.to_string()))?;

        let (e, leaf) = match head {
            VarOrSym::Sym(s) => (
                DocumentElement::Notation(NotationReference {
                    symbol: s.clone(),
                    uri: uri.clone(),
                    notation,
                    source: self.current_source_range,
                }),
                s.into(),
            ),
            VarOrSym::Var(Variable::Ref { declaration, .. }) => (
                DocumentElement::VariableNotation(VariableNotationReference {
                    variable: declaration.clone(),
                    uri: uri.clone(),
                    notation,
                    source: self.current_source_range,
                }),
                declaration.into(),
            ),
            VarOrSym::Var(_) => {
                return Err(FtmlExtractionError::InvalidValue(FtmlKey::Notation));
            }
        };
        self.notations.push((leaf, uri, not));
        self.push_elem(e);
        Ok(())
    }

    fn close_notation_component(
        &mut self,
        node: &N,
        components: Vec<(NotationComponent, crate::NodePath)>,
    ) -> super::Result<()> {
        let not = node.as_notation(components)?;
        if let Some(OpenNarrativeElement::Notation { component, .. }) = self.narrative.last_mut() {
            *component = Some(not);
            return Ok(());
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::NotationComp))
    }

    fn close_notation_arg(&mut self, node: &N, position: ArgumentPosition) -> super::Result<()> {
        let mode = position.mode();
        let index = position.index();
        let Some((components, ancestor)) = self.narrative.iter_mut().find_map(|e| match e {
            OpenNarrativeElement::NotationComp {
                components,
                node: ancestor,
            }
            | OpenNarrativeElement::ArgSep {
                components,
                node: ancestor,
            } => Some((components, ancestor)),
            _ => None,
        }) else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Arg));
        };
        let path = node.path_from(ancestor);
        components.push((NotationComponent::Argument { index, mode }, path));
        Ok(())
    }

    fn close_argsep(
        &mut self,
        node: &N,
        components: Vec<(NotationComponent, crate::NodePath)>,
    ) -> super::Result<()> {
        let (index, mode, sep) = match node.as_notation(components)? {
            NotationComponent::Node {
                tag,
                attributes,
                children,
            } => match children.first() {
                Some(NotationComponent::Argument { index, mode }) => {
                    let index = *index;
                    let mode = *mode;
                    let mut children = children.into_vec();
                    children.remove(0);
                    (index, mode, children.into_boxed_slice())
                }
                _ => return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ArgSep)),
            },
            _ => return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ArgSep)),
        };
        if let Some(
            OpenNarrativeElement::NotationComp {
                components,
                node: ancestor,
            }
            | OpenNarrativeElement::ArgSep {
                components,
                node: ancestor,
            },
        ) = self.narrative.last_mut()
        {
            let path = node.path_from(ancestor);
            components.push((NotationComponent::ArgSep { index, mode, sep }, path));
            Ok(())
        } else {
            Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::NotationComp))
        }
    }

    fn close_notation_op(&mut self, node: &N) -> super::Result<()> {
        let component = node.as_component()?;
        if let Some(OpenNarrativeElement::Notation { op, .. }) = self.narrative.last_mut() {
            *op = Some(component);
            return Ok(());
        }
        Err(FtmlExtractionError::UnexpectedEndOf(
            FtmlKey::NotationOpComp,
        ))
    }

    fn close_comp_in_notation(&mut self, node: &N, is_main: bool) -> super::Result<()> {
        let component = node.as_component()?;
        if is_main {
            for e in self.narrative.iter_mut() {
                match e {
                    OpenNarrativeElement::Invisible
                    | OpenNarrativeElement::NotationComp { .. }
                    | OpenNarrativeElement::ArgSep { .. } => {}
                    OpenNarrativeElement::Notation { op, .. } => {
                        *op = Some(component.clone());
                        break;
                    }
                    o => {
                        return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Comp));
                    }
                }
            }
        }
        match self.narrative.last_mut() {
            Some(
                OpenNarrativeElement::NotationComp {
                    node: ancestor,
                    components,
                }
                | OpenNarrativeElement::ArgSep {
                    node: ancestor,
                    components,
                },
            ) => {
                let comp = if is_main {
                    NotationComponent::MainComp(component)
                } else {
                    NotationComponent::Comp(component)
                };
                components.push((comp, node.path_from(ancestor)));
                Ok(())
            }
            Some(OpenNarrativeElement::Notation { .. }) if is_main => Ok(()),
            _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Comp)),
        }
    }

    fn close_problem_title(&mut self, node: &N) -> super::Result<()> {
        for e in self.narrative.iter_mut() {
            match e {
                OpenNarrativeElement::Problem { title, .. } if title.is_none() => {
                    let str = node.inner_string();
                    if !str.is_empty() {
                        *title = Some(node.inner_string().into_owned().into_boxed_str());
                    }
                    return Ok(());
                }
                OpenNarrativeElement::Problem { title, .. } => {
                    return Err(FtmlExtractionError::DuplicateValue(FtmlKey::Title));
                }
                OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::NotationArg(_) => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title));
                }
                OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::Morphism { .. }
                | OpenNarrativeElement::Invisible => (),
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title))
    }

    fn close_section_title(&mut self, node: &N) -> super::Result<()> {
        for e in self.narrative.iter_mut() {
            match e {
                OpenNarrativeElement::Section { title, .. } if title.is_none() => {
                    let str = node.inner_string();
                    if !str.is_empty() {
                        *title = Some(node.inner_string().into_owned().into_boxed_str());
                    }
                    return Ok(());
                }
                OpenNarrativeElement::Section { title, .. } => {
                    return Err(FtmlExtractionError::DuplicateValue(FtmlKey::Title));
                }
                OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Problem { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::NotationArg(_) => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title));
                }
                OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::Morphism { .. }
                | OpenNarrativeElement::Invisible => (),
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title))
    }

    fn close_paragraph_title(&mut self, node: &N) -> super::Result<()> {
        for e in self.narrative.iter_mut() {
            match e {
                OpenNarrativeElement::Paragraph { title, .. } if title.is_none() => {
                    let str = node.inner_string();
                    if !str.is_empty() {
                        *title = Some(node.inner_string().into_owned().into_boxed_str());
                    }
                    return Ok(());
                }
                OpenNarrativeElement::Paragraph { title, .. } => {
                    return Err(FtmlExtractionError::DuplicateValue(FtmlKey::Title));
                }
                OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Problem { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::NotationArg(_) => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title));
                }
                OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::Morphism { .. }
                | OpenNarrativeElement::Invisible => (),
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title))
    }

    fn close_slide_title(&mut self, node: &N) -> super::Result<()> {
        for e in self.narrative.iter_mut() {
            match e {
                OpenNarrativeElement::Slide { title, .. } if title.is_none() => {
                    let str = node.inner_string();
                    if !str.is_empty() {
                        *title = Some(node.inner_string().into_owned().into_boxed_str());
                    }
                    return Ok(());
                }
                OpenNarrativeElement::Slide { title, .. } => {
                    return Err(FtmlExtractionError::DuplicateValue(FtmlKey::Title));
                }
                OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Problem { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::NotationArg(_) => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title));
                }
                OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::Morphism { .. }
                | OpenNarrativeElement::Invisible => (),
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title))
    }

    fn close_argument(
        &mut self,
        position: ArgumentPosition,
        terms: Vec<(Term, crate::NodePath)>,
        node: &N,
    ) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::OMA { arguments, .. }) => {
                OpenArgument::set(arguments, position, term)
            }
            Some(OpenDomainElement::OMBIND { arguments, .. }) => {
                OpenBoundArgument::set(arguments, position, term)
            }
            Some(OpenDomainElement::InferenceRule { parameters, .. }) => {
                parameters.push(term);
                Ok(())
            }
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::ArgTypes(_)
                | OpenDomainElement::ReturnType { .. }
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::MathStructure { .. }
                | OpenDomainElement::Morphism { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::ComplexTerm { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::Comp
                | OpenDomainElement::DefComp
                | OpenDomainElement::VariableReference { .. },
            ) => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Arg)),
        }
    }

    fn close_argtypes(&mut self, terms: Vec<Term>, node: &N) -> super::Result<()> {
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data }) if data.tp.is_none() => {
                data.argument_types = terms.into_boxed_slice();
                return Ok(());
            }
            _ => (),
        }

        for n in self.narrative.iter_mut() {
            match n {
                OpenNarrativeElement::VariableDeclaration { uri, data } if data.tp.is_none() => {
                    data.argument_types = terms.into_boxed_slice();
                    return Ok(());
                }
                _ => (),
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::ArgTypes))
    }

    fn close_type(&mut self, terms: Vec<(Term, crate::NodePath)>, node: &N) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data }) if data.tp.is_none() => {
                data.tp = TermContainer::new(term, None);
                return Ok(());
            }
            Some(OpenDomainElement::OML { tp, .. }) if tp.is_none() => {
                *tp = Some(term);
                return Ok(());
            }
            Some(OpenDomainElement::Assign { refined_type, .. }) if refined_type.is_none() => {
                *refined_type = Some(term);
                return Ok(());
            }
            Some(OpenDomainElement::ArgTypes(v)) => {
                v.push(term);
                return Ok(());
            }
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::ReturnType { .. }
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::InferenceRule { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::ComplexTerm { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::MathStructure { .. }
                | OpenDomainElement::Morphism { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::Comp
                | OpenDomainElement::DefComp
                | OpenDomainElement::VariableReference { .. },
            ) => (),
        }
        for n in self.narrative.iter_mut() {
            match n {
                OpenNarrativeElement::VariableDeclaration { uri, data } if data.tp.is_none() => {
                    data.tp = TermContainer::new(term, None);
                    return Ok(());
                }
                OpenNarrativeElement::Invisible => (),
                OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::Problem { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::NotationArg(_)
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::Morphism { .. } => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type));
                }
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type))
    }

    fn close_return_type(
        &mut self,
        terms: Vec<(Term, crate::NodePath)>,
        node: &N,
    ) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data })
                if data.return_type.is_none() =>
            {
                data.return_type = Some(term);
                return Ok(());
            }
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::ReturnType { .. }
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::InferenceRule { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::ComplexTerm { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::MathStructure { .. }
                | OpenDomainElement::Morphism { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::Comp
                | OpenDomainElement::DefComp
                | OpenDomainElement::ArgTypes(_)
                | OpenDomainElement::VariableReference { .. },
            ) => (),
        }
        for n in self.narrative.iter_mut() {
            match n {
                OpenNarrativeElement::VariableDeclaration { uri, data } if data.tp.is_none() => {
                    data.return_type = Some(term);
                    return Ok(());
                }
                OpenNarrativeElement::Invisible => (),
                OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Problem { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::NotationArg(_)
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::Morphism { .. } => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type));
                }
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type))
    }

    #[allow(clippy::too_many_lines)]
    fn close_definiens(
        &mut self,
        terms: Vec<(Term, crate::NodePath)>,
        of: Option<SymbolUri>,
        node: &N,
    ) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data }) if data.df.is_none() => {
                data.df = TermContainer::new(term, None);
                return Ok(());
            }
            Some(OpenDomainElement::OML { df, .. }) if df.is_none() => {
                *df = Some(term);
                return Ok(());
            }
            Some(OpenDomainElement::Assign { definiens, .. }) if definiens.is_none() => {
                *definiens = Some(term);
                return Ok(());
            }
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::ReturnType { .. }
                | OpenDomainElement::ArgTypes(_)
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::InferenceRule { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::ComplexTerm { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::MathStructure { .. }
                | OpenDomainElement::Morphism { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::Comp
                | OpenDomainElement::DefComp
                | OpenDomainElement::VariableReference { .. },
            ) => (),
        }

        for n in self.narrative.iter_mut() {
            match n {
                OpenNarrativeElement::VariableDeclaration { uri, data } if data.df.is_none() => {
                    data.df = TermContainer::new(term, None);
                    return Ok(());
                }
                OpenNarrativeElement::Paragraph {
                    kind, styles, fors, ..
                } if kind.is_definition_like(styles) => {
                    if let Some(of) = of {
                        if let Some(data) = Self::find_symbol(&mut self.domain, &of)
                            && data.df.is_none()
                        {
                            data.df = TermContainer::new(term.clone(), None);
                        }
                        if let Some((a, b)) = fors.iter_mut().find(|(k, v)| *k == of) {
                            *b = Some(term);
                        } else {
                            fors.push((of, Some(term)));
                        }
                    } else if let Some((k, v)) = fors.first_mut() {
                        if let Some(data) = Self::find_symbol(&mut self.domain, k)
                            && data.df.is_none()
                        {
                            data.df = TermContainer::new(term.clone(), None);
                        }
                        *v = Some(term);
                    } else {
                        return Err(FtmlExtractionError::DuplicateValue(FtmlKey::Definiens));
                    }
                    return Ok(());
                }
                OpenNarrativeElement::Invisible => (),
                OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::Problem { .. }
                | OpenNarrativeElement::Solution(..)
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::NotationArg(_)
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
                | OpenNarrativeElement::FillinSol { .. }
                | OpenNarrativeElement::ProblemHint
                | OpenNarrativeElement::ProblemExNote
                | OpenNarrativeElement::ProblemGradingNote(_)
                | OpenNarrativeElement::AnswerClass { .. }
                | OpenNarrativeElement::ChoiceBlock { .. }
                | OpenNarrativeElement::ProblemChoice { .. }
                | OpenNarrativeElement::ProblemChoiceVerdict
                | OpenNarrativeElement::ProblemChoiceFeedback
                | OpenNarrativeElement::FillinSolCase(_)
                | OpenNarrativeElement::Morphism { .. } => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiens));
                }
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiens))
    }

    fn find_symbol<'a>(
        domain: &'a mut StackVec<OpenDomainElement<N>>,
        uri: &SymbolUri,
    ) -> Option<&'a mut SymbolData> {
        // TODO could be optimized to only traverse those where the URI actually matches
        for d in domain.iter_mut() {
            match d {
                OpenDomainElement::MathStructure { children, .. } => {
                    if let Some(c) = Self::find_content_ii(&mut *children, uri) {
                        return Some(c);
                    }
                }
                OpenDomainElement::Module { children, .. } => {
                    if let Some(c) = Self::find_content_i(&mut *children, uri) {
                        return Some(c);
                    }
                }
                OpenDomainElement::SymbolDeclaration { data, uri: u } if *u == *uri => {
                    return Some(data);
                }
                OpenDomainElement::Morphism { .. }
                | OpenDomainElement::Argument { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::Comp
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::ComplexTerm { .. }
                | OpenDomainElement::DefComp
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::InferenceRule { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::ReturnType { .. }
                | OpenDomainElement::ArgTypes(_)
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::VariableReference { .. }
                | OpenDomainElement::Type { .. } => (),
            }
        }
        None
    }

    fn find_content_i<'a>(
        domain: &'a mut [Declaration],
        uri: &SymbolUri,
    ) -> Option<&'a mut SymbolData> {
        for d in domain {
            match d {
                Declaration::Extension(ext) => {
                    if let Some(c) = Self::find_content_ii(&mut ext.elements, uri) {
                        return Some(c);
                    }
                }
                Declaration::MathStructure(s) => {
                    if let Some(c) = Self::find_content_ii(&mut s.elements, uri) {
                        return Some(c);
                    }
                }
                Declaration::NestedModule(m) => {
                    if let Some(c) = Self::find_content_i(&mut m.declarations, uri) {
                        return Some(c);
                    }
                }
                Declaration::Symbol(s) if s.uri == *uri => return Some(&mut s.data),
                Declaration::Symbol(_)
                | Declaration::Import { .. }
                | Declaration::Morphism(_)
                | Declaration::Rule { .. } => (),
            }
        }
        None
    }

    fn find_content_ii<'a>(
        domain: &'a mut [StructureDeclaration],
        uri: &SymbolUri,
    ) -> Option<&'a mut SymbolData> {
        for d in domain {
            match d {
                StructureDeclaration::Symbol(s) if s.uri == *uri => return Some(&mut s.data),
                StructureDeclaration::Symbol(_)
                | StructureDeclaration::Rule { .. }
                | StructureDeclaration::Import { .. }
                | StructureDeclaration::Morphism(_) => (),
            }
        }
        None
    }

    fn close_head_term(
        &mut self,
        terms: Vec<(Term, crate::NodePath)>,
        node: &N,
    ) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        tracing::trace!("Closed head term: {term:?}");
        if let Some(
            OpenDomainElement::ComplexTerm {
                head_term: ht @ None,
                ..
            }
            | OpenDomainElement::OMA {
                head_term: ht @ None,
                ..
            }
            | OpenDomainElement::OMBIND {
                head_term: ht @ None,
                ..
            },
        ) = &mut self.domain.last
        {
            *ht = Some(term);
            return Ok(());
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::HeadTerm))
    }

    #[allow(clippy::match_same_arms)]
    fn close_term(
        &mut self,
        term: Term,
        node: &N,
        otherwise: impl FnOnce(&mut Self, Term) -> super::Result<()>,
    ) -> super::Result<()> {
        match &mut self.domain.last {
            Some(
                OpenDomainElement::Module { .. }
                | OpenDomainElement::MathStructure { .. }
                | OpenDomainElement::Morphism { .. }
                | OpenDomainElement::InferenceRule { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::ArgTypes(_)
                | OpenDomainElement::VariableReference { .. },
            )
            | None => (),
            Some(
                OpenDomainElement::Argument {
                    terms,
                    node: ancestor,
                    ..
                }
                | OpenDomainElement::Type {
                    terms,
                    node: ancestor,
                }
                | OpenDomainElement::ReturnType {
                    terms,
                    node: ancestor,
                }
                | OpenDomainElement::Definiens {
                    terms,
                    node: ancestor,
                    ..
                }
                | OpenDomainElement::HeadTerm {
                    terms,
                    node: ancestor,
                },
            ) => {
                terms.push((term, node.path_from(ancestor)));
                return Ok(());
            }
            Some(OpenDomainElement::ComplexTerm { .. }) => {
                // TODO forget about it?
                return Ok(());
            }
            Some(OpenDomainElement::Comp | OpenDomainElement::DefComp) => {
                return Ok(());
                // this is incompatible with \this in stex:
                /*
                tracing::debug!("Error: {:?}", self.domain);
                return Err(FtmlExtractionError::InvalidIn(
                    FtmlKey::Term,
                    "declarations or terms outside of an argument",
                ));
                 */
            }
        }
        otherwise(self, term)
    }

    fn close_app_term(
        &mut self,
        uri: Option<DocumentElementUri>,
        term: Term,
        node: &N,
    ) -> super::Result<()> {
        self.close_term(term, node, |slf, term| {
            uri.map_or_else(
                || {
                    tracing::debug!("Error: 1");
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                },
                |uri| {
                    slf.push_elem(DocumentElement::Term(DocumentTerm::new(uri, term, None)));
                    Ok(())
                },
            )
        })
    }

    fn close_oma(
        &mut self,
        head: VarOrSym,
        head_term: Option<Term>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenArgument>,
        node: &N,
    ) -> super::Result<()> {
        let mut args = Vec::with_capacity(arguments.len());
        for (i, a) in arguments.into_iter().enumerate() {
            if let Some(a) = a.close() {
                args.push(a);
            } else {
                return Err(FtmlExtractionError::MissingArgument(i + 1));
            }
        }
        let (head, presentation) = if let Some(t) = head_term {
            (t, Some(head))
        } else {
            (
                match head {
                    VarOrSym::Sym(s) => Term::Symbol {
                        uri: s,
                        presentation: None,
                    },
                    VarOrSym::Var(v) => Term::Var {
                        variable: v,
                        presentation: None,
                    },
                },
                None,
            )
        };
        let term = Term::Application(ApplicationTerm::new(
            head,
            args.into_boxed_slice(),
            presentation,
        ))
        .simplify();
        self.close_app_term(uri, term, node)
    }

    fn close_ombind(
        &mut self,
        head: VarOrSym,
        head_term: Option<Term>,
        uri: Option<DocumentElementUri>,
        arguments: Vec<OpenBoundArgument>,
        node: &N,
    ) -> super::Result<()> {
        tracing::info!("Closing OMBIND {head:?} ({head_term:?}) @ {arguments:?}");
        /*
        let body = match arguments.pop() {
            Some(
                OpenBoundArgument::Simple {
                    term: body,
                    should_be_var: false,
                }
                | OpenBoundArgument::Sequence {
                    terms: either::Either::Left(body),
                    should_be_var: false,
                },
            ) => body,
            Some(OpenBoundArgument::Sequence {
                terms: either::Either::Right(terms),
                should_be_var: false,
            }) if terms.iter().all(Option::is_some) =>
            // SAFETY: pattern match
            {
                Term::into_seq(unsafe { terms.into_iter().map(|e| e.unwrap_unchecked()) })
            }
            _ => return Err(FtmlExtractionError::MissingArgument(arguments.len())),
        };
         */
        let mut args = Vec::with_capacity(arguments.len());
        for (i, a) in arguments.into_iter().enumerate() {
            if let Some(a) = a.close() {
                args.push(a);
            } else {
                return Err(FtmlExtractionError::MissingArgument(i + 1));
            }
        }

        let (head, presentation) = if let Some(t) = head_term {
            (t, Some(head))
        } else {
            (
                match head {
                    VarOrSym::Sym(s) => Term::Symbol {
                        uri: s,
                        presentation: None,
                    },
                    VarOrSym::Var(v) => Term::Var {
                        variable: v,
                        presentation: None,
                    },
                },
                None,
            )
        };
        let term = Term::Bound(BindingTerm::new(
            head,
            args.into_boxed_slice(),
            //body,
            presentation,
        ))
        .simplify();
        self.close_app_term(uri, term, node)
    }

    fn close_complex(
        &mut self,
        head: VarOrSym,
        head_term: Option<Term>,
        uri: Option<DocumentElementUri>,
        node: &N,
    ) -> super::Result<()> {
        let Some(term) = head_term else {
            return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term));
        };
        let term = term.with_presentation(head);
        self.close_term(term, node, |slf, term| {
            uri.map_or_else(
                || {
                    tracing::debug!("Error: 1");
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                },
                |uri| {
                    slf.push_elem(DocumentElement::Term(DocumentTerm::new(uri, term, None)));
                    Ok(())
                },
            )
        })
    }

    fn close_oms(&mut self, uri: SymbolUri, notation: Option<Id>, node: &N) -> super::Result<()> {
        let source = self.current_source_range;
        self.close_term(
            Term::Symbol {
                uri,
                presentation: None,
            },
            node,
            |slf, term| {
                let Term::Symbol { uri, .. } = term else {
                    // SAFETY: close_term returns the same term
                    unsafe { unreachable_unchecked() }
                };
                slf.push_elem(DocumentElement::SymbolReference {
                    range: node.range(),
                    uri,
                    notation,
                    source,
                });
                Ok(())
            },
        )
    }

    fn close_omv(&mut self, var: Variable, notation: Option<Id>, node: &N) -> super::Result<()> {
        let source = self.current_source_range;
        self.close_term(
            Term::Var {
                variable: var,
                presentation: None,
            },
            node,
            |slf, term| {
                let Term::Var { variable: var, .. } = term else {
                    // SAFETY: close_term returns the same term
                    unsafe { unreachable_unchecked() }
                };
                let uri = match var {
                    Variable::Name { .. } => return Ok(()),
                    Variable::Ref {
                        declaration,
                        is_sequence,
                    } => declaration,
                };
                slf.push_elem(DocumentElement::VariableReference {
                    range: node.range(),
                    uri,
                    notation,
                    source,
                });
                Ok(())
            },
        )
    }
}
