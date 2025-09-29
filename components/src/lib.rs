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
use ftml_dom::{DocumentState, toc::TocSource, utils::local_cache::SendBackend};
use ftml_uris::{DocumentUri, NarrativeUri};
use leptos::{
    IntoView,
    html::{ElementChild, div},
    prelude::use_context,
};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SidebarPosition {
    #[default]
    Find,
    Next,
    None,
}

#[derive(Copy, Clone)]
struct InFtmlTop;

pub struct Views<B: SendBackend>(PhantomData<B>);
impl<B: SendBackend> Views<B> {
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

    pub fn maybe_top<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        if use_context::<InFtmlTop>().is_some() {
            leptos::either::Either::Left(then())
        } else {
            leptos::either::Either::Right(Self::top_safe(then))
        }
    }

    pub fn setup_document<Ch: IntoView + 'static>(
        uri: DocumentUri,
        sidebar: SidebarPosition,
        is_stripped: bool,
        children: impl FnOnce() -> Ch + Send + 'static,
    ) -> impl IntoView {
        use leptos::{
            either::Either::{Left, Right},
            prelude::*,
        };
        Self::maybe_top(move || {
            ftml_dom::setup_document(uri.clone(), is_stripped, move || {
                let (v, s) = Slides::new();
                provide_context(s);
                let children = move || view! {{children()}{v}};
                let show_content = FtmlConfig::show_content();
                let pdf_link = FtmlConfig::pdf_link();
                let choose_highlight_style = FtmlConfig::choose_highlight_style();
                let do_sidebar = sidebar != SidebarPosition::None
                    && (show_content
                        || pdf_link
                        || choose_highlight_style
                        || FtmlConfig::with_toc_source(|toc| !matches!(toc, TocSource::None))
                            .is_some_and(|b| b));
                if do_sidebar {
                    Left(components::sidebar::do_sidebar::<B, _>(
                        uri,
                        show_content,
                        pdf_link,
                        choose_highlight_style,
                        sidebar == SidebarPosition::Find,
                        children,
                    ))
                } else {
                    Right(children())
                }
            })
        })
    }

    pub fn render_fragment<Ch: IntoView + 'static>(
        uri: Option<NarrativeUri>,
        sidebar: SidebarPosition,
        is_stripped: bool,
        children: impl FnOnce() -> Ch + Send + 'static,
    ) -> impl IntoView {
        let (doc, wrap) = if let Some(NarrativeUri::Document(d)) = &uri {
            (d.clone(), false)
        } else {
            (DocumentUri::no_doc().clone(), true)
        };
        let inner = Self::maybe_top(move || {
            Self::setup_document(doc, sidebar, is_stripped, move || {
                if let Some(NarrativeUri::Element(uri)) = uri {
                    DocumentState::force_uri(uri);
                }
                children()
            })
        });
        if wrap {
            leptos::either::Either::Left(
                div().child(inner), //.style("padding: 0 60px;--rustex-this-width:590px;"),
            )
        } else {
            leptos::either::Either::Right(inner)
        }
    }
}
