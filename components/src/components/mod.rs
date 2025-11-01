pub mod content;
pub mod inputref;
pub mod notations;
pub mod paragraphs;
pub mod problems;
pub mod sections;
pub mod sidebar;
pub mod terms;
pub mod toc;

use crate::config::FtmlConfig;
use ftml_dom::{
    ClonableView, TermTrackedViews,
    structure::{Inputref, SectionInfo},
    terms::ReactiveApplication,
    utils::local_cache::SendBackend,
};
use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::{DocumentElementUri, Id};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

impl<B: SendBackend> TermTrackedViews for crate::Views<B> {
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        use crate::utils::theming::Themer;
        ftml_dom::global_setup(|| {
            view!(
                <Themer>
                    {
                        FtmlConfig::init();
                        then()
                    }
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
    fn section_title(class: &'static str, then: OriginalNode) -> impl IntoView {
        sections::section_title(class, then)
    }

    #[inline]
    fn paragraph<V: IntoView>(
        info: ftml_dom::markers::ParagraphInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        paragraphs::paragraph(info, then)
    }

    #[inline]
    fn proof_body(then: OriginalNode) -> impl IntoView {
        paragraphs::proof_body::<B>(then)
    }

    #[inline]
    fn paragraph_title(then: OriginalNode) -> impl IntoView {
        paragraphs::title::<B>(then)
    }

    #[inline]
    fn problem<V: IntoView>(
        uri: DocumentElementUri,
        styles: Box<[Id]>,
        style: Memo<String>,
        class: String,
        is_subproblem: bool,
        autogradable: bool,
        points: Option<f32>,
        minutes: Option<f32>,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        problems::problem::<B, _>(
            uri,
            styles,
            style,
            class,
            is_subproblem,
            autogradable,
            points,
            minutes,
            then,
        )
    }

    #[inline]
    fn problem_solution() -> impl IntoView {
        problems::solution::<B>()
    }

    #[inline]
    fn fillinsol(width: Option<f32>) -> impl IntoView {
        problems::fillinsol(width)
    }

    #[inline]
    fn problem_hint<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        problems::hint(then)
    }

    #[inline]
    fn problem_gnote() -> impl IntoView {
        problems::gnote()
    }

    #[inline]
    fn multiple_choice_block<V: IntoView + 'static>(
        style: ftml_ontology::narrative::elements::problems::ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        problems::choice_block(true, style, then)
    }

    #[inline]
    fn single_choice_block<V: IntoView + 'static>(
        style: ftml_ontology::narrative::elements::problems::ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        problems::choice_block(false, style, then)
    }

    fn problem_choice<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        problems::choice(then)
    }

    #[inline]
    fn slide<V: IntoView>(
        uri: DocumentElementUri,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        paragraphs::slide(uri, then)
    }

    #[inline]
    fn inputref(info: Inputref) -> impl IntoView {
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
    fn def_comp(uri: Option<ftml_uris::SymbolUri>, then: ClonableView) -> impl IntoView {
        terms::defcomp::<B>(uri, then)
    }

    #[inline]
    fn application(
        head: ReadSignal<ReactiveApplication>,
        _notation: Option<Id>,
        _uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        terms::oma::<B>(false, head, then)
    }

    #[inline]
    fn binder_application(
        head: ReadSignal<ReactiveApplication>,
        _notation: Option<Id>,
        _uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        terms::oma::<B>(true, head, then)
    }
}
