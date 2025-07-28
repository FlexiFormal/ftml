use crate::{
    markers::{InputrefInfo, SectionInfo},
    terms::ReactiveApplication,
};
use ftml_core::extraction::VarOrSym;
use ftml_ontology::{narrative::elements::SectionLevel, terms::Variable};
use ftml_uris::{SymbolUri, UriName};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

pub trait FtmlViews: 'static {
    fn render_ftml(html: String) -> impl IntoView {
        use leptos_posthoc::{DomStringCont, DomStringContProps};
        DomStringCont(DomStringContProps {
            html,
            cont: super::iterate::<Self>,
            on_load: None,
            class: None::<String>.into(),
            style: None::<String>.into(),
        })
    }
    fn render_math_ftml(html: String) -> impl IntoView {
        use leptos_posthoc::{DomStringContMath, DomStringContMathProps};
        DomStringContMath(DomStringContMathProps {
            html,
            cont: super::iterate::<Self>,
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
            cont: super::iterate::<Self>,
        })
    }
    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        super::global_setup(then)
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
        _in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn variable_reference<V: IntoView + 'static>(
        _var: Variable,
        _notation: Option<UriName>,
        _is_math: bool,
        _in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn application<V: IntoView + 'static>(
        _head: VarOrSym,
        _notation: Option<UriName>,
        _is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn binder_application<V: IntoView + 'static>(
        _head: VarOrSym,
        _notation: Option<UriName>,
        _is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }
    /*
    #[inline]
    fn argument<V: IntoView + 'static>(
        _position: ArgumentPosition,
        _is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }
     */

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

pub trait TermTrackedViews: 'static {
    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        super::global_setup(then)
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
        _in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn variable_reference<V: IntoView + 'static>(
        _var: Variable,
        _notation: Option<UriName>,
        _is_math: bool,
        _in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    fn application<V: IntoView + 'static>(
        head: ReadSignal<ReactiveApplication>,
        notation: Option<UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView;

    fn binder_application<V: IntoView + 'static>(
        head: ReadSignal<ReactiveApplication>,
        notation: Option<UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView;

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

impl<T: TermTrackedViews> FtmlViews for T {
    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        <T as TermTrackedViews>::top(then)
    }
    #[inline]
    fn section<V: IntoView>(info: SectionInfo, then: impl FnOnce() -> V) -> impl IntoView {
        <T as TermTrackedViews>::section(info, then)
    }
    #[inline]
    fn symbol_reference<V: IntoView + 'static>(
        uri: SymbolUri,
        notation: Option<UriName>,
        is_math: bool,
        in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::symbol_reference(uri, notation, is_math, in_term, then)
    }

    #[inline]
    fn variable_reference<V: IntoView + 'static>(
        var: Variable,
        notation: Option<UriName>,
        is_math: bool,
        in_term: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::variable_reference(var, notation, is_math, in_term, then)
    }

    #[inline]
    fn application<V: IntoView + 'static>(
        head: VarOrSym,
        notation: Option<UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        ReactiveApplication::track(head, move |a| {
            <T as TermTrackedViews>::application(a, notation, is_math, then)
        })
    }

    #[inline]
    fn binder_application<V: IntoView + 'static>(
        head: VarOrSym,
        notation: Option<UriName>,
        is_math: bool,
        then: impl FnOnce() -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        ReactiveApplication::track(head, move |a| {
            <T as TermTrackedViews>::binder_application(a, notation, is_math, then)
        })
    }

    #[inline]
    fn section_title<V: IntoView>(
        lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V,
    ) -> impl IntoView {
        <T as TermTrackedViews>::section_title(lvl, class, then)
    }

    #[inline]
    fn inputref(info: InputrefInfo) -> impl IntoView {
        <T as TermTrackedViews>::inputref(info)
    }

    #[inline]
    fn comp<V: IntoView + 'static>(then: impl FnOnce() -> V) -> impl IntoView {
        <T as TermTrackedViews>::comp(then)
    }
}
