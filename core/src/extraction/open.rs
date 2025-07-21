use ftml_ontology::{
    domain::declarations::{AnyDeclaration, symbols::SymbolData},
    narrative::{
        DocumentRange,
        documents::{DocumentCounter, DocumentStyle},
        elements::{DocumentElement, sections::SectionLevel},
    },
};
use ftml_uris::{DocumentElementUri, Language, ModuleUri, SymbolUri};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenFtmlElement {
    None,
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
    },
    Symbol {
        uri: SymbolUri,
        data: Box<SymbolData>,
    },
    Section(DocumentElementUri),
    SetSectionLevel(SectionLevel),
    Style(DocumentStyle),
    Counter(DocumentCounter),
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseFtmlElement {
    Module,
    Symbol,
    Invisible,
    Section,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenDomainElement {
    Module {
        uri: ModuleUri,
        meta: Option<ModuleUri>,
        signature: Option<Language>,
        children: Vec<AnyDeclaration>,
    },
    Symbol {
        uri: SymbolUri,
        data: Box<SymbolData>,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaDatum {
    Style(DocumentStyle),
    Counter(DocumentCounter),
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
            Self::Symbol { uri, data } => Split::Open {
                domain: Some(OpenDomainElement::Symbol { uri, data }),
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
            Self::Style(s) => Split::Meta(MetaDatum::Style(s)),
            Self::Counter(c) => Split::Meta(MetaDatum::Counter(c)),
            Self::SetSectionLevel(_) | Self::None | Self::Invisible => Split::None,
        }
    }
}
