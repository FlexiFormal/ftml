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

use std::marker::PhantomData;

use ftml_dom::{FtmlViews, markers::SectionInfo, utils::local_cache::SendBackend};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;

use crate::config::FtmlConfig;

pub struct Views<B: SendBackend>(PhantomData<B>);
impl<B: SendBackend> FtmlViews for Views<B> {
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + 'static + Send) -> impl IntoView {
        use utils::theming::Themer;
        ftml_dom::global_setup(|| {
            view!(
                <Themer attr:style="\
                    font-family:inherit;\
                    font-size:inherit;\
                    font-weight:inherit;\
                    line-height:inherit;\
                    background-color:inherit;\
                    color:inherit;\
                    display:contents;
                ">
                    {
                        FtmlConfig::init();
                        then()
                    }
                    //{Self::cont(node)}
                </Themer>
            )
        })
    }

    #[inline]
    fn section<V: IntoView>(info: SectionInfo, then: impl FnOnce() -> V) -> impl IntoView {
        components::sections::section(info, then)
    }
    #[inline]
    fn section_title<V: IntoView>(
        lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V,
    ) -> impl IntoView {
        components::sections::section_title(lvl, class, then)
    }

    #[inline]
    fn symbol_reference<V: IntoView + 'static>(
        uri: ftml_uris::SymbolUri,
        _notation: Option<ftml_uris::UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        use leptos::either::Either::{Left, Right};
        if is_math {
            Left(components::terms::oms::<B, _>(uri, false, then))
        } else {
            Right(components::terms::symbol_reference::<B, _>(uri, then))
        }
    }

    #[inline]
    fn comp<V: IntoView + 'static>(then: impl FnOnce() -> V) -> impl IntoView {
        components::terms::comp::<B, _>(then)
    }

    #[inline]
    fn inputref(info: ftml_dom::markers::InputrefInfo) -> impl IntoView {
        components::inputref::inputref::<B>(info)
    }
}

/// Activate FTML viewer on the entire body of the page
#[cfg(feature = "csr")]
#[allow(clippy::semicolon_if_nothing_returned)]
pub fn iterate_body<B: SendBackend>() {
    leptos_posthoc::hydrate_body(move |orig| {
        use ftml_uris::DocumentUri;

        let mut meta = ftml_dom::DocumentMeta::get();
        if let Ok(scripts) = leptos::tachys::dom::document().query_selector_all("head script") {
            let mut i = 0;
            while let Some(node) = scripts.get(i) {
                use leptos::wasm_bindgen::JsCast;
                i += 1;
                let Ok(elem) = node.dyn_into::<leptos::web_sys::Element>() else {
                    continue;
                };
                if elem.get_attribute("src").is_none()
                    && elem
                        .get_attribute("type")
                        .is_some_and(|s| s == "application/json")
                    && elem.get_attribute("id").is_some_and(|s| s == "ftml")
                {
                    let inner = elem.inner_html();
                    match serde_json::from_str::<config::FtmlConfig>(&inner) {
                        Ok(cfg) => {
                            if let Some(uri) = cfg.apply() {
                                meta.uri = Some(uri);
                            }
                        }
                        Err(e) => tracing::error!("failed to deserialize ftml config json: {e}"),
                    }
                }
            }
        }
        let uri = meta.uri.unwrap_or_else(|| DocumentUri::no_doc().clone());

        Views::<B>::top(|| ftml_dom::setup_document(uri, || Views::<B>::cont(orig)))
    })
}
