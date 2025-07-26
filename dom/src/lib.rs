#![allow(unexpected_cfgs)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod counters;
mod document;
pub mod extractor;
pub mod markers;
pub mod mathml;
pub mod toc;
pub mod utils {
    pub mod actions;
    pub mod css;
    pub mod local_cache;
}

use crate::{
    extractor::FtmlDomElement,
    markers::{InputrefInfo, Marker, SectionInfo},
};
pub use document::{DocumentMeta, DocumentState, setup_document};
use ftml_core::extraction::FtmlExtractor;
use ftml_ontology::{narrative::elements::SectionLevel, terms::Variable};
use ftml_uris::{SymbolUri, UriName};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

#[inline]
pub fn global_setup<V: IntoView>(f: impl FnOnce() -> V) -> impl IntoView {
    #[cfg(feature = "ssr")]
    provide_context(utils::css::CssIds::default());
    f()
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum VarOrSym {
    S(SymbolUri),
    V(Variable),
}

pub trait FtmlViews: 'static {
    fn render_ftml(html: String) -> impl IntoView {
        use leptos_posthoc::{DomStringCont, DomStringContProps};
        DomStringCont(DomStringContProps {
            html,
            cont: iterate::<Self>,
            on_load: None,
            class: None::<String>.into(),
            style: None::<String>.into(),
        })
    }
    fn render_math_ftml(html: String) -> impl IntoView {
        use leptos_posthoc::{DomStringContMath, DomStringContMathProps};
        DomStringContMath(DomStringContMathProps {
            html,
            cont: iterate::<Self>,
            on_load: None,
            class: None::<String>.into(),
            style: None::<String>.into(),
        })
    }

    #[inline]
    fn cont(node: OriginalNode) -> impl IntoView {
        use leptos_posthoc::{DomChildrenCont, DomChildrenContProps};
        DomChildrenCont(DomChildrenContProps {
            orig: node,
            cont: iterate::<Self>,
        })
    }
    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        global_setup(then)
    }

    #[inline]
    fn section<V: IntoView>(_info: SectionInfo, then: impl FnOnce() -> V) -> impl IntoView {
        then()
    }

    #[inline]
    fn symbol_reference<V: IntoView + 'static>(
        _uri: SymbolUri,
        _notation: Option<UriName>,
        _is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn section_title<V: IntoView>(
        _lvl: SectionLevel,
        _class: &'static str,
        then: impl FnOnce() -> V,
    ) -> impl IntoView {
        then()
    }

    fn inputref(_info: InputrefInfo) -> impl IntoView {}

    #[inline]
    fn comp<V: IntoView + 'static>(then: impl FnOnce() -> V) -> impl IntoView {
        then()
    }
}

fn iterate<Views: FtmlViews + ?Sized>(
    e: &leptos::web_sys::Element,
) -> (
    Option<impl FnOnce() -> AnyView + use<Views>>,
    Option<impl FnOnce() + use<Views>>,
) {
    use extractor::DomExtractor;
    use extractor::NodeAttrs;

    tracing::trace!("iterating {}", e.outer_html());
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        client::init();
        if !client::has_ftml_attribute(e) {
            tracing::trace!("No attributes");
            return (None, None);
        }
    }
    tracing::debug!("Has ftml attributes");
    let sig = expect_context::<RwSignal<DomExtractor>>();
    let (mut markers, close) = sig.update_untracked(|extractor| {
        let mut attrs = NodeAttrs::new(e);
        let rules = attrs.keys();
        let mut markers = smallvec::SmallVec::<_, 4>::new();
        let mut close = smallvec::SmallVec::<_, 2>::new();
        for r in rules.apply(extractor, &mut attrs) {
            match r {
                Ok((m, c)) => {
                    if let Some(m) = m {
                        markers.push(m);
                    }
                    if let Some(c) = c {
                        close.push(c);
                    }
                }
                Err(err) => {
                    tracing::error!("{err}");
                    leptos::web_sys::console::log_1(e);
                }
            }
        }
        (markers, close)
    });
    let rview = if markers.is_empty() {
        tracing::debug!("No markers");
        None
    } else {
        tracing::debug!("got elements: {markers:?}");
        let e: OriginalNode = e.clone().into();
        Some(move || {
            markers.reverse();
            Marker::apply::<Views>(markers, mathml::is(&e.tag_name()).is_some(), e).into_any()
        })
    };
    let and_then = if close.is_empty() {
        None
    } else {
        let e = e.clone();
        Some(move || {
            tracing::debug!("closing element: {close:?}");
            sig.update_untracked(move |extractor| {
                let n = FtmlDomElement(&e);
                for c in close.into_iter().rev() {
                    if let Err(e) = extractor.close(c, &n) {
                        tracing::error!("{e}");
                    }
                }
            });
        })
    };
    (rview, and_then)
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
mod client {
    use wasm_bindgen::{JsCast, JsValue};
    static INIT: std::sync::Once = std::sync::Once::new();

    #[inline]
    pub fn init() {
        INIT.call_once(|| {
            let window = leptos::tachys::dom::window();

            web_sys::js_sys::Reflect::set(
                &JsValue::from(window.clone()),
                &JsValue::from("hasFtmlAttribute"),
                &JsValue::from(web_sys::js_sys::Function::new_with_args(
                    "node",
                    include_str!("hasFtmlAttribute.js"),
                )),
            )
            .expect("error defining js function");

            #[cfg(feature = "csr")]
            web_sys::js_sys::Reflect::set(
                &JsValue::from(window),
                &JsValue::from("FLAMS_SERVER_URL"),
                &JsValue::from("https://mathhub.info"),
            )
            .expect("error setting Window property");
        });
    }

    std::thread_local! {
        static HAS_FTML_ATTRIBUTE: std::cell::LazyCell<web_sys::js_sys::Function> =
            const { std::cell::LazyCell::new(|| {
                let window = leptos::tachys::dom::window();
                let ga = window
                    .get("hasFtmlAttribute")
                    .expect("error getting Window property");
                ga.dyn_into()
                    .expect("Window.hasFtmlAttribute is not a function")
            })  };
    }

    pub fn has_ftml_attribute(node: &web_sys::Node) -> bool {
        HAS_FTML_ATTRIBUTE.with(|o| {
            o.call1(&JsValue::NULL, &JsValue::from(node))
                .expect("error calling hasFtmlAttribute")
                .as_bool()
                .expect("error calling hasFtmlAttribute")
        })
    }
}
