use std::hint::unreachable_unchecked;

use ftml_dom::{
    ClonableView, DocumentState, TermTrackedViews,
    utils::{ContextChain, css::inject_css},
};
use ftml_ontology::terms::{ArgumentMode, VarOrSym};
use ftml_uris::SymbolUri;
use leptos::prelude::*;

use crate::{
    components::terms::{InBinder, InTerm},
    config::{FtmlConfig, HighlightStyle},
    utils::ReactiveStore,
};

pub const fn comp_class(is_hovered: bool, is_var: bool, style: HighlightStyle) -> &'static str {
    use HighlightStyle as HL;
    match (is_hovered, is_var, style) {
        (false, true, _) => "ftml-var-comp",
        (true, true, _) => "ftml-var-comp ftml-comp-hover",
        (false, false, HL::Colored | HL::None) => "ftml-comp",
        (false, false, HL::Subtle) => "ftml-comp-subtle",
        (true, false, HL::Subtle) => "ftml-comp-subtle ftml-comp-hover",
        (true, false, HL::Colored | HL::None) => "ftml-comp ftml-comp-hover",
        (false, _, HL::Off) => "ftml-comp-off",
        (true, _, HL::Off) => "ftml-comp-hover",
    }
}

pub fn comp_like<V: IntoView + 'static>(
    head: VarOrSym,
    is_hovered: Option<RwSignal<bool>>,
    top_term: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> AnyView {
    use thaw::{Popover, PopoverSize, PopoverTrigger};

    inject_css("ftml-comp", include_str!("comp.css"));
    let is_hovered = is_hovered.unwrap_or_else(|| RwSignal::new(false));
    let on_click = ReactiveStore::on_click(&head);
    let allow_formals = FtmlConfig::allow_formal_info();
    let top_class = Memo::new(move |_| {
        if is_hovered.get() {
            tracing::trace!("Hovering");
            "ftml-symbol-hover ftml-symbol-hover-hovered".to_string()
        } else {
            "ftml-symbol-hover ftml-symbol-hover-hidden".to_string()
        }
    });
    let style = FtmlConfig::highlight_style();
    let is_var = matches!(head, VarOrSym::Var(_));
    let class = Memo::new(move |_| comp_class(is_hovered.get(), is_var, style.get()).to_string());
    let top_term = if top_term && allow_formals {
        crate::Views::current_top_term()
    } else {
        None
    };
    let on_click = move |_| {
        on_click.click(allow_formals, top_term.clone());
    };
    view! {
        <Popover
            class=top_class
            size=PopoverSize::Small
            on_open=move || is_hovered.set(true)
            on_close=move || is_hovered.set(false)
            //on_click_signal=ocp
        >
            <PopoverTrigger slot>{
                children().attr("class",move || class)
                .add_any_attr(leptos::ev::on(
                    leptos::ev::click,
                    Box::new(on_click)
                ))
            }</PopoverTrigger>
            {super::popover::term_popover(head)}
        </Popover>
    }
    .into_any()
}

pub fn comp(children: ClonableView) -> AnyView {
    tracing::trace!("doing comp");
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return children.into_view::<crate::Views>();
    }
    let Some(head) = DocumentState::current_term_head() else {
        tracing::warn!("no current head");
        return children.into_view::<crate::Views>();
    };

    let is_var = matches!(&head, VarOrSym::Var(_));
    let Some(is_hovered) = use_context::<InTerm>().map(|h| h.hovered) else {
        tracing::warn!("InTerm is missing!");
        return children.into_view::<crate::Views>();
    };
    if is_var {
        let arg = DocumentState::arguments().next();
        tracing::trace!("variable comp: {head}@{arg:?}");
        if arg.is_some_and(|a| [ArgumentMode::Sequence, ArgumentMode::Simple].contains(&a.mode())) {
            let VarOrSym::Var(var) = head.clone() else {
                // SAFETY: is_var==true
                unsafe { unreachable_unchecked() }
            };
            let actual_signal = RwSignal::new(None);
            Effect::new(move || {
                if let Some(new_hovered) = ContextChain::<InBinder>::iter().find_map(|ctx| {
                    if ctx.vars.with(|v| v.contains(&var)) {
                        Some(ctx.hovered)
                    } else {
                        None
                    }
                }) {
                    tracing::trace!("{var}@{arg:?} inherits signal");
                    actual_signal.set(Some(new_hovered));
                }
            });
            let has_changed = RwSignal::new(false);
            Effect::new(move || {
                is_hovered.track();
                if let Some(sig) = actual_signal.get() {
                    if has_changed.get_untracked() {
                        has_changed.update_untracked(|b| *b = false);
                    } else {
                        has_changed.update_untracked(|b| *b = true);
                        is_hovered.set(sig.get());
                    }
                }
            });
            Effect::new(move || {
                is_hovered.track();
                if let Some(sig) = actual_signal.get() {
                    if has_changed.get_untracked() {
                        has_changed.update_untracked(|b| *b = false);
                    } else {
                        has_changed.update_untracked(|b| *b = true);
                        sig.set(is_hovered.get());
                    }
                }
            });
        }
    }

    comp_like(head, Some(is_hovered), true, move || {
        children.into_view::<crate::Views>()
    })
}

pub fn defcomp(uri: Option<SymbolUri>, children: ClonableView) -> AnyView {
    use HighlightStyle as HL;
    tracing::trace!("doing defcomp");
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return children.into_view::<crate::Views>();
    }
    inject_css("ftml-comp", include_str!("comp.css"));
    let is_hovered = use_context::<InTerm>().map(|h| h.hovered);
    let style = FtmlConfig::highlight_style();
    let class = Memo::new(
        move |_| match (is_hovered.is_some_and(|h| h.get()), style.get()) {
            (false, HL::Colored | HL::None) => "ftml-def-comp",
            (false, HL::Subtle) => "ftml-def-comp-subtle",
            (true, HL::Colored | HL::None) => "ftml-def-comp ftml-comp-hover",
            (true, HL::Subtle) => "ftml-def-comp-subtle ftml-comp-hover",
            (false, HL::Off) => "ftml-comp-off",
            (true, HL::Off) => "ftml-comp-hover",
        },
    );
    if let Some(uri) = uri {
        let on_click = ReactiveStore::on_click(&uri.into());
        let allow_formals = FtmlConfig::allow_formal_info();
        let on_click = move |_| {
            on_click.click(allow_formals, None);
        };
        children
            .into_view::<crate::Views>()
            .attr("class", move || class)
            .add_any_attr(leptos::ev::on(leptos::ev::click, Box::new(on_click)))
            .into_any()
    } else {
        children
            .into_view::<crate::Views>()
            .attr("class", move || class)
            .into_any()
    }
}
