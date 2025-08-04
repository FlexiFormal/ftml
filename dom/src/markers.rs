use crate::{
    FtmlViews,
    clonable_views::MarkedNode,
    counters::LogicalLevel,
    document::DocumentState,
    extractor::DomExtractor,
    terms::{ReactiveApplication, ReactiveTerm},
    utils::{
        ContextChain,
        actions::{OneShot, SetOneShotDone},
        local_cache::LOCAL_CACHE,
    },
};
use ftml_core::extraction::{ArgumentPosition, FtmlExtractor, OpenFtmlElement, VarOrSym};
use ftml_ontology::{
    narrative::elements::{
        SectionLevel,
        paragraphs::{ParagraphFormatting, ParagraphKind},
    },
    terms::Variable,
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri, SymbolUri};
use leptos::{
    IntoView,
    prelude::{IntoAny, Memo, RwSignal, with_context},
};
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
    Comp,
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
    ) -> impl IntoView {
        #[allow(clippy::enum_glob_use)]
        use leptos::either::EitherOf15::*;
        let Some(m) = markers.pop() else {
            return A(leptos_posthoc::DomCont(leptos_posthoc::DomContProps {
                orig,
                cont: super::iterate::<Views>,
                skip_head: true,
                class: None::<String>.into(),
                style: None::<String>.into(),
            }));
        };
        if invisible {
            tracing::debug!("skipping invisibles");
        }
        match m {
            Self::Comp
            | Self::Argument(_)
            | Self::OMA { .. }
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
                if invisible =>
            {
                B(Self::apply::<Views>(markers, invisible, is_math, orig).into_any())
            }
            Self::Section(uri) => C(DocumentState::new_section(uri, move |info| {
                Views::section(info, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
                })
            })),
            Self::SkipSection => D(DocumentState::skip_section(move || {
                Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
            })),
            Self::SectionTitle => {
                let (LogicalLevel::Section(lvl), cls) = DocumentState::title_class() else {
                    tracing::error!("Unexpected section title");
                    return E(Self::apply::<Views>(markers, invisible, is_math, orig).into_any());
                };
                F(Views::section_title(lvl, cls, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
                }))
            }
            Self::InputRef { target, uri } => {
                H(DocumentState::do_inputref(target, uri, Views::inputref))
            }
            Self::Argument(pos) => {
                ContextChain::provide(Some(pos));
                if let Some(r) =
                    with_context::<Option<ReactiveTerm>, _>(|t| t.as_ref().map(|t| t.app)).flatten()
                {
                    let node = MarkedNode::new(markers, orig, is_math, false).into();
                    M(
                        //owned!(
                        ReactiveApplication::add_argument::<Views>(r, pos, node), //)
                    )
                } else {
                    B(Self::apply::<Views>(markers, invisible, is_math, orig).into_any())
                }
            }
            Self::CurrentSectionLevel(cap) => {
                let lvl = DocumentState::current_section_level();
                N(match (lvl, cap) {
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
                })
            }
            Self::Comp => G(Views::comp(
                MarkedNode::new(markers, orig, is_math, true).into(),
            )),
            Self::SymbolReference {
                uri,
                notation,
                in_term,
            } => I(Views::symbol_reference(
                uri,
                notation,
                in_term,
                MarkedNode::new(markers, orig, is_math, true).into(),
            )),
            Self::VariableReference {
                var,
                notation,
                in_term,
            } => J(Views::variable_reference(
                var,
                notation,
                in_term,
                MarkedNode::new(markers, orig, is_math, true).into(),
            )),
            Self::OMA {
                head,
                notation,
                uri,
            } => K(Views::application(
                head,
                notation,
                uri,
                MarkedNode::new(markers, orig, is_math, true).into(),
            )),
            Self::OMBIND {
                head,
                notation,
                uri,
            } => L(Views::binder_application(
                head,
                notation,
                uri,
                MarkedNode::new(markers, orig, is_math, true).into(),
            )),
            Self::Paragraph {
                uri,
                kind,
                formatting,
                styles,
                fors,
            } => {
                if *uri.document_uri() != *DocumentUri::no_doc() {
                    LOCAL_CACHE
                        .paragraphs
                        .insert(uri.clone(), orig.html_string());
                }
                O(DocumentState::new_paragraph(
                    uri,
                    kind,
                    formatting,
                    styles,
                    fors,
                    move |info| {
                        Views::paragraph(info, move || {
                            Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
                        })
                    },
                ))
            }
        }
    }

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
            OpenFtmlElement::Comp => Some(Self::Comp),
            OpenFtmlElement::SkipSection => Some(Self::SkipSection),
            OpenFtmlElement::SectionTitle => Some(Self::SectionTitle),
            OpenFtmlElement::Section(uri) => Some(Self::Section(uri.clone())),
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
            | OpenFtmlElement::Style(_)
            | OpenFtmlElement::NotationArg(_)
            | OpenFtmlElement::Type
            | OpenFtmlElement::Definiens
            | OpenFtmlElement::Notation { .. }
            | OpenFtmlElement::SymbolDeclaration { .. }
            | OpenFtmlElement::NotationComp
            | OpenFtmlElement::ArgSep
            | OpenFtmlElement::ImportModule(_)
            | OpenFtmlElement::UseModule(_)
            | OpenFtmlElement::VariableDeclaration { .. }
            | OpenFtmlElement::ParagraphTitle
            | OpenFtmlElement::None => None,
        }
    }
}
