#![allow(clippy::must_use_candidate)]

use ftml_dom::{DocumentState, VarOrSym, utils::css::inject_css};
use ftml_uris::SymbolUri;
use leptos::{prelude::*, tachys::reactive_graph::OwnedView};

use crate::config::{FtmlConfigState, HighlightStyle};

macro_rules! provide {
    ($value:expr; $ret:expr) => {{
        let owner = Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let children = owner.with(move || {
            provide_context($value);
            $ret
        });
        OwnedView::new_with_owner(children, owner)
    }};
}

#[derive(Copy, Clone)]
struct InTerm {
    hovered: RwSignal<bool>,
}

pub fn symbol_reference<V: IntoView>(
    uri: SymbolUri,
    children: impl FnOnce() -> V,
) -> impl IntoView {
    provide!(
        InTerm {
            hovered: RwSignal::new(false)
        };
        children()
    )
}

pub fn oms<V: IntoView>(
    uri: SymbolUri,
    is_math: bool,
    children: impl FnOnce() -> V,
) -> impl IntoView {
    ftml_core::TODO!();
    ()
}

const fn comp_class(
    is_defi: bool,
    is_hovered: bool,
    is_var: bool,
    style: HighlightStyle,
) -> &'static str {
    use HighlightStyle as HL;
    match (is_defi, is_hovered, is_var, style) {
        (_, false, true, _) => "ftml-var-comp",
        (_, true, true, _) => "ftml-var-comp ftml-comp-hover",
        (true, false, _, HL::Colored | HL::None) => "ftml-def-comp",
        (true, false, _, HL::Subtle) => "ftml-def-comp-subtle",
        (true, true, _, HL::Colored | HL::None) => "ftml-def-comp ftml-comp-hover",
        (true, true, _, HL::Subtle) => "ftml-def-comp-subtle ftml-comp-hover",
        (_, false, false, HL::Colored | HL::None) => "ftml-comp",
        (_, false, false, HL::Subtle) => "ftml-comp-subtle",
        (_, true, false, HL::Subtle) => "ftml-comp-subtle ftml-comp-hover",
        (_, true, false, HL::Colored | HL::None) => "ftml-comp ftml-comp-hover",
        (_, false, _, HL::Off) => "",
        (_, true, _, HL::Off) => "ftml-comp-hover",
    }
}

pub fn comp<V: IntoView + 'static>(children: impl FnOnce() -> V) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::{Popover, PopoverSize, PopoverTrigger};
    if !FtmlConfigState::allow_hovers() {
        return Left(children());
    }
    let Some(head) = DocumentState::current_term_head() else {
        return Left(children());
    };

    inject_css("ftml-comp", include_str!("comp.css"));

    let is_var = matches!(&head, VarOrSym::V(_));
    let is_hovered = expect_context::<InTerm>().hovered;
    let style = FtmlConfigState::highlight_style();
    let class = Memo::new(move |_| comp_class(false, is_hovered.get(), is_var, style.get()));
    let top_class = Memo::new(move |_| {
        if is_hovered.get() {
            tracing::trace!("Hovering");
            "ftml-symbol-hover ftml-symbol-hover-hovered".to_string()
        } else {
            "ftml-symbol-hover ftml-symbol-hover-hidden".to_string()
        }
    });
    //let ocp = expect_context::<crate::config::FTMLConfig>().get_on_click(&s);
    //let none: Option<FragmentContinuation> = None;
    let children = children();
    Right(view! {
        <Popover
            class=top_class
            size=PopoverSize::Small
            on_open=move || is_hovered.set(true)
            on_close=move || is_hovered.set(false)
            //on_click_signal=ocp
        >
            <PopoverTrigger slot>{
            children.add_any_attr(leptos::tachys::html::class::class(move || class))
            }</PopoverTrigger>
            <SymbolPopover head/>
        </Popover>
    })
}

#[component]
pub fn SymbolPopover(head: VarOrSym) -> impl IntoView {
    /*
    match head {
        VarOrSym::V(v) => EitherOf3::A(do_var_hover(v)),
        VarOrSym::S(DomainUri::Symbol(s)) => {
            EitherOf3::B(crate::remote::get!(definition(s.clone()) = (css,s) => {
              for c in css { do_css(c); }
              Some(view!(
                <div style="color:black;background-color:white;padding:3px;max-width:600px;">
                  <FTMLString html=s/>
                </div>
              ))
            }))
        }
        VarOrSym::S(DomainUri::Module(m)) => {
            EitherOf3::C(view! {<div>"Module" {m.module_name().last().to_string()}</div>})
        }
    }
     */
}
