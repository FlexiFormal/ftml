#![recursion_limit = "256"]
#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod callbacks;
pub mod components;
pub mod config;
pub mod utils;

use crate::{components::paragraphs::Slides, config::FtmlConfig};
use ftml_backend::{SendBackend, dynbackend::DynBackend};
use ftml_dom::{
    DocumentState,
    structure::DocumentStructure,
    toc::{TocSource, TocStyle},
};
use ftml_ontology::narrative::documents::Document;
use ftml_uris::{DocumentUri, NarrativeUri};
use leptos::{
    IntoView,
    html::{ElementChild, div},
    prelude::{AnyView, IntoAny, use_context},
};

static GLOBAL_BACKEND: std::sync::RwLock<Option<&'static dyn DynBackend>> =
    std::sync::RwLock::new(None);
static CONTINUATIONS: std::sync::RwLock<&'static dyn ViewContinuations> =
    std::sync::RwLock::new(&NoContinuations);
/// #### Panics
pub fn backend() -> &'static dyn DynBackend {
    GLOBAL_BACKEND
        .read()
        .expect("Backend not set")
        .expect("Backend not set")
}
/// #### Panics
pub fn set_backend<Be: SendBackend>() {
    *GLOBAL_BACKEND.write().expect("error") = Some(Be::as_dyn());
}
/// #### Panics
pub fn continuations() -> &'static dyn ViewContinuations {
    *CONTINUATIONS.read().expect("Error")
}
/// #### Panics
pub fn set_continuation(cont: &'static impl ViewContinuations) {
    *CONTINUATIONS.write().expect("Error") = cont;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SidebarPosition {
    #[default]
    Find,
    Next,
    None,
}

#[derive(Copy, Clone)]
struct InFtmlTop;

pub trait ViewContinuations: Sync + 'static {
    fn document_drawer(&self, doc: &Document) -> AnyView;
}
pub struct NoContinuations;
impl ViewContinuations for NoContinuations {
    fn document_drawer(&self, _: &Document) -> AnyView {
        ().into_any()
    }
}

pub struct Views;
impl Views {
    pub fn top_safe<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        use crate::utils::theming::Themer;
        use leptos::prelude::*;
        ftml_dom::global_setup(|| {
            provide_context(InFtmlTop);
            view!(
                <Themer safe=true>
                    {
                        FtmlConfig::init();
                        then()
                    }
                </Themer>
            )
        })
    }

    pub fn maybe_top<V: IntoView + Send + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        if use_context::<InFtmlTop>().is_some() {
            leptos::either::Either::Left(then())
        } else {
            leptos::either::Either::Right(Self::top_safe(then))
        }
    }

    pub fn setup_document(
        uri: DocumentUri,
        sidebar: SidebarPosition,
        is_stripped: bool,
        toc: TocSource,
        children: impl FnOnce() -> AnyView + Send + 'static,
    ) -> AnyView {
        use leptos::prelude::*;
        Self::maybe_top(move || {
            ftml_dom::setup_document(uri, is_stripped, toc, crate::backend(), move || {
                let (v, s) = Slides::new();
                provide_context(s);
                let children = move || view! {{children()}{v}}.into_any();
                let show_content = FtmlConfig::show_content();
                let pdf_link = FtmlConfig::pdf_link();
                let choose_highlight_style = FtmlConfig::choose_highlight_style();
                let do_sidebar = sidebar != SidebarPosition::None
                    && (
                        show_content
                            || pdf_link
                            || choose_highlight_style
                            || DocumentStructure::toc_style() != TocStyle::None
                        //FtmlConfig::with_toc_source(|toc| !matches!(toc, TocSource::None)).is_some_and(|b| b)
                    );
                if do_sidebar {
                    components::sidebar::do_sidebar(
                        show_content,
                        pdf_link,
                        choose_highlight_style,
                        sidebar == SidebarPosition::Find,
                        children,
                    )
                } else {
                    children()
                }
            })
        })
        .into_any()
    }

    pub fn render_fragment(
        uri: Option<NarrativeUri>,
        sidebar: SidebarPosition,
        is_stripped: bool,
        toc: TocSource,
        children: impl FnOnce() -> AnyView + Send + 'static,
    ) -> AnyView {
        let (doc, wrap) = if let Some(NarrativeUri::Document(d)) = &uri {
            (d.clone(), false)
        } else {
            (DocumentUri::no_doc().clone(), true)
        };
        let inner = Self::maybe_top(move || {
            Self::setup_document(doc, sidebar, is_stripped, toc, move || {
                if let Some(NarrativeUri::Element(uri)) = uri {
                    DocumentState::force_uri(uri);
                }
                children()
            })
        })
        .into_any();
        if wrap {
            div().child(inner).into_any() //.style("padding: 0 60px;--rustex-this-width:590px;"),
        } else {
            inner
        }
    }
}
