pub mod inputref;
pub mod notations;
pub mod omdoc;
pub mod sections;
pub mod terms;

use crate::config::FtmlConfig;
use ftml_dom::{
    ClonableView, TermTrackedViews, markers::SectionInfo, terms::ReactiveApplication,
    utils::local_cache::SendBackend,
};
use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::{DocumentElementUri, Id};
use leptos::prelude::*;

impl<B: SendBackend> TermTrackedViews for crate::Views<B> {
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
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
    fn section<V: IntoView>(
        info: SectionInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        sections::section(info, then)
    }
    #[inline]
    fn section_title<V: IntoView>(
        lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        sections::section_title(lvl, class, then)
    }

    #[inline]
    fn inputref(info: ftml_dom::markers::InputrefInfo) -> impl IntoView {
        inputref::inputref::<B>(info)
    }

    #[inline]
    fn symbol_reference(
        uri: ftml_uris::SymbolUri,
        _notation: Option<Id>,
        in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        use leptos::either::Either::{Left, Right};
        if then.is_math() {
            Left(terms::oms::<B>(uri, in_term, then))
        } else {
            Right(terms::symbol_reference::<B>(uri, then))
        }
    }
    fn variable_reference(
        var: ftml_ontology::terms::Variable,
        _notation: Option<Id>,
        in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        use leptos::either::Either::{Left, Right};
        if then.is_math() {
            Left(terms::omv::<B>(var, in_term, then))
        } else {
            Right(terms::variable_reference::<B>(var, then))
        }
    }

    #[inline]
    fn comp(then: ClonableView) -> impl IntoView {
        terms::comp::<B>(then)
    }

    #[inline]
    fn application(
        head: ReadSignal<ReactiveApplication>,
        _notation: Option<Id>,
        _uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        terms::oma::<B>(head, then)
    }

    #[inline]
    fn binder_application(
        head: ReadSignal<ReactiveApplication>,
        _notation: Option<Id>,
        _uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        terms::oma::<B>(head, then)
    }
}
