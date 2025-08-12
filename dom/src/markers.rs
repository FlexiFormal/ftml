use crate::{
    FtmlViews,
    clonable_views::MarkedNode,
    counters::LogicalLevel,
    document::{CurrentUri, DocumentState, WithHead},
    extractor::DomExtractor,
    terms::ReactiveTerm,
    utils::{
        ContextChain,
        actions::{OneShot, SetOneShotDone},
        local_cache::LOCAL_CACHE,
    },
};
use ftml_core::extraction::{ArgumentPosition, FtmlExtractor, OpenFtmlElement};
use ftml_ontology::{
    narrative::elements::{
        SectionLevel,
        paragraphs::{ParagraphFormatting, ParagraphKind},
    },
    terms::{VarOrSym, Variable},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri, SymbolUri};
use leptos::prelude::{AnyView, IntoAny, Memo, RwSignal, provide_context, use_context};
use leptos_posthoc::OriginalNode;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Marker {
    Section(DocumentElementUri),
    SymbolReference {
        in_term: bool,
        uri: SymbolUri,
        notation: Option<Id>,
    },
    VariableReference {
        in_term: bool,
        var: Variable,
        notation: Option<Id>,
    },
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
    SkipSection,
    SectionTitle,
    ParagraphTitle,
    Comp,
    DefComp,
    OMA {
        uri: Option<DocumentElementUri>,
        head: VarOrSym,
        notation: Option<Id>,
    },
    OMBIND {
        uri: Option<DocumentElementUri>,
        head: VarOrSym,
        notation: Option<Id>,
    },
    Argument(ArgumentPosition),
    CurrentSectionLevel(bool),
    Paragraph {
        uri: DocumentElementUri,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Id]>,
        fors: Vec<SymbolUri>,
    },
    IfInputref(bool),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SectionInfo {
    pub uri: DocumentElementUri,
    pub style: Option<Memo<String>>,
    pub class: Option<&'static str>,
    pub lvl: LogicalLevel,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParagraphInfo {
    pub uri: DocumentElementUri,
    pub style: Memo<String>,
    pub class: Option<String>,
    pub kind: ParagraphKind,
    pub formatting: ParagraphFormatting,
    pub styles: Box<[Id]>,
    pub fors: Vec<SymbolUri>,
}

#[derive(Clone, Debug)]
pub struct InputrefInfo {
    pub uri: DocumentElementUri,
    pub target: DocumentUri,
    pub replace: OneShot,
    pub replacing_done: SetOneShotDone,
    pub id: String,
    pub title: RwSignal<String>,
}

pub type MarkerList = smallvec::SmallVec<Marker, 4>;

impl Marker {
    #[allow(clippy::too_many_lines)]
    pub fn apply<Views: FtmlViews + ?Sized>(
        mut markers: MarkerList,
        invisible: bool,
        is_math: bool,
        orig: OriginalNode,
    ) -> AnyView {
        let Some(m) = markers.pop() else {
            return leptos_posthoc::DomCont(leptos_posthoc::DomContProps {
                orig,
                cont: super::iterate::<Views>,
                skip_head: true,
                class: None::<String>.into(),
                style: None::<String>.into(),
            })
            .into_any();
        };
        if invisible {
            tracing::debug!("skipping invisibles");
        }
        match m {
            Self::Comp
            | Self::DefComp
            | Self::Argument(_)
            | Self::OMA { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
                if invisible =>
            {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            }
            Self::IfInputref(b) if DocumentState::in_inputref() == b => {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            }
            Self::IfInputref(_) => ().into_any(),
            Self::Section(uri) => {
                provide_context(CurrentUri(uri.clone().into()));
                DocumentState::new_section(uri, move |info| {
                    Views::section(info, move || {
                        Self::apply::<Views>(markers, invisible, is_math, orig)
                    })
                })
                .into_any()
            }
            Self::SkipSection => DocumentState::skip_section(move || {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            })
            .into_any(),
            Self::SectionTitle => {
                let (LogicalLevel::Section(lvl), cls) = DocumentState::title_class() else {
                    tracing::error!("Unexpected section title");
                    return Self::apply::<Views>(markers, invisible, is_math, orig);
                };
                Views::section_title(lvl, cls, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig)
                })
                .into_any()
            }
            Self::ParagraphTitle => Views::paragraph_title(move || {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            })
            .into_any(),
            Self::InputRef { target, uri } => {
                DocumentState::do_inputref(target, uri, Views::inputref).into_any()
            }
            Self::Argument(pos) => {
                ContextChain::provide(Some(pos));
                provide_context(WithHead(None));
                if let Some(r) = use_context::<Option<ReactiveTerm>>().flatten() {
                    let node = MarkedNode::new(markers, orig, is_math, false).into();
                    r.add_argument::<Views>(pos, node).into_any()
                } else {
                    Self::apply::<Views>(markers, invisible, is_math, orig)
                }
            }
            Self::CurrentSectionLevel(cap) => {
                let lvl = DocumentState::current_section_level();
                match (lvl, cap) {
                    (LogicalLevel::None, true) => "Document",
                    (LogicalLevel::None, _) => "document",
                    (LogicalLevel::Section(SectionLevel::Part), true) => "Part",
                    (LogicalLevel::Section(SectionLevel::Part), _) => "part",
                    (LogicalLevel::Section(SectionLevel::Chapter), true) => "Chapter",
                    (LogicalLevel::Section(SectionLevel::Chapter), _) => "chapter",
                    (LogicalLevel::Section(SectionLevel::Section), true) => "Section",
                    (LogicalLevel::Section(SectionLevel::Section), _) => "section",
                    (LogicalLevel::Section(SectionLevel::Subsection), true) => "Subsection",
                    (LogicalLevel::Section(SectionLevel::Subsection), _) => "subsection",
                    (LogicalLevel::Section(SectionLevel::Subsubsection), true) => "Subsubsection",
                    (LogicalLevel::Section(SectionLevel::Subsubsection), _) => "subsubsection",
                    (LogicalLevel::BeamerSlide, true) => "Slide",
                    (LogicalLevel::BeamerSlide, _) => "slide",
                    (_, true) => "Paragraph",
                    (_, _) => "paragraph",
                }
                .into_any()
            }
            Self::Comp => {
                Views::comp(false, MarkedNode::new(markers, orig, is_math, true).into()).into_any()
            }
            Self::DefComp => {
                Views::comp(true, MarkedNode::new(markers, orig, is_math, true).into()).into_any()
            }
            Self::SymbolReference {
                uri,
                notation,
                in_term,
            } => {
                provide_context(WithHead(Some(VarOrSym::Sym(uri.clone()))));
                Views::symbol_reference(
                    uri,
                    notation,
                    in_term,
                    MarkedNode::new(markers, orig, is_math, true).into(),
                )
                .into_any()
            }
            Self::VariableReference {
                var,
                notation,
                in_term,
            } => {
                provide_context(WithHead(Some(VarOrSym::Var(var.clone()))));
                Views::variable_reference(
                    var,
                    notation,
                    in_term,
                    MarkedNode::new(markers, orig, is_math, true).into(),
                )
                .into_any()
            }
            Self::OMA {
                head,
                notation,
                uri,
            } => {
                provide_context(WithHead(Some(head.clone())));
                if let Some(uri) = &uri {
                    provide_context(CurrentUri(uri.clone().into()));
                }
                Views::application(
                    head,
                    notation,
                    uri,
                    MarkedNode::new(markers, orig, is_math, true).into(),
                )
                .into_any()
            }
            Self::OMBIND {
                head,
                notation,
                uri,
            } => {
                provide_context(WithHead(Some(head.clone())));
                if let Some(uri) = &uri {
                    provide_context(CurrentUri(uri.clone().into()));
                }
                Views::binder_application(
                    head,
                    notation,
                    uri,
                    MarkedNode::new(markers, orig, is_math, true).into(),
                )
                .into_any()
            }
            Self::Paragraph {
                uri,
                kind,
                formatting,
                styles,
                fors,
            } => {
                provide_context(CurrentUri(uri.clone().into()));
                if *uri.document_uri() != *DocumentUri::no_doc() {
                    LOCAL_CACHE
                        .paragraphs
                        .insert(uri.clone(), orig.html_string());
                }
                DocumentState::new_paragraph(uri, kind, formatting, styles, fors, move |info| {
                    Views::paragraph(info, move || {
                        Self::apply::<Views>(markers, invisible, is_math, orig)
                    })
                })
                .into_any()
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn from(ext: &DomExtractor, elem: &OpenFtmlElement) -> Option<Self> {
        match elem {
            OpenFtmlElement::SetSectionLevel(lvl) => {
                let in_inputref = DocumentState::in_inputref();
                if !in_inputref {
                    DocumentState::update_counters(|c| {
                        if c.current_level() == LogicalLevel::None {
                            tracing::trace!("SetSectionLevel {lvl}");
                            c.max = *lvl;
                        } else {
                            tracing::error!("ftml:set-section-level: Section already started");
                        }
                    });
                }
                None
            }
            OpenFtmlElement::InputRef { target, uri } => Some(Self::InputRef {
                target: target.clone(),
                uri: uri.clone(),
            }),
            OpenFtmlElement::IfInputref(b) => Some(Self::IfInputref(*b)),
            OpenFtmlElement::Comp => Some(Self::Comp),
            OpenFtmlElement::DefComp | OpenFtmlElement::Definiendum(_) => Some(Self::DefComp),
            OpenFtmlElement::SkipSection => Some(Self::SkipSection),
            OpenFtmlElement::SectionTitle => Some(Self::SectionTitle),
            OpenFtmlElement::ParagraphTitle => Some(Self::ParagraphTitle),
            OpenFtmlElement::Section(uri) => Some(Self::Section(uri.clone())),
            OpenFtmlElement::ComplexTerm { head, notation, .. } => match head {
                VarOrSym::Sym(s) => Some(Self::SymbolReference {
                    uri: s.clone(),
                    notation: notation.clone(),
                    in_term: ext.in_term(),
                }),
                VarOrSym::Var(v) => Some(Self::VariableReference {
                    var: v.clone(),
                    notation: notation.clone(),
                    in_term: ext.in_term(),
                }),
            },
            OpenFtmlElement::SymbolReference { uri, notation } => Some(Self::SymbolReference {
                uri: uri.clone(),
                notation: notation.clone(),
                in_term: ext.in_term(),
            }),
            OpenFtmlElement::VariableReference { var, notation } => Some(Self::VariableReference {
                var: var.clone(),
                notation: notation.clone(),
                in_term: ext.in_term(),
            }),
            OpenFtmlElement::OMA {
                head,
                notation,
                uri,
            } => Some(Self::OMA {
                head: head.clone(),
                notation: notation.clone(),
                uri: uri.clone(),
            }),
            OpenFtmlElement::OMBIND {
                head,
                notation,
                uri,
            } => Some(Self::OMBIND {
                head: head.clone(),
                notation: notation.clone(),
                uri: uri.clone(),
            }),
            OpenFtmlElement::Paragraph {
                uri,
                kind,
                formatting,
                styles,
                fors,
            } => Some(Self::Paragraph {
                uri: uri.clone(),
                kind: *kind,
                formatting: *formatting,
                styles: styles.clone(),
                fors: fors.iter().map(|(u, _)| u.clone()).collect(),
            }),
            OpenFtmlElement::Argument(pos) => Some(Self::Argument(*pos)),
            OpenFtmlElement::CurrentSectionLevel(b) => Some(Self::CurrentSectionLevel(*b)),
            OpenFtmlElement::Counter(_)
            | OpenFtmlElement::Invisible
            | OpenFtmlElement::Module { .. }
            | OpenFtmlElement::MathStructure { .. }
            | OpenFtmlElement::Style(_)
            | OpenFtmlElement::NotationArg(_)
            | OpenFtmlElement::Type
            | OpenFtmlElement::ReturnType
            | OpenFtmlElement::Definiens(_)
            | OpenFtmlElement::Notation { .. }
            | OpenFtmlElement::SymbolDeclaration { .. }
            | OpenFtmlElement::NotationComp
            | OpenFtmlElement::ArgSep
            | OpenFtmlElement::ImportModule(_)
            | OpenFtmlElement::UseModule(_)
            | OpenFtmlElement::VariableDeclaration { .. }
            | OpenFtmlElement::None
            | OpenFtmlElement::OML { .. }
            | OpenFtmlElement::HeadTerm => None,
        }
    }
}
