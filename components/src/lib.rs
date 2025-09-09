#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
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
use ftml_dom::{toc::TocSource, utils::local_cache::SendBackend};
use ftml_uris::DocumentUri;
use leptos::IntoView;
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SidebarPosition {
    #[default]
    Find,
    Next,
    None,
}

#[derive(Copy,Clone)]
pub struct InFtmlTop;

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

    pub fn document<Ch: IntoView + 'static>(
        uri: DocumentUri,
        sidebar: SidebarPosition,
        is_stripped: bool,
        children: impl FnOnce() -> Ch + 'static,
    ) -> impl IntoView {
        use leptos::{
            either::Either::{Left, Right},
            prelude::*,
        };
        ftml_dom::setup_document(uri, is_stripped, move || {
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
    }
}
