#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod components;
pub mod config;
pub mod utils;

use ftml_dom::{FtmlViews, markers::SectionInfo};
use leptos::prelude::*;

struct Views;
impl FtmlViews for Views {
    fn top(node: leptos_posthoc::OriginalNode) -> impl IntoView {
        use utils::theming::Themer;
        ftml_dom::global_setup(|| {
            view!(
                <Themer attr:style="\
                    font-family:inherit;\
                    font-size:inherit;\
                    font-weight:inherit;\
                    line-height:inherit;\
                    background-color:inherit;\
                ">
                    {Self::cont(node)}
                </Themer>
            )
        })
    }

    #[inline]
    fn section<V: IntoView>(info: SectionInfo, then: impl FnOnce() -> V) -> impl IntoView {
        components::sections::section(info, then)
    }
}

/// Activate FTML viewer on the entire body of the page
#[cfg(feature = "csr")]
#[allow(clippy::semicolon_if_nothing_returned)]
pub fn iterate_body() {
    leptos_posthoc::hydrate_body(|orig| {
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

        ftml_dom::setup_document(|| Views::top(orig), uri)
        /*
        view!(
            <Themer attr:style="font-family:inherit;font-size:inherit;font-weight:inherit;line-height:inherit;background-color:inherit;">
                <FTMLGlobalSetup>
                    <FTMLDocumentSetup uri=DocumentUri::no_doc().clone()>
                        <DomChildrenCont orig cont=ftml_viewer_components::iterate/>
                    </FTMLDocumentSetup>
                </FTMLGlobalSetup>
            </Themer>
        )
         */
    })
}
