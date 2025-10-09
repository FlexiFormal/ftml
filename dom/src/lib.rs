#![allow(unexpected_cfgs)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

mod clonable_views;
pub mod counters;
mod document;
pub(crate) mod extractor;
pub mod markers;
pub mod mathml;
pub mod notations;
pub mod terms;
pub mod toc;
pub mod utils;
pub use clonable_views::ClonableView;
use ftml_ontology::narrative::elements::{ParagraphOrProblemKind, paragraphs::ParagraphKind};
use ftml_uris::DocumentUri;
use smallvec::SmallVec;
mod views;

use crate::{
    extractor::{DomExtractor, ExtractorMode, FtmlDomElement},
    markers::Marker,
    terms::ReactiveApplication,
    utils::local_cache::LOCAL_CACHE,
};
pub use document::{DocumentMeta, DocumentState, setup_document};
use ftml_parser::extraction::{CloseFtmlElement, FtmlExtractor};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;
pub use views::*;

#[inline]
pub fn global_setup<V: IntoView>(f: impl FnOnce() -> V) -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if use_context::<utils::css::CssIds>().is_none() {
            provide_context(utils::css::CssIds::default());
        }
    }
    f()
}

pub fn iterate<Views: FtmlViews + ?Sized>(
    e: &leptos::web_sys::Element,
) -> (
    Option<impl FnOnce() -> AnyView + use<Views>>,
    Option<impl FnOnce() + use<Views>>,
) {
    use extractor::DomExtractor;
    use extractor::NodeAttrs;

    //provide_context(OwnerId::new());
    let Some(sig) = use_context::<RwSignal<DomExtractor>>() else {
        return (None, None);
    };
    let finish = sig.update_untracked(|ext| {
        ext.mode == ExtractorMode::Pending && {
            ext.mode = ExtractorMode::Extracting;
            tracing::info!("Starting extracting {}", ext.state.document);
            true
        }
    });

    tracing::trace!("iterating {}", e.outer_html());
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        client::init();
        if !client::has_ftml_attribute(e) && !finish {
            tracing::trace!("No attributes");
            return (None, None);
        }
    }
    let n = FtmlDomElement::new(e.clone());

    tracing::trace!("Has ftml attributes");
    let (mut markers, invisible, close) = sig.update_untracked(|extractor| {
        let mut attrs = NodeAttrs::new(e);
        let rules = attrs.keys();
        let mut markers = smallvec::SmallVec::<_, 4>::new();
        let mut close = smallvec::SmallVec::<_, 2>::new();
        for r in rules.apply(extractor, &mut attrs, &n) {
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
        (markers, extractor.invisible(), close)
    });
    let rview = if markers.is_empty() {
        tracing::trace!("No markers");
        None
    } else {
        tracing::debug!("got elements: {markers:?}");
        let e: OriginalNode = e.clone().into();
        Some(move || {
            markers.reverse();
            //provide_context(sig);
            Marker::apply::<Views>(markers, invisible, mathml::is(&e.tag_name()).is_some(), e)
                .into_any()
        })
    };

    let and_then = if close.is_empty() && !finish {
        None
    } else {
        Some(move || close_things(close, sig, invisible, finish, n))
    };
    (rview, and_then)
}

fn close_things(
    close: SmallVec<CloseFtmlElement, 2>,
    sig: RwSignal<DomExtractor>,
    invisible: bool,
    finish: bool,
    n: FtmlDomElement,
) {
    let mut closes = close.clone();
    closes.reverse();
    tracing::trace!("closing element: {close:?}");
    sig.update_untracked(move |extractor| {
        for c in close.into_iter().rev() {
            if let Err(e) = extractor.close(c, &n) {
                tracing::error!("{e}");
                leptos::web_sys::console::log_1(&n.0);
            }
        }
    });
    if !invisible {
        for c in closes {
            #[allow(clippy::enum_glob_use)]
            use CloseFtmlElement::*;
            match c {
                OMA | OMBIND => ReactiveApplication::close(),
                Paragraph => {
                    if sig.with_untracked(|r| r.state.document == *DocumentUri::no_doc()) {
                        tracing::debug!("No document; paragraph ignored");
                    } else {
                        add_paragraph(sig);
                    }
                }
                Module
                | SymbolDeclaration
                | Invisible
                | Section
                | SectionTitle
                | SkipSection
                | SymbolReference
                | VariableReference
                | Argument
                | Type
                | Definiens
                | Notation
                | CompInNotation
                | NotationOpComp
                | NotationComp
                | ArgSep
                | MainCompInNotation
                | NotationArg
                | DocTitle
                | ReturnType
                | VariableDeclaration
                | Comp
                | DefComp
                | ParagraphTitle
                | SlideTitle
                | Slide
                | Definiendum
                | MathStructure
                | ComplexTerm
                | HeadTerm
                | OML
                | Morphism
                | Assign
                | ProblemTitle
                | Problem
                | Solution
                | FillinSol
                | ProblemHint
                | ProblemExNote
                | ProblemGradingNote
                | AnswerClass
                | ChoiceBlock
                | ProblemChoice
                | ArgTypes
                | ProblemChoiceFeedback
                | FillinSolCase
                | ProblemChoiceVerdict => (),
            }
        }
    }
    if finish {
        let r = sig.update_untracked(|r| {
            if r.is_stripped {
                Some(r.is_done.write_only())
            } else {
                r.finish()
            }
        });
        if let Some(r) = r {
            r.set(true);
        }
    }
}

fn add_paragraph(sig: RwSignal<DomExtractor>) {
    sig.with_untracked(|ext| {
        if let Some(p) = ext.last_paragraph() {
            let popk = if p.kind.is_definition_like(&p.styles) {
                ParagraphOrProblemKind::Definition
            } else {
                match p.kind {
                    ParagraphKind::Definition => ParagraphOrProblemKind::Definition,
                    ParagraphKind::Example => ParagraphOrProblemKind::Example,
                    _ => return,
                }
            };
            tracing::debug!("Adding paragraph for: {:?}", p.fors);
            for (s, _) in &p.fors {
                tracing::trace!("Adding local paragraph for {s}");
                match LOCAL_CACHE.fors.entry(s.clone()) {
                    dashmap::Entry::Occupied(mut v) => {
                        v.get_mut().push((p.uri.clone(), popk));
                    }
                    dashmap::Entry::Vacant(e) => {
                        e.insert(vec![(p.uri.clone(), popk)]);
                    }
                }
            }
        } else {
            tracing::warn!("No closing paragraph found!");
        }
    });
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
mod client {
    use wasm_bindgen::{JsCast, JsValue};
    static INIT: std::sync::Once = std::sync::Once::new();

    #[inline]
    pub fn init() {
        INIT.call_once(|| {
            let global = leptos::web_sys::js_sys::global();

            web_sys::js_sys::Reflect::set(
                &JsValue::from(global.clone()),
                &JsValue::from("hasFtmlAttribute"),
                &JsValue::from(web_sys::js_sys::Function::new_with_args(
                    "node",
                    include_str!("hasFtmlAttribute.js"),
                )),
            )
            .expect("error defining js function");

            #[cfg(feature = "csr")]
            web_sys::js_sys::Reflect::set(
                &JsValue::from(global),
                &JsValue::from("FTML_SERVER_URL"),
                &JsValue::from("https://mathhub.info"),
            )
            .expect("error setting Window property");
        });
    }

    std::thread_local! {
        static HAS_FTML_ATTRIBUTE: std::cell::LazyCell<web_sys::js_sys::Function> =
            const { std::cell::LazyCell::new(|| {
                let global = leptos::web_sys::js_sys::global();
                let ga = web_sys::js_sys::Reflect::get(&global,&JsValue::from_str("hasFtmlAttribute"))
                    .expect("error getting Window property");
                ga.dyn_into()
                    .expect("Global.hasFtmlAttribute is not a function")
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
