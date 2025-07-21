use std::borrow::Cow;

use crate::{
    FtmlKey,
    extraction::{
        CloseFtmlElement, FtmlExtractionError, MetaDatum, OpenDomainElement, OpenFtmlElement,
        OpenNarrativeElement, Split, nodes::FtmlNode,
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
};
#[cfg(feature = "rdf")]
use ftml_uris::FtmlUri;
use ftml_uris::{DocumentElementUri, DocumentUri, Id, Language, ModuleUri, SymbolUri};

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

pub struct ExtractorState {
    document: DocumentUri,
    pub top: Vec<DocumentElement>,
    domain: Vec<OpenDomainElement>,
    narrative: Vec<OpenNarrativeElement>,
    ids: IdCounter,
    pub counters: Vec<DocumentCounter>,
    pub styles: Vec<DocumentStyle>,
    #[allow(dead_code)]
    do_rdf: bool,
    pub modules: Vec<ModuleData>,
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
        }) = $self
            .domain
            .iter_mut()
            .rev()
            .find(|e| matches!(e, OpenDomainElement::Module { .. }))
        else {
            return Err(FtmlExtractionError::NotInModule(FtmlKey::Module));
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
impl ExtractorState {
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
            domain: Vec::new(),
            narrative: Vec::new(),
            #[cfg(feature = "rdf")]
            rdf: rustc_hash::FxHashSet::default(),
        }
    }

    #[inline]
    /// ### Errors
    pub fn new_id(&mut self, prefix: impl Into<Cow<'static, str>>) -> super::Result<Id> {
        Ok(self.ids.new_id(prefix).parse()?)
    }

    #[inline]
    #[must_use]
    pub const fn in_document(&self) -> &DocumentUri {
        &self.document
    }
    #[inline]
    #[must_use]
    pub fn domain(&self) -> &[OpenDomainElement] {
        &self.domain
    }
    #[inline]
    #[must_use]
    pub fn narrative(&self) -> &[OpenNarrativeElement] {
        &self.narrative
    }

    /// ### Errors
    pub fn add(&mut self, e: OpenFtmlElement) {
        match e.split() {
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
            },
            Split::None => (),
        }
    }

    /// ### Errors
    pub fn close<N: FtmlNode>(&mut self, elem: CloseFtmlElement, node: &N) -> super::Result<()> {
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
            CloseFtmlElement::Symbol => match self.domain.pop() {
                Some(OpenDomainElement::Symbol { uri, data }) => self.close_symbol(uri, data),
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
            CloseFtmlElement::Invisible => {
                node.delete();
                Ok(())
            }
        }
    }

    fn push_elem(&mut self, e: DocumentElement) {
        #[allow(clippy::never_loop)]
        for p in self.narrative.iter_mut().rev() {
            match p {
                OpenNarrativeElement::Module { children, .. }
                | OpenNarrativeElement::Section { children, .. } => {
                    children.push(e);
                    return;
                }
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
