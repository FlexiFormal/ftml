use crate::{
    FtmlKey,
    extraction::{
        ArgumentPosition, CloseFtmlElement, FtmlExtractionError, MetaDatum, OpenArgument,
        OpenBoundArgument, OpenDomainElement, OpenFtmlElement, OpenNarrativeElement, Split,
        VarOrSym, nodes::FtmlNode,
    },
};
use ftml_ontology::{
    domain::{
        declarations::{
            Declaration,
            symbols::{Symbol, SymbolData},
        },
        modules::{Module, ModuleData, NestedModule},
    },
    narrative::{
        DataBuffer, DocumentRange,
        documents::{Document, DocumentCounter, DocumentData, DocumentStyle, DocumentStyles},
        elements::{
            DocumentElement, DocumentTerm, Notation, Section, VariableDeclaration,
            notations::{NotationComponent, NotationNode},
            variables::VariableData,
        },
    },
    terms::{Term, Variable},
};
#[cfg(feature = "rdf")]
use ftml_uris::FtmlUri;
use ftml_uris::{DocumentElementUri, DocumentUri, Id, Language, LeafUri, ModuleUri, SymbolUri};
use smallvec::SmallVec;
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
    ids: IdCounter,
    #[allow(dead_code)]
    do_rdf: bool,
    #[cfg(feature = "rdf")]
    rdf: rustc_hash::FxHashSet<ulo::rdf_types::Triple>,
    #[cfg(feature = "rdf")]
    iri: ulo::rdf_types::NamedNode,
}

macro_rules! get_module {
    ($children:ident,$uri:ident <- $self:ident) => {
        let Some(OpenDomainElement::Module {
            children: $children,
            uri: $uri,
            ..
        }) = $self.domain.last_mut()
        /*.iter_mut()
        .rev()
        .find(|e| matches!(e, OpenDomainElement::Module { .. }))
         */
        else {
            return Err(FtmlExtractionError::NotIn(
                FtmlKey::Module,
                "a module (or inside of a declaration)",
            ));
        };
    };
}

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
            #[cfg(feature = "rdf")]
            iri: document.to_iri(),
            document,
            title: None,
            ids: IdCounter::default(),
            counters: Vec::new(),
            styles: Vec::new(),
            notations: Vec::new(),
            buffer: DataBuffer::default(),
            top: Vec::new(),
            modules: Vec::new(),
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
            styles: DocumentStyles {
                counters: take(&mut self.counters).into_boxed_slice(),
                styles: take(&mut self.styles).into_boxed_slice(),
            },
        }
        .close();
        tracing::info!("Finished document {document:?}");
        let modules = take(&mut self.modules)
            .into_iter()
            .map(|m| {
                tracing::info!("Found module {m:?}");
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
            Split::Open { domain, narrative } => {
                if let Some(dom) = domain {
                    self.domain.push(dom);
                }
                if let Some(narr) = narrative {
                    self.narrative.push(narr);
                }
            }
            Split::Meta(m) => match m {
                MetaDatum::Style(s) => self.styles.push(s),
                MetaDatum::Counter(c) => self.counters.push(c),
                MetaDatum::InputRef { target, uri } => {
                    self.push_elem(DocumentElement::DocumentReference { uri, target });
                }
                MetaDatum::SetSectionLevel(lvl) => {
                    self.push_elem(DocumentElement::SetSectionLevel(lvl));
                }
                MetaDatum::UseModule(uri) => self.push_elem(DocumentElement::UseModule(uri)),
                MetaDatum::ImportModule(uri) => {
                    get_module!(parent,parent_uri <- self);
                    parent.push(Declaration::Import(uri.clone()));
                    self.push_elem(DocumentElement::ImportModule(uri));
                }
            },
            Split::None => (),
        }
        Ok(())
    }

    /// ### Errors
    #[allow(clippy::too_many_lines)]
    pub fn close(&mut self, elem: CloseFtmlElement, node: &N) -> super::Result<()> {
        tracing::debug!("Closing: {elem:?} in {:?}", self.domain);
        match elem {
            CloseFtmlElement::Module => match self.domain.pop() {
                Some(OpenDomainElement::Module {
                    uri,
                    meta,
                    signature,
                    children,
                }) => {
                    if uri.is_top() {
                        self.close_module(uri, meta, signature, children)?;
                    } else {
                        self.close_nested_module(uri, children)?;
                    }
                    let Some(OpenNarrativeElement::Module { uri, children }) = self.narrative.pop()
                    else {
                        return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Module));
                    };
                    self.push_elem(DocumentElement::Module {
                        range: node.range(),
                        module: uri,
                        children: children.into_boxed_slice(),
                    });
                    Ok(())
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Module)),
            },
            CloseFtmlElement::Comp => match self.domain.pop() {
                Some(OpenDomainElement::Comp) => Ok(()),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Comp)),
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
                    arguments,
                    uri,
                }) = self.domain.pop()
                {
                    self.close_oma(head, uri, arguments, node)
                } else {
                    tracing::debug!("Error: {:?}", self.domain);
                    Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term))
                }
            }
            CloseFtmlElement::OMBIND => {
                if let Some(OpenDomainElement::OMBIND {
                    head,
                    notation: _,
                    arguments,
                    uri,
                }) = self.domain.pop()
                {
                    self.close_ombind(head, uri, arguments, node)
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
            CloseFtmlElement::Type => match self.domain.pop() {
                Some(OpenDomainElement::Type { terms, node: n }) => {
                    //debug_assert_eq!(node,n);
                    self.close_type(terms, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type)),
            },
            CloseFtmlElement::Definiens => match self.domain.pop() {
                Some(OpenDomainElement::Definiens { terms, node: n }) => {
                    //debug_assert_eq!(node,n);
                    self.close_definiens(terms, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiens)),
            },
            CloseFtmlElement::SectionTitle => self.close_title(node),
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

    fn push_elem(&mut self, e: DocumentElement) {
        for d in self.narrative.iter_mut() {
            match d {
                OpenNarrativeElement::Module { children, .. }
                | OpenNarrativeElement::Section { children, .. }
                | OpenNarrativeElement::SkipSection { children } => {
                    children.push(e);
                    return;
                }
                OpenNarrativeElement::Invisible
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::NotationArg(_) => (),
            }
        }
        self.top.push(e);
    }

    fn close_section(
        &mut self,
        uri: DocumentElementUri,
        title: Option<DocumentRange>,
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
    fn close_notation(
        &mut self,
        uri: DocumentElementUri,
        id: Option<Id>,
        head: VarOrSym,
        prec: isize,
        argprecs: SmallVec<isize, 9>,
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
        tracing::info!("New notation for {head:?}: {not:?}");
        let notation = self
            .buffer
            .push(&not)
            .map_err(|e| FtmlExtractionError::EncodingError(FtmlKey::Notation, e.to_string()))?;

        let (e, leaf) = match head {
            VarOrSym::S(s) => (
                DocumentElement::Notation {
                    symbol: s.clone(),
                    uri: uri.clone(),
                    notation,
                },
                s.into(),
            ),
            VarOrSym::V(Variable::Ref { declaration, .. }) => (
                DocumentElement::VariableNotation {
                    variable: declaration.clone(),
                    uri: uri.clone(),
                    notation,
                },
                declaration.into(),
            ),
            VarOrSym::V(_) => return Err(FtmlExtractionError::InvalidValue(FtmlKey::Notation)),
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

    fn close_title(&mut self, node: &N) -> super::Result<()> {
        for e in self.narrative.iter_mut() {
            match e {
                OpenNarrativeElement::Section { title, .. } if title.is_none() => {
                    *title = Some(node.range());
                    return Ok(());
                }
                OpenNarrativeElement::Section { title, .. } => {
                    return Err(FtmlExtractionError::DuplicateValue(FtmlKey::Title));
                }
                OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::NotationArg(_) => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Title));
                }
                OpenNarrativeElement::Module { .. } | OpenNarrativeElement::Invisible => (),
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
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::Comp
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
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::Comp
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
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::NotationArg(_)
                | OpenNarrativeElement::Module { .. } => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type));
                }
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Type))
    }

    fn close_definiens(
        &mut self,
        terms: Vec<(Term, crate::NodePath)>,
        node: &N,
    ) -> super::Result<()> {
        let term = node.as_term(terms)?.simplify();
        match self.domain.last_mut() {
            Some(OpenDomainElement::SymbolDeclaration { uri, data }) if data.df.is_none() => {
                data.df = Some(term);
                return Ok(());
            }
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::Type { .. }
                | OpenDomainElement::Definiens { .. }
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. }
                | OpenDomainElement::Comp
                | OpenDomainElement::VariableReference { .. },
            ) => (),
        }

        for n in self.narrative.iter_mut() {
            match n {
                OpenNarrativeElement::VariableDeclaration { uri, data } if data.df.is_none() => {
                    data.df = Some(term);
                    return Ok(());
                }
                OpenNarrativeElement::Invisible => (),

                OpenNarrativeElement::Section { .. }
                | OpenNarrativeElement::VariableDeclaration { .. }
                | OpenNarrativeElement::SkipSection { .. }
                | OpenNarrativeElement::Notation { .. }
                | OpenNarrativeElement::NotationComp { .. }
                | OpenNarrativeElement::ArgSep { .. }
                | OpenNarrativeElement::NotationArg(_)
                | OpenNarrativeElement::Module { .. } => {
                    return Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiens));
                }
            }
        }
        Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Definiens))
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
                | OpenDomainElement::OMA { .. }
                | OpenDomainElement::OMBIND { .. }
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
                | OpenDomainElement::Definiens {
                    terms,
                    node: ancestor,
                },
            ) => {
                terms.push((term, node.path_from(ancestor)));
                return Ok(());
            }
            Some(OpenDomainElement::Comp) => {
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
        let term = Term::Application {
            head: Box::new(match head {
                VarOrSym::S(s) => Term::Symbol(s),
                VarOrSym::V(v) => Term::Var(v),
            }),
            arguments: args.into_boxed_slice(),
        }
        .simplify();
        self.close_app_term(uri, term, node)
    }

    fn close_ombind(
        &mut self,
        head: VarOrSym,
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
        let term = Term::Bound {
            head: Box::new(match head {
                VarOrSym::S(s) => Term::Symbol(s),
                VarOrSym::V(v) => Term::Var(v),
            }),
            body: Box::new(body),
            arguments: args.into_boxed_slice(),
        }
        .simplify();
        self.close_app_term(uri, term, node)
    }

    fn close_oms(&mut self, uri: SymbolUri, notation: Option<Id>, node: &N) -> super::Result<()> {
        self.close_term(Term::Symbol(uri), node, |slf, term| {
            let Term::Symbol(uri) = term else {
                // SAFETY: close_term returns the same term
                unsafe { unreachable_unchecked() }
            };
            slf.push_elem(DocumentElement::SymbolReference {
                range: node.range(),
                uri,
                notation,
            });
            Ok(())
        })
    }

    fn close_omv(&mut self, var: Variable, notation: Option<Id>, node: &N) -> super::Result<()> {
        self.close_term(Term::Var(var), node, |slf, term| {
            let Term::Var(var) = term else {
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
        })
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
        add_triples!(NARR self,module -> (self.iri.clone()) ulo:contains);
        self.modules.push(module);
        Ok(())
    }

    fn close_nested_module(
        &mut self,
        uri: ModuleUri,
        children: Vec<Declaration>,
    ) -> super::Result<()> {
        get_module!(parent,parent_uri <- self);
        let module = NestedModule {
            // SAFETY: uri is not is_top() verified above
            uri: unsafe { uri.into_symbol().unwrap_unchecked() },
            declarations: children.into_boxed_slice(),
        };
        add_triples!(DOM self,module -> parent_uri);
        parent.push(Declaration::NestedModule(module));
        Ok(())
    }

    fn close_symbol(&mut self, uri: SymbolUri, data: Box<SymbolData>) -> super::Result<()> {
        get_module!(parent,parent_uri <- self);
        let symbol = Symbol { uri, data };
        tracing::info!("New symbol {symbol:?}");
        add_triples!(DOM self,symbol -> parent_uri);
        parent.push(Declaration::Symbol(symbol));
        Ok(())
    }

    fn close_vardecl(&mut self, uri: DocumentElementUri, data: Box<VariableData>) {
        let var = VariableDeclaration { uri, data };
        tracing::info!("New variable {var:?}");
        self.push_elem(DocumentElement::VariableDeclaration(var));
        //add_triples!(DOM self,symbol -> parent_uri);
        //parent.push(Declaration::Symbol(symbol));
    }
}
