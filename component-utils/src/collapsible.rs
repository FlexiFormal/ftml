#![allow(clippy::must_use_candidate)]

use leptos::{html::Details, prelude::*};

#[component]
pub fn Collapsible<Ch: IntoView + 'static>(
    #[prop(optional)] header: Option<super::Header>,
    children: TypedChildren<Ch>,
    #[prop(optional, into)] expanded: Option<RwSignal<bool>>,
) -> impl IntoView {
    collapsible(
        header.map(|h| move || (h.children)()),
        children.into_inner(),
        expanded,
    )
}

pub fn collapsible<H: IntoView, V: IntoView>(
    header: Option<impl FnOnce() -> H>,
    children: impl FnOnce() -> V,
    expanded: Option<RwSignal<bool>>,
) -> impl IntoView {
    let expanded = expanded.unwrap_or_else(|| RwSignal::new(false));
    view! {<div>
        <div on:click=move |_| expanded.update(|b| *b = !*b) style=move || if expanded.get() {"margin-left:15px;display:list-item;list-style-type:disclosure-open"} else {"margin-left:15px;display:list-item;list-style-type:disclosure-closed"}>{
            {header.map(|h| h())}
        }</div>
        <div style=move || if expanded.get() {""} else {"display:none;"}>{children()}</div>
    </div>}
}

#[component]
pub fn LazyCollapsible<Ch: IntoView + 'static>(
    #[prop(optional)] header: Option<super::Header>,
    children: TypedChildrenMut<Ch>,
) -> impl IntoView {
    lazy_collapsible(
        header.map(|h| move || (h.children)()),
        children.into_inner(),
    )
}

pub fn lazy_collapsible<H: IntoView, V: IntoView + 'static>(
    header: Option<impl FnOnce() -> H>,
    mut children: impl FnMut() -> V + Send + 'static,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    view! {<div>
        <div on:click=move |_| expanded.update(|b| *b = !*b)  style=move || if expanded.get() {"margin-left:15px;display:list-item;list-style-type:disclosure-open"} else {"margin-left:15px;display:list-item;list-style-type:disclosure-closed"}>{
            {header.map(|h| h())}
        }</div>
        <div style=move || if expanded.get() {""} else {"display:none;"}>{move || if expanded.get() {
          Some(children())
        } else { None }}</div>
    </div>}
}

pub fn fancy_collapsible<V: IntoView>(
    body: impl FnOnce() -> V,
    visible: RwSignal<bool>,
    class: &'static str,
    style: &'static str,
) -> impl IntoView {
    super::inject_css("ftml-collapsible", include_str!("collapsible.css"));
    let style = Memo::new(move |_| {
        if !style.is_empty() && visible.get() {
            Some(style)
        } else {
            None
        }
    });
    let class = Memo::new(move |_| {
        if visible.get() {
            if class.is_empty() {
                "ftml-collapsible--visible".to_string()
            } else {
                format!("ftml-collapsible--visible {class}")
            }
        } else {
            "ftml-collapsible--invisible".to_string()
        }
    });
    view!(<div class=class style=style>{body()}</div>)
}

pub fn collapse_marker(signal: RwSignal<bool>, moved: bool) -> impl IntoView {
    let style = if moved {
        "cursor:pointer;position:relative;bottom:0.65ex;left:-1.3ex;margin-right:-1.3ex;"
    } else {
        "cursor:pointer;width:0;"
    };
    move || {
        leptos::html::span()
            .child(if signal.get() { "▾ " } else { "▸ " })
            .style(style)
    }
}
