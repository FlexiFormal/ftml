use crate::{
    ClonableView,
    markers::{InputrefInfo, ParagraphInfo, SectionInfo},
    terms::{ReactiveApplication, TopTerm},
};
use ftml_ontology::{
    narrative::elements::SectionLevel,
    terms::{VarOrSym, Variable},
};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};
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

    fn section<V: IntoView>(
        info: SectionInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
            <div id=info.id style=info.style class=info.class>
              {then()}
            </div>
        }
    }

    #[inline]
    fn paragraph<V: IntoView>(
        info: ParagraphInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
          <div class=info.class style=info.style>{then()}</div>
        }
    }

    #[inline]
    fn section_title<V: IntoView>(
        _lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
          <div class=class>{then()}</div>
        }
    }

    #[inline]
    fn paragraph_title<V: IntoView>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        view! {
          <div class="ftml-title-paragraph">{then()}</div>
        }
    }

    fn inputref(_info: InputrefInfo) -> impl IntoView {}

    #[inline]
    fn symbol_reference(
        _uri: SymbolUri,
        _notation: Option<Id>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn variable_reference(
        _var: Variable,
        _notation: Option<Id>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn application(
        _head: VarOrSym,
        _notation: Option<Id>,
        _uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn binder_application(
        _head: VarOrSym,
        _notation: Option<Id>,
        _uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn comp(_is_def: bool, then: ClonableView) -> impl IntoView {
        then.into_view::<Self>()
    }
}

pub trait TermTrackedViews: 'static {
    fn current_top_term() -> Option<DocumentElementUri> {
        use_context::<Option<TopTerm>>().and_then(|t| t.map(|t| t.uri))
    }

    #[inline]
    fn top<V: IntoView + 'static>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        super::global_setup(then)
    }

    fn section<V: IntoView>(
        info: SectionInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
            <div id=info.id style=info.style class=info.class>
              {then()}
            </div>
        }
    }

    #[inline]
    fn paragraph<V: IntoView>(
        info: ParagraphInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
          <div class=info.class style=info.style>{then()}</div>
        }
    }

    #[inline]
    fn section_title<V: IntoView>(
        _lvl: SectionLevel,
        class: &'static str,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
          <div class=class>{then()}</div>
        }
    }

    #[inline]
    fn paragraph_title<V: IntoView>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        view! {
          <div class="ftml-title-paragraph">{then()}</div>
        }
    }

    fn inputref(_info: InputrefInfo) -> impl IntoView {}

    #[inline]
    fn symbol_reference(
        _uri: SymbolUri,
        _notation: Option<Id>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    #[inline]
    fn variable_reference(
        _var: Variable,
        _notation: Option<Id>,
        _in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        then.into_view::<Self>()
    }

    fn application(
        head: ReadSignal<ReactiveApplication>,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView;

    fn binder_application(
        head: ReadSignal<ReactiveApplication>,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView;

    #[inline]
    fn comp(_is_def: bool, then: ClonableView) -> impl IntoView {
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
    fn paragraph<V: IntoView>(
        info: ParagraphInfo,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::paragraph(info, then)
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
    fn paragraph_title<V: IntoView>(then: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
        <T as TermTrackedViews>::paragraph_title(then)
    }

    #[inline]
    fn inputref(info: InputrefInfo) -> impl IntoView {
        <T as TermTrackedViews>::inputref(info)
    }

    #[inline]
    fn symbol_reference(
        uri: SymbolUri,
        notation: Option<Id>,
        in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        <T as TermTrackedViews>::symbol_reference(uri, notation, in_term, then)
    }

    #[inline]
    fn variable_reference(
        var: Variable,
        notation: Option<Id>,
        in_term: bool,
        then: ClonableView,
    ) -> impl IntoView {
        <T as TermTrackedViews>::variable_reference(var, notation, in_term, then)
    }

    #[inline]
    fn application(
        head: VarOrSym,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        ReactiveApplication::track(head, uri.clone(), move |a| {
            <T as TermTrackedViews>::application(a, notation, uri, then)
        })
    }

    #[inline]
    fn binder_application(
        head: VarOrSym,
        notation: Option<Id>,
        uri: Option<DocumentElementUri>,
        then: ClonableView,
    ) -> impl IntoView {
        ReactiveApplication::track(head, uri.clone(), move |a| {
            <T as TermTrackedViews>::binder_application(a, notation, uri, then)
        })
    }

    #[inline]
    fn comp(is_def: bool, then: ClonableView) -> impl IntoView {
        <T as TermTrackedViews>::comp(is_def, then)
    }
}
