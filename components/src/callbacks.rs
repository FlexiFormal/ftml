use ftml_ontology::narrative::elements::{SectionLevel, paragraphs::ParagraphKind};
use ftml_uris::DocumentElementUri;
use leptos::prelude::*;

use crate::config::FtmlConfig;

#[cfg(feature = "callbacks")]
leptos_react::wrapper!(SectionWrap(u:DocumentElementUri,lvl:SectionLevel));
#[cfg(feature = "callbacks")]
leptos_react::wrapper!(ParagraphWrap(u:DocumentElementUri,kind:ParagraphKind));
#[cfg(feature = "callbacks")]
leptos_react::wrapper!(ProblemWrap(u:DocumentElementUri,sub_problem:bool,autogradable:bool));
#[cfg(feature = "callbacks")]
leptos_react::wrapper!(SlideWrap(u:DocumentElementUri));
#[cfg(feature = "callbacks")]
leptos_react::insertion!(OnSectionTitle(u:DocumentElementUri,lvl: SectionLevel));

impl FtmlConfig {
    #[allow(unused_variables)]
    pub fn wrap_section<V: IntoView, F: FnOnce() -> V>(
        uri: &DocumentElementUri,
        lvl: Option<SectionLevel>,
        children: F,
    ) -> impl IntoView + use<V, F> {
        #[cfg(not(feature = "callbacks"))]
        {
            children()
        }
        #[cfg(feature = "callbacks")]
        {
            use leptos::either::Either::{Left, Right};
            let lvl = lvl.unwrap_or(SectionLevel::Subparagraph);
            if let Some(Some(w)) = use_context::<Option<SectionWrap>>() {
                Left(w.wrap(uri, &lvl, children))
            } else {
                Right(children())
            }
        }
    }

    #[allow(unused_variables)]
    pub fn wrap_paragraph<V: IntoView, F: FnOnce() -> V>(
        uri: &DocumentElementUri,
        kind: ParagraphKind,
        children: F,
    ) -> impl IntoView + use<V, F> {
        #[cfg(not(feature = "callbacks"))]
        {
            children()
        }
        #[cfg(feature = "callbacks")]
        {
            use leptos::either::Either::{Left, Right};

            if let Some(Some(w)) = use_context::<Option<ParagraphWrap>>() {
                Left(w.wrap(uri, &kind, children))
            } else {
                Right(children())
            }
        }
    }

    #[allow(unused_variables)]
    pub fn wrap_slide<V: IntoView, F: FnOnce() -> V>(
        uri: &DocumentElementUri,
        children: F,
    ) -> impl IntoView + use<V, F> {
        #[cfg(not(feature = "callbacks"))]
        {
            children()
        }
        #[cfg(feature = "callbacks")]
        {
            use leptos::either::Either::{Left, Right};

            if let Some(Some(w)) = use_context::<Option<SlideWrap>>() {
                Left(w.wrap(uri, children))
            } else {
                Right(children())
            }
        }
    }

    #[allow(unused_variables)]
    pub fn wrap_problem<V: IntoView, F: FnOnce() -> V>(
        uri: &DocumentElementUri,
        sub_problem: bool,
        autogradable: bool,
        children: F,
    ) -> impl IntoView + use<V, F> {
        #[cfg(not(feature = "callbacks"))]
        {
            children()
        }
        #[cfg(feature = "callbacks")]
        {
            use leptos::either::Either::{Left, Right};

            if let Some(Some(w)) = use_context::<Option<ProblemWrap>>() {
                Left(w.wrap(uri, &sub_problem, &autogradable, children))
            } else {
                Right(children())
            }
        }
    }

    #[must_use]
    #[allow(unused_variables)]
    pub fn insert_section_title(lvl: SectionLevel) -> impl IntoView + use<> {
        #[cfg(not(feature = "callbacks"))]
        {
            None::<&'static str>
        }
        #[cfg(feature = "callbacks")]
        {
            use ftml_dom::DocumentState;
            use ftml_uris::NarrativeUri;
            if let Some(Some(w)) = use_context::<Option<OnSectionTitle>>() {
                let NarrativeUri::Element(uri) = DocumentState::current_uri() else {
                    tracing::error!("Could not determine URI for current section");
                    return None;
                };
                Some(w.insert(&uri, &lvl))
            } else {
                None
            }
        }
    }
}
