use crate::{
    FtmlViews,
    counters::LogicalLevel,
    document::DocumentState,
    extractor::DomExtractor,
    terms::{ReactiveApplication, ReactiveTerm},
    utils::actions::{OneShot, SetOneShotDone},
};
use ftml_core::extraction::{ArgumentPosition, FtmlExtractor, OpenFtmlElement, VarOrSym};
use ftml_ontology::terms::Variable;
use ftml_uris::{DocumentElementUri, DocumentUri, SymbolUri, UriName};
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
        notation: Option<UriName>,
    },
    VariableReference {
        in_term: bool,
        var: Variable,
        notation: Option<UriName>,
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
        notation: Option<UriName>,
    },
    OMBIND {
        uri: Option<DocumentElementUri>,
        head: VarOrSym,
        notation: Option<UriName>,
    },
    Argument(ArgumentPosition),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SectionInfo {
    pub uri: DocumentElementUri,
    pub style: Option<Memo<String>>,
    pub class: Option<&'static str>,
    pub lvl: LogicalLevel,
    pub id: String,
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

macro_rules! owned {
    ($e:expr) => {
        $e
    };
    (!$e:expr) => {{
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let children = owner.with(move || $e);
        leptos::tachys::reactive_graph::OwnedView::new_with_owner(children, owner)
    }};
}

impl Marker {
    #[allow(clippy::too_many_lines)]
    pub fn apply<Views: FtmlViews + ?Sized>(
        mut markers: MarkerList,
        invisible: bool,
        is_math: bool,
        mut orig: OriginalNode,
    ) -> impl IntoView {
        #[allow(clippy::enum_glob_use)]
        use leptos::either::EitherOf13::*;
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
            Self::Section(uri) => C(owned!(DocumentState::new_section(uri, move |info| {
                Views::section(info, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
                })
            }))),
            Self::SkipSection => D(owned!(DocumentState::skip_section(move || {
                Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
            }))),
            Self::SectionTitle => {
                let (LogicalLevel::Section(lvl), cls) = DocumentState::title_class() else {
                    tracing::error!("Unexpected section title");
                    return E(Self::apply::<Views>(markers, invisible, is_math, orig).into_any());
                };
                F(owned!(Views::section_title(lvl, cls, move || {
                    Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
                })))
            }
            Self::Comp => G(owned!(Views::comp(move || {
                Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
            }))),
            Self::InputRef { target, uri } => H(owned!(DocumentState::do_inputref(
                target,
                uri,
                Views::inputref
            ))),
            Self::SymbolReference {
                uri,
                notation,
                in_term,
            } => I(owned!({
                //provide_context(ReactiveTerm::Symbol(uri.clone()));
                Views::symbol_reference(uri, notation, is_math, in_term, move || {
                    // makes sure the "current orig" gets actually used / hydrated first
                    // just in case it has, like, listeners or something
                    let clone = orig.deep_clone();
                    Self::apply::<Views>(
                        markers,
                        invisible,
                        is_math,
                        std::mem::replace(&mut orig, clone),
                    )
                    .into_any()
                })
            })),
            Self::VariableReference {
                var,
                notation,
                in_term,
            } => J(owned!({
                //provide_context(ReactiveTerm::Symbol(uri.clone()));
                Views::variable_reference(var, notation, is_math, in_term, move || {
                    // makes sure the "current orig" gets actually used / hydrated first
                    // just in case it has, like, listeners or something
                    let clone = orig.deep_clone();
                    Self::apply::<Views>(
                        markers,
                        invisible,
                        is_math,
                        std::mem::replace(&mut orig, clone),
                    )
                    .into_any()
                })
            })),
            Self::OMA { head, notation, .. } => K(owned!(Views::application(
                head,
                notation,
                is_math,
                move || {
                    // makes sure the "current orig" gets actually used / hydrated first
                    // just in case it has, like, listeners or something
                    let clone = orig.deep_clone();
                    Self::apply::<Views>(
                        markers,
                        invisible,
                        is_math,
                        std::mem::replace(&mut orig, clone),
                    )
                    .into_any()
                }
            ))),
            Self::OMBIND { head, notation, .. } => L(owned!(Views::binder_application(
                head,
                notation,
                is_math,
                move || {
                    // makes sure the "current orig" gets actually used / hydrated first
                    // just in case it has, like, listeners or something
                    let clone = orig.deep_clone();
                    Self::apply::<Views>(
                        markers,
                        invisible,
                        is_math,
                        std::mem::replace(&mut orig, clone),
                    )
                    .into_any()
                }
            ))),
            Self::Argument(pos) => {
                if let Some(r) = with_context::<ReactiveTerm, _>(|t| match t {
                    ReactiveTerm::Application(s) => *s,
                }) {
                    M(
                        //owned!(
                        ReactiveApplication::add_argument(r, pos, move || {
                            Self::apply::<Views>(markers, invisible, is_math, orig).into_any()
                        }), //)
                    )
                } else {
                    B(Self::apply::<Views>(markers, invisible, is_math, orig).into_any())
                }
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
            OpenFtmlElement::Argument(pos) => Some(Self::Argument(*pos)),
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
            | OpenFtmlElement::None => None,
        }
    }
}
