use crate::{
    ClonableView,
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
    fn section<V: IntoView>(
        _info: SectionInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn section_title<V: IntoView>(
        _lvl: SectionLevel,
        _class: &'static str,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    fn inputref(_info: InputrefInfo) -> impl IntoView {}

    #[inline]
    fn symbol_reference(
        _uri: SymbolUri,
        _notation: Option<UriName>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn variable_reference(
        _var: Variable,
        _notation: Option<UriName>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn application(
        _head: VarOrSym,
        _notation: Option<UriName>,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn binder_application(
        _head: VarOrSym,
        _notation: Option<UriName>,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn comp(then: ClonableView) -> impl IntoView {
        then.into_view::<Self>()
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
}

pub trait TermTrackedViews: 'static {
    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        super::global_setup(then)
    }

    #[inline]
    fn section<V: IntoView>(
        _info: SectionInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn section_title<V: IntoView>(
        _lvl: SectionLevel,
        _class: &'static str,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    fn inputref(_info: InputrefInfo) -> impl IntoView {}

    #[inline]
    fn symbol_reference(
        _uri: SymbolUri,
        _notation: Option<UriName>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn variable_reference(
        _var: Variable,
        _notation: Option<UriName>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    fn application(
        head: ReadSignal<ReactiveApplication>,
        notation: Option<UriName>,
        then: ClonableView,
    ) -> impl IntoView;

    fn binder_application(
        head: ReadSignal<ReactiveApplication>,
        notation: Option<UriName>,
        then: ClonableView,
    ) -> impl IntoView;

    #[inline]
    fn comp(then: ClonableView) -> impl IntoView {
        then.into_view::<Self>()
    }
}

impl<T: TermTrackedViews + ?Sized> FtmlViews for T {
    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        <T as TermTrackedViews>::top(then)
    }
    #[inline]
    fn section<V: IntoView>(
        info: SectionInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::section(info, then)
    }

    #[inline]
    fn section_title<V: IntoView>(
        lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::section_title(lvl, class, then)
    }

    #[inline]
    fn inputref(info: InputrefInfo) -> impl IntoView {
        <T as TermTrackedViews>::inputref(info)
    }

    #[inline]
    fn symbol_reference(
        uri: SymbolUri,
        notation: Option<UriName>,
        in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        <T as TermTrackedViews>::symbol_reference(uri, notation, in_term, then)
    }

    #[inline]
    fn variable_reference(
        var: Variable,
        notation: Option<UriName>,
        in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        <T as TermTrackedViews>::variable_reference(var, notation, in_term, then)
    }

    #[inline]
    fn application(head: VarOrSym, notation: Option<UriName>, then: ClonableView) -> impl IntoView {
        ReactiveApplication::track(head, move |a| {
            <T as TermTrackedViews>::application(a, notation, then)
        })
    }

    #[inline]
    fn binder_application(
        head: VarOrSym,
        notation: Option<UriName>,
        then: ClonableView,
    ) -> impl IntoView {
        ReactiveApplication::track(head, move |a| {
            <T as TermTrackedViews>::binder_application(a, notation, then)
        })
    }

    #[inline]
    fn comp(then: ClonableView) -> impl IntoView {
        <T as TermTrackedViews>::comp(then)
    }
}
