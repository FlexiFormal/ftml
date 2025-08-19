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
            DocumentElement, DocumentTerm, LogicalParagraph, Notation, Section, SectionLevel,
            VariableDeclaration,
            notations::{NotationComponent, NotationNode},
            paragraphs::{ParagraphFormatting, ParagraphKind},
            variables::VariableData,
        },
    },
    terms::{Term, VarOrSym, Variable},
};

use ftml_uris::{
    DocumentElementUri, DocumentUri, DomainUriRef, Id, IsDomainUri, Language, LeafUri, ModuleUri,
    SymbolUri,
};
use std::{borrow::Cow, hint::unreachable_unchecked};

#[derive(Clone, Debug)]
pub struct IdCounter {
    inner: rustc_hash::FxHashMap<Cow<'static, str>, u32>,
}
impl Default for IdCounter {
    fn default() -> Self {
        let mut inner = rustc_hash::FxHashMap::default();
        inner.insert("EXTSTRUCT".into(), 0);
        Self { inner }
    }
}
impl IdCounter {
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
    pub domain: StackVec<OpenDomainElement<N>>,
    pub narrative: StackVec<OpenNarrativeElement<N>>,
    pub kind: DocumentKind,
    top_section_level: Option<SectionLevel>,
    ids: IdCounter,
    #[allow(dead_code)]
    do_rdf: bool,
    #[cfg(feature = "rdf")]
    rdf: rustc_hash::FxHashSet<ulo::rdf_types::Triple>,
}
/*
macro_rules! add_triples {
    (DOM $self:ident,$elem:ident -> $module:ident) => {
        add_triples!(NARR $self,$elem -> ($module.to_iri()) ulo:declares)
    };
    (NARR $self:ident,$elem:ident -> ($module:expr) $p:ident:$r:ident) => {
        #[cfg(feature = "rdf")]
        if $self.do_rdf {
            use ftml_ontology::Ftml;
            use ftml_uris::FtmlUri;
            use ulo::triple;
            $self.rdf
                .extend($elem.triples().into_iter().chain(std::iter::once(
                    triple!(<($module)> $p:$r <($elem.uri.to_iri())>),
                )));
        }
    }
}
*/

#[derive(Debug)]
pub struct ExtractionResult {
    pub document: Document,
    pub modules: Vec<Module>,
    pub data: Box<[u8]>,
    #[cfg(feature = "rdf")]
    pub triples: rustc_hash::FxHashSet<ulo::rdf_types::Triple>,
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
            buffer: DataBuffer::default(),
            top: Vec::new(),
            modules: Vec::new(),
            kind: DocumentKind::default(),
            domain: StackVec::default(),
            narrative: StackVec::default(),
            #[cfg(feature = "rdf")]
            rdf: rustc_hash::FxHashSet::default(),
        }
    }

    pub fn finish(&mut self) -> ExtractionResult {
        use std::mem::take;
        let document = DocumentData {
            uri: self.document.clone(),
            title: take(&mut self.title), // todo
            elements: take(&mut self.top).into_boxed_slice(),
            top_section_level: self.top_section_level.unwrap_or_default(),
            kind: self.kind,
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
            AnyOpen::Meta(m) => match m {
                MetaDatum::DocumentKind(k) => self.kind = k,
                MetaDatum::Style(s) => self.styles.push(s),
                MetaDatum::Counter(c) => self.counters.push(c),
                MetaDatum::InputRef { target, uri } => {
                    self.push_elem(DocumentElement::DocumentReference { uri, target });
                }
                MetaDatum::SetSectionLevel(lvl) => {
                    self.top_section_level = Some(lvl);
                }
                MetaDatum::UseModule(uri) => self.push_elem(DocumentElement::UseModule(uri)),
                MetaDatum::ImportModule(uri) => {
                    self.push_domain(uri.clone(), Declaration::Import, |uri| {
                        Ok(StructureDeclaration::Import(uri))
                    })?;
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
                            });
                        }
                    } else {
                        return Err(FtmlExtractionError::InvalidIn(
                            FtmlKey::Rename,
                            "outside of morphisms",
                        ));
                    }
                }
                MetaDatum::IfInputref(_) => (),
            },
            AnyOpen::None => (),
        }
        Ok(())
    }

    /// ### Errors
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    pub fn close(&mut self, elem: CloseFtmlElement, node: &N) -> super::Result<()> {
        tracing::debug!("Closing: {elem:?} in {:?}", self.domain);
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
            CloseFtmlElement::SectionTitle => self.close_section_title(node),
            CloseFtmlElement::ParagraphTitle => self.close_paragraph_title(node),
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
                node.delete();
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
        let uri = match self.domain.last_mut() {
            Some(OpenDomainElement::Module { uri, children, .. }) => {
                let elem = module(data);
                #[cfg(feature = "rdf")]
                {
                    use ftml_ontology::Ftml;
                    self.rdf.extend(elem.triples());
                }
                children.push(elem);
                DomainUriRef::Module(uri)
            }
            Some(OpenDomainElement::MathStructure { uri, children, .. }) => {
                let elem = structure(data)?;
                #[cfg(feature = "rdf")]
                {
                    use ftml_ontology::Ftml;
                    self.rdf.extend(elem.triples());
                }
                children.push(elem);
                DomainUriRef::Symbol(uri)
            }
            _ => {
                return Err(FtmlExtractionError::NotIn(
                    FtmlKey::Module,
                    "a module or structure (or inside of a declaration)",
                ));
            }
        };
        Ok(uri)
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
            matches!(i,StructureDeclaration::Import(i) if !i.module_name().last().starts_with("EXTSTRUCT"))
        ) else {
            return Err(FtmlExtractionError::MissingArgument(0))
        };
        let StructureDeclaration::Import(i) = children.remove(i) else {
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
        if let Some(OpenDomainElement::Morphism { children, uri, .. }) = self.domain.last_mut() {
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
                });
            }
            Ok(())
        } else {
            Err(FtmlExtractionError::InvalidIn(
                FtmlKey::Assign,
                "outside of morphisms",
            ))
        }
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
        //get_module!(parent,parent_uri <- self);
        let uricl = uri.clone();
        let symbol = Symbol { uri, data };
        tracing::info!("New symbol {symbol:#?}");
        self.push_domain(symbol, Declaration::Symbol, |s| {
            Ok(StructureDeclaration::Symbol(s))
        })?;
        self.push_elem(DocumentElement::SymbolDeclaration(uricl));
        //add_triples!(DOM self,symbol -> parent_uri);
        //parent.push(Declaration::Symbol(symbol));
        Ok(())
    }

    fn close_vardecl(&mut self, uri: DocumentElementUri, data: Box<VariableData>) {
        let var = VariableDeclaration { uri, data };
        tracing::info!("New variable {var:#?}");
        self.push_elem(DocumentElement::VariableDeclaration(var));
        //add_triples!(DOM self,symbol -> parent_uri);
        //parent.push(Declaration::Symbol(symbol));
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
                self.push_elem(DocumentElement::Definiendum { range, uri });
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
        };
        self.push_elem(DocumentElement::Section(sec));
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
        };
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
        let p = DocumentElement::Slide {
            uri,
            title,
            range,
            children: children.into_boxed_slice(),
        };
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
                DocumentElement::Notation {
                    symbol: s.clone(),
                    uri: uri.clone(),
                    notation,
                },
                s.into(),
            ),
            VarOrSym::Var(Variable::Ref { declaration, .. }) => (
                DocumentElement::VariableNotation {
                    variable: declaration.clone(),
                    uri: uri.clone(),
                    notation,
                },
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
            components.push((NotationComponent::Argument { index, mode }, path));
            Ok(())
        } else {
            Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Arg))
        }
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
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
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
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
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
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Paragraph { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::Definiendum(_)
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
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::Type { .. }
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

    fn close_type(&mut self, terms: Vec<(Term, crate::NodePath)>, node: &N) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data }) if data.tp.is_none() => {
                data.tp = Some(term);
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
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::HeadTerm { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::ReturnType { .. }
                | OpenDomainElement::Definiens { .. }
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
                    data.tp = Some(term);
                    return Ok(());
                }
                OpenNarrativeElement::Invisible => (),
                OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::Paragraph { .. }
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
                    data.tp = Some(term);
                    return Ok(());
                }
                OpenNarrativeElement::Invisible => (),
                OpenNarrativeElement::Section { .. }
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
                | OpenNarrativeElement::Morphism { .. } => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type));
                }
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type))
    }

    fn close_definiens(
        &mut self,
        terms: Vec<(Term, crate::NodePath)>,
        of: Option<SymbolUri>,
        node: &N,
    ) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data }) if data.df.is_none() => {
                data.df = Some(term);
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
                | OpenDomainElement::Definiens { .. }
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
                    data.df = Some(term);
                    return Ok(());
                }
                OpenNarrativeElement::Paragraph {
                    kind, styles, fors, ..
                } if kind.is_definition_like(styles) => {
                    if let Some(of) = of {
                        if let Some(data) = Self::find_symbol(&mut self.domain, &of)
                            && data.df.is_none()
                        {
                            data.df = Some(term.clone());
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
                            data.df = Some(term.clone());
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
                | OpenNarrativeElement::Slide { .. }
                | OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::NotationArg(_)
                | OpenNarrativeElement::Definiendum(_)
                | OpenNarrativeElement::Module { .. }
                | OpenNarrativeElement::MathStructure { .. }
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
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::ReturnType { .. }
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
                Declaration::Symbol(_) | Declaration::Import(_) | Declaration::Morphism(_) => (),
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
                | StructureDeclaration::Import(_)
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
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::OML { .. }
                | OpenDomainElement::Assign { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
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
                tracing::debug!("Error: {:?}", self.domain);
                return Err(FtmlExtractionError::InvalidIn(
                    FtmlKey::Term,
                    "declarations or terms outside of an argument",
                ));
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
                    slf.push_elem(DocumentElement::Term(DocumentTerm { uri, term }));
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
        let term = Term::Application {
            head: Box::new(head),
            arguments: args.into_boxed_slice(),
            presentation,
        }
        .simplify();
        self.close_app_term(uri, term, node)
    }

    fn close_ombind(
        &mut self,
        head: VarOrSym,
        head_term: Option<Term>,
        uri: Option<DocumentElementUri>,
        mut arguments: Vec<OpenBoundArgument>,
        node: &N,
    ) -> super::Result<()> {
        let Some(OpenBoundArgument::Simple {
            term: body,
            should_be_var: false,
        }) = arguments.pop()
        else {
            return Err(FtmlExtractionError::MissingArgument(arguments.len()));
        };
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
        let term = Term::Bound {
            head: Box::new(head),
            body: Box::new(body),
            arguments: args.into_boxed_slice(),
            presentation,
        }
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
                    slf.push_elem(DocumentElement::Term(DocumentTerm { uri, term }));
                    Ok(())
                },
            )
        })
    }

    fn close_oms(&mut self, uri: SymbolUri, notation: Option<Id>, node: &N) -> super::Result<()> {
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
                });
                Ok(())
            },
        )
    }

    fn close_omv(&mut self, var: Variable, notation: Option<Id>, node: &N) -> super::Result<()> {
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
                });
                Ok(())
            },
        )
    }
}
