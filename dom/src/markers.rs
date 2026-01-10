use crate::{
    FtmlViews,
    clonable_views::MarkedNode,
    counters::LogicalLevel,
    document::{CurrentUri, DocumentState, WithHead},
    extractor::{DomExtractor, FtmlDomElement},
    structure::{DocumentStructure, TocSource},
    terms::ReactiveTerm,
    utils::{ContextChain, local_cache::LOCAL_CACHE},
};
use ftml_ontology::{
    narrative::elements::{
        SectionLevel,
        paragraphs::{ParagraphFormatting, ParagraphKind},
        problems::ChoiceBlockStyle,
    },
    terms::{VarOrSym, Variable},
};
use ftml_parser::extraction::{ArgumentPosition, FtmlExtractor, OpenFtmlElement, nodes::FtmlNode};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri, NarrativeUri, SymbolUri};
use leptos::prelude::{
    AnyView, CustomAttribute, Get, IntoAny, Memo, RwSignal, Update, expect_context,
    provide_context, use_context, with_context,
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
    SlideNumber,
    ParagraphTitle,
    SlideTitle,
    ProblemTitle,
    Comp,
    DefComp(Option<SymbolUri>),
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
    Problem {
        is_subproblem: bool,
        styles: Box<[Id]>,
        uri: DocumentElementUri,
        autogradable: bool,
        points: Option<ordered_float::OrderedFloat<f32>>,
        minutes: Option<ordered_float::OrderedFloat<f32>>,
    },
    Solution,
    Fillinsol(Option<ordered_float::OrderedFloat<f32>>),
    Slide(DocumentElementUri),
    IfInputref(bool),
    ProblemHint,
    ProblemExNote,
    ProblemGNote,
    SingleChoiceBlock(ChoiceBlockStyle),
    MultipleChoiceBlock(ChoiceBlockStyle),
    Choice,
    ProofBody,
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
            | Self::DefComp(_)
            | Self::Argument(_)
            | Self::OMA { .. }
            | Self::SymbolReference { .. }
            | Self::SlideNumber
            | Self::VariableReference { .. }
                if invisible =>
            {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            }
            Self::IfInputref(b) if DocumentState::in_inputref() == b => {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            }
            Self::IfInputref(_) => orig.attr("style", "display:none;").into_any(),
            Self::Section(uri) => {
                let info = DocumentState::new_section(uri);
                Views::section(info, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig)
                })
                .into_any()
            }
            Self::SkipSection => {
                DocumentState::skip_section();
                Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
            }
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
                        .insert(uri.clone(), orig.html_string().into_boxed_str());
                }
                DocumentState::new_paragraph(uri, kind, formatting, styles, fors, move |info| {
                    Views::paragraph(info, move || {
                        Self::apply::<Views>(markers, invisible, is_math, orig)
                    })
                })
                .into_any()
            }
            Self::ProofBody => Views::proof_body(orig).into_any(),
            Self::Problem {
                is_subproblem,
                styles,
                uri,
                autogradable,
                points,
                minutes,
            } => {
                let (style, class) = DocumentState::new_problem(uri.clone(), &styles);
                Views::problem(
                    uri,
                    styles,
                    style,
                    class,
                    is_subproblem,
                    autogradable,
                    points.map(|f| *f),
                    minutes.map(|f| *f),
                    move || Self::apply::<Views>(markers, invisible, is_math, orig),
                )
                .into_any()
            }
            Self::Solution => {
                // parse node content:
                let _ = Self::apply::<Views>(markers, invisible, is_math, orig.clone());
                orig.set_inner_html("");
                Views::problem_solution().into_any()
            }
            Self::Fillinsol(wd) => {
                // parse node content:
                let _ = Self::apply::<Views>(markers, invisible, is_math, orig.clone());
                orig.set_inner_html("");
                Views::fillinsol(wd.map(|f| *f)).into_any()
            }
            Self::ProblemHint => {
                // parse node content:
                Views::problem_hint(move || Self::apply::<Views>(markers, invisible, is_math, orig))
                    .into_any()
            }
            Self::ProblemExNote => {
                // parse node content:
                Views::problem_ex_note(move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig)
                })
                .into_any()
            }
            Self::ProblemGNote => {
                // parse node content:
                let _ = Self::apply::<Views>(markers, invisible, is_math, orig.clone());
                orig.set_inner_html("");
                Views::problem_gnote().into_any()
            }
            Self::MultipleChoiceBlock(style) => Views::multiple_choice_block(style, move || {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            })
            .into_any(),
            Self::SingleChoiceBlock(style) => Views::single_choice_block(style, move || {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            })
            .into_any(),
            Self::Choice => Views::problem_choice(move || {
                Self::apply::<Views>(markers, invisible, is_math, orig)
            })
            .into_any(),
            Self::Slide(uri) => {
                DocumentState::new_slide(uri.clone());
                Views::slide(uri, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig)
                })
                .into_any()
            }
            Self::SectionTitle => {
                /*
                let (LogicalLevel::Section(lvl), cls) = DocumentState::title_class() else {
                    tracing::error!("Unexpected section title");
                    return Self::apply::<Views>(markers, invisible, is_math, orig);
                };

                if with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract))
                    .is_some_and(|b| b)
                    && let NarrativeUri::Element(uri) = expect_context::<CurrentUri>().0
                {
                    let current_toc = expect_context::<RwSignal<CurrentTOC>>();
                    let node = FtmlDomElement::new((*orig).clone());
                    let title = node.inner_string().into_owned();
                    if !title.is_empty() {
                        current_toc.update(|toc| toc.set_title(&uri, title.into_boxed_str()));
                    }
                }
                 */
                let cls = DocumentStructure::insert_section_title(|| {
                    FtmlDomElement::new((*orig).clone())
                        .inner_string()
                        .into_owned()
                });
                Views::section_title(cls, orig).into_any()
            }
            Self::ParagraphTitle => Views::paragraph_title(orig).into_any(),
            Self::SlideTitle => Views::slide_title(orig).into_any(),
            Self::ProblemTitle => Views::problem_title(orig).into_any(),
            Self::InputRef { target, uri } => {
                let ipr = DocumentState::do_inputref(target, uri);
                Views::inputref(ipr).into_any()
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
                lvl.into_view(cap).into_any()
            }
            Self::SlideNumber => DocumentStructure::get_slide().into_any(),
            Self::Comp => {
                Views::comp(MarkedNode::new(markers, orig, is_math, true).into()).into_any()
            }
            Self::DefComp(u) => {
                Views::def_comp(u, MarkedNode::new(markers, orig, is_math, true).into()).into_any()
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
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn from(ext: &DomExtractor, elem: &OpenFtmlElement) -> Option<Self> {
        match elem {
            OpenFtmlElement::SetSectionLevel(lvl) => {
                DocumentStructure::set_max_level(*lvl);
                None
            }
            OpenFtmlElement::InputRef { target, uri } => Some(Self::InputRef {
                target: target.clone(),
                uri: uri.clone(),
            }),
            OpenFtmlElement::Problem {
                is_subproblem,
                styles,
                uri,
                autogradable,
                points,
                minutes,
            } => Some(Self::Problem {
                is_subproblem: *is_subproblem,
                styles: styles.clone(),
                uri: uri.clone(),
                autogradable: *autogradable,
                points: points.map(Into::into),
                minutes: minutes.map(Into::into),
            }),
            OpenFtmlElement::FillinSol(wd) => Some(Self::Fillinsol(wd.map(Into::into))),
            OpenFtmlElement::IfInputref(b) => Some(Self::IfInputref(*b)),
            OpenFtmlElement::Comp => Some(Self::Comp),
            OpenFtmlElement::DefComp => Some(Self::DefComp(None)),
            OpenFtmlElement::Definiendum(u) => Some(Self::DefComp(Some(u.clone()))),
            OpenFtmlElement::SkipSection => Some(Self::SkipSection),
            OpenFtmlElement::SectionTitle => Some(Self::SectionTitle),
            OpenFtmlElement::ParagraphTitle => Some(Self::ParagraphTitle),
            OpenFtmlElement::SlideTitle => Some(Self::SlideTitle),
            OpenFtmlElement::ProblemTitle => Some(Self::ProblemTitle),
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
            OpenFtmlElement::Slide(uri) => Some(Self::Slide(uri.clone())),
            OpenFtmlElement::Argument(pos) => Some(Self::Argument(*pos)),
            OpenFtmlElement::CurrentSectionLevel(b) => Some(Self::CurrentSectionLevel(*b)),
            OpenFtmlElement::SlideNumber => Some(Self::SlideNumber),
            OpenFtmlElement::Solution(_) => Some(Self::Solution),
            OpenFtmlElement::ProblemHint => Some(Self::ProblemHint),
            OpenFtmlElement::ProblemExNote => Some(Self::ProblemExNote),
            OpenFtmlElement::ProblemGradingNote => Some(Self::ProblemGNote),
            OpenFtmlElement::ChoiceBlock {
                block_style,
                multiple: true,
                ..
            } => Some(Self::MultipleChoiceBlock(*block_style)),
            OpenFtmlElement::ChoiceBlock { block_style, .. } => {
                Some(Self::SingleChoiceBlock(*block_style))
            }
            OpenFtmlElement::ProblemChoice(_) => Some(Self::Choice),
            OpenFtmlElement::ProofBody => Some(Self::ProofBody),
            OpenFtmlElement::Counter(_)
            | OpenFtmlElement::Invisible
            | OpenFtmlElement::Module { .. }
            | OpenFtmlElement::MathStructure { .. }
            | OpenFtmlElement::Morphism { .. }
            | OpenFtmlElement::Style(_)
            | OpenFtmlElement::NotationArg(_)
            | OpenFtmlElement::Type
            | OpenFtmlElement::Precondition { .. }
            | OpenFtmlElement::Objective { .. }
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
            | OpenFtmlElement::Assign(_)
            | OpenFtmlElement::DocumentKind(_)
            | OpenFtmlElement::OML { .. }
            | OpenFtmlElement::Rename { .. }
            | OpenFtmlElement::FillinSolCase(_)
            | OpenFtmlElement::AnswerClass(..)
            | OpenFtmlElement::AnswerClassFeedback
            | OpenFtmlElement::ProblemChoiceVerdict
            | OpenFtmlElement::ProblemChoiceFeedback
            | OpenFtmlElement::ArgTypes
            | OpenFtmlElement::HeadTerm
            | OpenFtmlElement::Rule(_) => None,
        }
    }
}
