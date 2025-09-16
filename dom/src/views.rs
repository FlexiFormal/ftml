use crate::{
    ClonableView,
    markers::{InputrefInfo, ParagraphInfo, SectionInfo},
    terms::{ReactiveApplication, TopTerm},
};
use ftml_ontology::{
    narrative::elements::{SectionLevel, problems::ChoiceBlockStyle},
    terms::{VarOrSym, Variable},
};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

pub trait FtmlViews: 'static {
    fn render_ftml(html: String, on_load: Option<RwSignal<bool>>) -> impl IntoView {
        use leptos_posthoc::{DomStringCont, DomStringContProps};
        DomStringCont(DomStringContProps {
            html,
            cont: super::iterate::<Self>,
            on_load,
            class: None::<String>.into(),
            style: None::<String>.into(),
        })
    }
    fn render_ftml_and_then(html: String, f: impl FnOnce() + 'static) -> impl IntoView {
        use leptos_posthoc::{DomStringCont, DomStringContProps};
        let sig = RwSignal::new(false);
        let do_once = std::sync::LazyLock::new(f);
        let _ = Effect::new(move || {
            if sig.get() {
                std::sync::LazyLock::force(&do_once);
            }
        });
        DomStringCont(DomStringContProps {
            html,
            cont: super::iterate::<Self>,
            on_load: Some(sig),
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
    fn cont(node: OriginalNode, skip_head: bool) -> impl IntoView {
        use leptos_posthoc::{DomCont, DomContProps};
        DomCont(DomContProps {
            orig: node,
            skip_head,
            class: None::<String>.into(),
            style: None::<String>.into(),
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
    fn proof_body(then: OriginalNode) -> impl IntoView {
        Self::cont(then, true)
    }

    #[inline]
    fn problem<V: IntoView>(
        _uri: DocumentElementUri,
        _styles: Box<[Id]>,
        style: Memo<String>,
        class: String,
        _is_subproblem: bool,
        _autogradable: bool,
        _points: Option<f32>,
        _minutes: Option<f32>,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
          <div class=class style=style>{then()}</div>
        }
    }

    #[inline]
    fn problem_solution() -> impl IntoView {}

    #[inline]
    fn problem_hint<V: IntoView + 'static>(
        _then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
    }

    #[inline]
    fn problem_ex_note<V: IntoView + 'static>(
        _then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
    }

    #[inline]
    fn multiple_choice_block<V: IntoView + 'static>(
        _style: ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn single_choice_block<V: IntoView + 'static>(
        _style: ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn problem_choice<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn problem_gnote() -> impl IntoView {}

    #[inline]
    fn fillinsol(_width: Option<f32>) -> impl IntoView {}

    #[inline]
    fn problem_title(then: OriginalNode) -> impl IntoView {
        then.attr("class", "ftml-title-paragraph")
    }

    #[inline]
    fn slide<V: IntoView>(
        _uri: DocumentElementUri,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn section_title(_lvl: SectionLevel, class: &'static str, then: OriginalNode) -> impl IntoView {
        then.attr("class", class)
    }

    #[inline]
    fn paragraph_title(then: OriginalNode) -> impl IntoView {
        then.attr("class", "ftml-title-paragraph").attr("style", "")
    }

    #[inline]
    fn slide_title(then: OriginalNode) -> impl IntoView {
        then.attr("class", "ftml-title-slide")
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
    fn proof_body(then: OriginalNode) -> impl IntoView {
        Self::cont(then, true)
    }

    #[inline]
    fn problem<V: IntoView>(
        _uri: DocumentElementUri,
        _styles: Box<[Id]>,
        style: Memo<String>,
        class: String,
        _is_subproblem: bool,
        _autogradable: bool,
        _points: Option<f32>,
        _minutes: Option<f32>,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        view! {
          <div class=class style=style>{then()}</div>
        }
    }

    #[inline]
    fn problem_solution() -> impl IntoView {}

    #[inline]
    fn problem_hint<V: IntoView + 'static>(
        _then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
    }

    #[inline]
    fn problem_ex_note<V: IntoView + 'static>(
        _then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
    }

    #[inline]
    fn problem_gnote() -> impl IntoView {}

    #[inline]
    fn multiple_choice_block<V: IntoView + 'static>(
        _style: ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn single_choice_block<V: IntoView + 'static>(
        _style: ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn problem_choice<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn fillinsol(_width: Option<f32>) -> impl IntoView {}

    #[inline]
    fn problem_title(then: OriginalNode) -> impl IntoView {
        then.attr("class", "ftml-title-paragraph")
    }

    #[inline]
    fn slide<V: IntoView>(
        _uri: DocumentElementUri,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        then()
    }

    #[inline]
    fn section_title(_lvl: SectionLevel, class: &'static str, then: OriginalNode) -> impl IntoView {
        then.attr("class", class)
    }

    #[inline]
    fn paragraph_title(then: OriginalNode) -> impl IntoView {
        then.attr("class", "ftml-title-paragraph").attr("style", "")
    }
    #[inline]
    fn slide_title(then: OriginalNode) -> impl IntoView {
        then.attr("class", "ftml-title-slide")
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
    fn proof_body(then: OriginalNode) -> impl IntoView {
        <T as TermTrackedViews>::proof_body(then)
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
        <T as TermTrackedViews>::problem(
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
        <T as TermTrackedViews>::problem_solution()
    }

    #[inline]
    fn problem_hint<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::problem_hint(then)
    }

    #[inline]
    fn problem_ex_note<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::problem_ex_note(then)
    }

    #[inline]
    fn problem_gnote() -> impl IntoView {
        <T as TermTrackedViews>::problem_gnote()
    }

    #[inline]
    fn multiple_choice_block<V: IntoView + 'static>(
        style: ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::multiple_choice_block(style, then)
    }

    #[inline]
    fn single_choice_block<V: IntoView + 'static>(
        style: ChoiceBlockStyle,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::single_choice_block(style, then)
    }

    #[inline]
    fn problem_choice<V: IntoView + 'static>(
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::problem_choice(then)
    }

    #[inline]
    fn fillinsol(width: Option<f32>) -> impl IntoView {
        <T as TermTrackedViews>::fillinsol(width)
    }

    #[inline]
    fn problem_title(then: OriginalNode) -> impl IntoView {
        <T as TermTrackedViews>::problem_title(then)
    }

    #[inline]
    fn slide<V: IntoView>(
        uri: DocumentElementUri,
        then: impl FnOnce() -> V + Send + 'static,
    ) -> impl IntoView {
        <T as TermTrackedViews>::slide(uri, then)
    }

    #[inline]
    fn section_title(lvl: SectionLevel, class: &'static str, then: OriginalNode) -> impl IntoView {
        <T as TermTrackedViews>::section_title(lvl, class, then)
    }

    #[inline]
    fn paragraph_title(then: OriginalNode) -> impl IntoView {
        <T as TermTrackedViews>::paragraph_title(then)
    }

    #[inline]
    fn slide_title(then: OriginalNode) -> impl IntoView {
        <T as TermTrackedViews>::slide_title(then)
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
