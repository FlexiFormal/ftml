use std::borrow::Cow;

use crate::{
    FtmlKey,
    extraction::{
        ArgumentPosition, CloseFtmlElement, FtmlExtractionError, MetaDatum, OpenArgument,
        OpenDomainElement, OpenFtmlElement, OpenNarrativeElement, Split, VarOrSym, nodes::FtmlNode,
    },
};
use ftml_ontology::{
    domain::{
        declarations::{
            AnyDeclaration,
            symbols::{Symbol, SymbolData},
        },
        modules::{ModuleData, NestedModule},
    },
    narrative::{
        DocumentRange,
        documents::{DocumentCounter, DocumentStyle},
        elements::{DocumentElement, Section},
    },
    terms::{Argument, ArgumentMode, BoundArgument, Term, Variable},
};
#[cfg(feature = "rdf")]
use ftml_uris::FtmlUri;
use ftml_uris::{DocumentElementUri, DocumentUri, Id, Language, ModuleUri, SymbolUri, UriName};

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

struct StackVec<T> {
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
    fn iter(&self) -> impl Iterator<Item = &T> {
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

    fn push(&mut self, e: T) {
        if let Some(e) = self.last.replace(e) {
            self.rest.push(e);
        }
    }

    fn pop(&mut self) -> Option<T> {
        std::mem::replace(&mut self.last, self.rest.pop())
    }
}

pub struct ExtractorState<N: FtmlNode> {
    pub document: DocumentUri,
    pub top: Vec<DocumentElement>,
    pub modules: Vec<ModuleData>,
    pub counters: Vec<DocumentCounter>,
    pub styles: Vec<DocumentStyle>,
    domain: StackVec<OpenDomainElement<N>>,
    narrative: StackVec<OpenNarrativeElement>,
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

#[allow(unused_variables)]
impl<N: FtmlNode> ExtractorState<N> {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(document: DocumentUri, do_rdf: bool) -> Self {
        Self {
            do_rdf,
            #[cfg(feature = "rdf")]
            iri: document.to_iri(),
            document,
            ids: IdCounter::default(),
            counters: Vec::new(),
            styles: Vec::new(),
            top: Vec::new(),
            modules: Vec::new(),
            domain: StackVec::default(),
            narrative: StackVec::default(),
            #[cfg(feature = "rdf")]
            rdf: rustc_hash::FxHashSet::default(),
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
    pub fn narrative(&self) -> impl Iterator<Item = &OpenNarrativeElement> {
        self.narrative.iter()
    }

    /// ### Errors
    pub fn add(&mut self, e: OpenFtmlElement, node: &N) {
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
            },
            Split::None => (),
        }
    }

    /// ### Errors
    pub fn close(&mut self, elem: CloseFtmlElement, node: &N) -> super::Result<()> {
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
            CloseFtmlElement::SymbolDeclaration => match self.domain.pop() {
                Some(OpenDomainElement::SymbolDeclaration { uri, data }) => {
                    self.close_symbol(uri, data)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Symdecl)),
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
            CloseFtmlElement::SymbolReference => match self.domain.pop() {
                Some(OpenDomainElement::SymbolReference { uri, notation }) => {
                    self.close_oms(uri, notation, node)
                }
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term)),
            },
            CloseFtmlElement::OMA => match self.domain.pop() {
                Some(OpenDomainElement::OMA {
                    head,
                    notation: _,
                    arguments,
                    uri,
                }) => self.close_oma(head, uri, arguments, node),
                _ => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Symdecl)),
            },
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
            CloseFtmlElement::SectionTitle => self.close_title(node),
            CloseFtmlElement::Invisible => {
                node.delete();
                Ok(())
            }
        }
    }

    fn push_elem(&mut self, e: DocumentElement) {
        match &mut self.narrative.last {
            Some(
                OpenNarrativeElement::Module { children, .. }
                | OpenNarrativeElement::Section { children, .. }
                | OpenNarrativeElement::SkipSection { children },
            ) => {
                children.push(e);
            }
            Some(OpenNarrativeElement::Invisible) => (),
            None => self.top.push(e),
        }
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
                OpenNarrativeElement::SkipSection { .. } => {
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
        mut terms: Vec<(Term, crate::NodePath)>,
        node: &N,
    ) -> super::Result<()> {
        let term = if terms.len() == 1 {
            match terms.first() {
                Some((_, path)) if path.is_empty() =>
                // SAFETY: len == 1
                unsafe { terms.pop().unwrap_unchecked().0 },
                _ => node.as_term(terms)?,
            }
        } else {
            node.as_term(terms)?
        };
        match self.domain.last_mut() {
            Some(OpenDomainElement::OMA { arguments, .. }) => {
                OpenArgument::set(arguments, position, term)
            }
            None
            | Some(
                OpenDomainElement::Argument { .. }
                | OpenDomainElement::Module { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. },
            ) => Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Arg)),
        }
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

        match &mut self.domain.last {
            Some(OpenDomainElement::Module { .. }) | None => (),
            Some(OpenDomainElement::Argument {
                terms,
                node: ancestor,
                ..
            }) => {
                terms.push((term, node.path_from(ancestor)));
                return Ok(());
            }
            Some(
                OpenDomainElement::OMA { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. },
            ) => {
                return Err(FtmlExtractionError::InvalidIn(
                    FtmlKey::Term,
                    "declarations or terms outside of an argument",
                ));
            }
        }
        uri.map_or(
            Err(FtmlExtractionError::UnexpectedEndOf(FtmlKey::Term)),
            |uri| {
                self.push_elem(DocumentElement::Term { uri, term });
                Ok(())
            },
        )
    }

    fn close_oms(
        &mut self,
        uri: SymbolUri,
        notation: Option<UriName>,
        node: &N,
    ) -> super::Result<()> {
        match &mut self.domain.last {
            Some(OpenDomainElement::Argument {
                terms,
                node: ancestor,
                ..
            }) => {
                terms.push((Term::Symbol(uri), node.path_from(ancestor)));
                return Ok(());
            }
            Some(
                OpenDomainElement::OMA { .. }
                | OpenDomainElement::SymbolDeclaration { .. }
                | OpenDomainElement::SymbolReference { .. },
            ) => {
                return Err(FtmlExtractionError::InvalidIn(
                    FtmlKey::Term,
                    "declarations or terms outside of an argument",
                ));
            }
            Some(OpenDomainElement::Module { .. }) | None => (),
        }
        self.push_elem(DocumentElement::SymbolReference {
            range: node.range(),
            uri,
            notation,
        });
        Ok(())
    }

    fn close_module(
        &mut self,
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<AnyDeclaration>,
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
        children: Vec<AnyDeclaration>,
    ) -> super::Result<()> {
        get_module!(parent,parent_uri <- self);
        let module = NestedModule {
            // SAFETY: uri is not is_top() verified above
            uri: unsafe { uri.into_symbol().unwrap_unchecked() },
            declarations: children.into_boxed_slice(),
        };
        add_triples!(DOM self,module -> parent_uri);
        parent.push(AnyDeclaration::NestedModule(module));
        Ok(())
    }

    fn close_symbol(&mut self, uri: SymbolUri, data: Box<SymbolData>) -> super::Result<()> {
        get_module!(parent,parent_uri <- self);
        let symbol = Symbol { uri, data };
        add_triples!(DOM self,symbol -> parent_uri);
        parent.push(AnyDeclaration::Symbol(symbol));
        Ok(())
    }
}
