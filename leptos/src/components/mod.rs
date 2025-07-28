pub mod inputref;
pub mod notations;
pub mod sections;
pub mod terms;

use crate::config::FtmlConfig;
use ftml_dom::{
    FtmlViews, TermTrackedViews, markers::SectionInfo, terms::ReactiveApplication,
    utils::local_cache::SendBackend,
};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;

impl<B: SendBackend> TermTrackedViews for crate::Views<B> {
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + 'static + Send) -> impl IntoView {
        use crate::utils::theming::Themer;
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
        sections::section(info, then)
    }
    #[inline]
    fn section_title<V: IntoView>(
        lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V,
    ) -> impl IntoView {
        sections::section_title(lvl, class, then)
    }

    #[inline]
    fn symbol_reference<V: IntoView + 'static>(
        uri: ftml_uris::SymbolUri,
        _notation: Option<ftml_uris::UriName>,
        is_math: bool,
        in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        use leptos::either::Either::{Left, Right};
        if is_math {
            Left(terms::oms::<B, _>(uri, in_term, then))
        } else {
            Right(terms::symbol_reference::<B, _>(uri, then))
        }
    }
    fn variable_reference<V: IntoView + 'static>(
        var: ftml_ontology::terms::Variable,
        _notation: Option<ftml_uris::UriName>,
        is_math: bool,
        in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        use leptos::either::Either::{Left, Right};
        if is_math {
            Left(terms::omv::<B, _>(var, in_term, then))
        } else {
            Right(terms::variable_reference::<B, _>(var, then))
        }
    }

    #[inline]
    fn comp<V: IntoView + 'static>(then: impl FnOnce() -> V) -> impl IntoView {
        terms::comp::<B, _>(then)
    }

    #[inline]
    fn inputref(info: ftml_dom::markers::InputrefInfo) -> impl IntoView {
        inputref::inputref::<B>(info)
    }

    #[inline]
    fn application<V: IntoView + 'static>(
        head: ReadSignal<ReactiveApplication>,
        _notation: Option<ftml_uris::UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        terms::oma::<B, _, _>(head, is_math, then)
    }

    #[inline]
    fn binder_application<V: IntoView + 'static>(
        head: ReadSignal<ReactiveApplication>,
        _notation: Option<ftml_uris::UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        terms::oma::<B, _, _>(head, is_math, then)
    }
}
