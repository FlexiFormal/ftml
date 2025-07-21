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
}

impl Marker {
    pub fn apply<Views: FtmlViews + ?Sized>(
        markers: impl IntoIterator<Item = Self>,
        is_math: bool,
        orig: OriginalNode,
    ) -> impl IntoView {
        use leptos::either::Either::{Left, Right};
        let mut markers = markers.into_iter();
        let Some(m) = markers.next() else {
            return Left(leptos_posthoc::DomChildrenCont(
                leptos_posthoc::DomChildrenContProps {
                    orig,
                    cont: super::iterate::<Views>,
                },
            ));
        };
        match m {
            Self::Section(uri) => Right(DocumentState::new_section(uri, move |info| {
                Views::section(info, move || {
                    Self::apply::<Views>(markers, is_math, orig).into_any()
                })
            })),
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
