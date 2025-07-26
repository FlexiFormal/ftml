use ftml_core::extraction::OpenFtmlElement;
use ftml_uris::{DocumentElementUri, DocumentUri, SymbolUri, UriName};
use leptos::{
    IntoView,
    prelude::{IntoAny, Memo, RwSignal},
};
use leptos_posthoc::OriginalNode;

use crate::{
    FtmlViews,
    counters::LogicalLevel,
    document::DocumentState,
    utils::actions::{OneShot, SetOneShotDone},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Marker {
    Section(DocumentElementUri),
    SymbolReference {
        uri: SymbolUri,
        notation: Option<UriName>,
        in_term: bool,
    },
    InputRef {
        target: DocumentUri,
        uri: DocumentElementUri,
    },
    SkipSection,
    SectionTitle,
    Comp,
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
    ($e:expr) => {{
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let children = owner.with(move || $e);
        leptos::tachys::reactive_graph::OwnedView::new_with_owner(children, owner)
    }};
}

impl Marker {
    pub fn apply<Views: FtmlViews + ?Sized>(
        mut markers: MarkerList,
        is_math: bool,
        mut orig: OriginalNode,
    ) -> impl IntoView {
        #[allow(clippy::enum_glob_use)]
        use leptos::either::EitherOf8::*;
        let Some(m) = markers.pop() else {
            return A(leptos_posthoc::DomCont(leptos_posthoc::DomContProps {
                orig,
                cont: super::iterate::<Views>,
                skip_head: true,
                class: None::<String>.into(),
                style: None::<String>.into(),
            }));
        };
        match m {
            Self::Section(uri) => B(owned!(DocumentState::new_section(uri, move |info| {
                Views::section(info, move || {
                    Self::apply::<Views>(markers, is_math, orig).into_any()
                })
            }))),
            Self::SkipSection => C(owned!(DocumentState::skip_section(move || {
                Self::apply::<Views>(markers, is_math, orig).into_any()
            }))),
            Self::SectionTitle => {
                let (LogicalLevel::Section(lvl), cls) = DocumentState::title_class() else {
                    tracing::error!("Unexpected section title");
                    return D(Self::apply::<Views>(markers, is_math, orig).into_any());
                };
                E(owned!(Views::section_title(lvl, cls, move || {
                    Self::apply::<Views>(markers, is_math, orig).into_any()
                })))
            }
            Self::Comp => F(owned!(Views::comp(move || {
                Self::apply::<Views>(markers, is_math, orig).into_any()
            }))),
            Self::InputRef { target, uri } => G(owned!(DocumentState::do_inputref(
                target,
                uri,
                Views::inputref
            ))),
            Self::SymbolReference {
                uri,
                notation,
                in_term: false,
            } => H(owned!(Views::symbol_reference(
                uri,
                notation,
                is_math,
                move || {
                    // makes sure the "current orig" gets actually used / hydrated first
                    // just in case it has, like, listeners or something
                    let clone = orig.deep_clone();
                    Self::apply::<Views>(markers, is_math, std::mem::replace(&mut orig, clone))
                        .into_any()
                }
            ))),
            Self::SymbolReference { uri, notation, .. } => ftml_core::TODO!(),
        }
    }

    pub fn from(elem: &OpenFtmlElement) -> Option<Self> {
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
            OpenFtmlElement::SymbolReference {
                uri,
                notation,
                in_term,
            } => Some(Self::SymbolReference {
                uri: uri.clone(),
                notation: notation.clone(),
                in_term: *in_term,
            }),
            OpenFtmlElement::Counter(_)
            | OpenFtmlElement::Invisible
            | OpenFtmlElement::None
            | OpenFtmlElement::Module { .. }
            | OpenFtmlElement::Style(_)
            | OpenFtmlElement::SymbolDeclaration { .. } => None,
        }
    }
}
