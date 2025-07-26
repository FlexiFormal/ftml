use ftml_ontology::{
    domain::declarations::{AnyDeclaration, symbols::SymbolData},
    narrative::{
        DocumentRange,
        documents::{DocumentCounter, DocumentStyle},
        elements::{DocumentElement, sections::SectionLevel},
    },
};
use ftml_uris::{DocumentElementUri, DocumentUri, Language, ModuleUri, SymbolUri, UriName};

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
        in_term: bool,
    },
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenDomainElement {
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
        in_term: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::large_enum_variant)]
pub enum Split {
    Meta(MetaDatum),
    Open {
        domain: Option<OpenDomainElement>,
        narrative: Option<OpenNarrativeElement>,
    },
    None,
}

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

impl OpenFtmlElement {
    #[must_use]
    pub(crate) fn split(self) -> Split {
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
            Self::SymbolReference {
                uri,
                notation,
                in_term,
            } => Split::Open {
                domain: Some(OpenDomainElement::SymbolReference {
                    uri,
                    notation,
                    in_term,
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
            Self::SetSectionLevel(_)
            | Self::None
            | Self::Invisible
            | Self::SectionTitle
            | Self::Comp => Split::None,
        }
    }
}
