use ftml_core::extraction::OpenFtmlElement;
use ftml_uris::DocumentElementUri;
use leptos::{
    IntoView,
    prelude::{IntoAny, Memo},
};
use leptos_posthoc::OriginalNode;

use crate::{FtmlViews, counters::LogicalLevel, document::DocumentState};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Marker {
    Section(DocumentElementUri),
    SkipSection,
    SectionTitle,
}

impl Marker {
    pub fn apply<Views: FtmlViews + ?Sized>(
        markers: impl IntoIterator<Item = Self>,
        is_math: bool,
        orig: OriginalNode,
    ) -> impl IntoView {
        #[allow(clippy::enum_glob_use)]
        use leptos::either::EitherOf5::*;
        let mut markers = markers.into_iter();
        let Some(m) = markers.next() else {
            return A(leptos_posthoc::DomChildrenCont(
                leptos_posthoc::DomChildrenContProps {
                    orig,
                    cont: super::iterate::<Views>,
                },
            ));
        };
        match m {
            Self::Section(uri) => B(DocumentState::new_section(uri, move |info| {
                Views::section(info, move || {
                    Self::apply::<Views>(markers, is_math, orig).into_any()
                })
            })),
            Self::SkipSection => C(DocumentState::skip_section(move || {
                Self::apply::<Views>(markers, is_math, orig).into_any()
            })),
            Self::SectionTitle => {
                let (LogicalLevel::Section(lvl), cls) = DocumentState::title_class() else {
                    tracing::error!("Unexpected section title");
                    return D(Self::apply::<Views>(markers, is_math, orig).into_any());
                };
                E(Views::section_title(lvl, cls, move || {
                    Self::apply::<Views>(markers, is_math, orig).into_any()
                }))
            }
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
            OpenFtmlElement::SkipSection => Some(Self::SkipSection),
            OpenFtmlElement::SectionTitle => Some(Self::SectionTitle),
            OpenFtmlElement::Section(uri) => Some(Self::Section(uri.clone())),
            OpenFtmlElement::Counter(_)
            | OpenFtmlElement::Invisible
            | OpenFtmlElement::None
            | OpenFtmlElement::Module { .. }
            | OpenFtmlElement::Style(_)
            | OpenFtmlElement::Symbol { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SectionInfo {
    pub uri: DocumentElementUri,
    pub style: Option<Memo<String>>,
    pub class: Option<&'static str>,
    pub lvl: LogicalLevel,
    pub id: String,
}
