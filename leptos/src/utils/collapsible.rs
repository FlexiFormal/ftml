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
    view! {<details open=move || expanded.get()>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            {header.map(|h| h())}
        }</summary>
        <div>{children()}</div>
    </details>}
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
    /*
    let spread = leptos::attr::open(move || {
        if expanded.get() {
            tracing::warn!("Setting to {}", expanded.get());
            Some(expanded.get().to_string())
        } else {
            tracing::warn!("Setting to None");
            None
        }
    });
    let spread = if expanded {
        leptos::either::Either::Left(view!(<{..} open="true"/>))
    } else {
        leptos::either::Either::Right(view!(<{..}/>))
    };
     */
    //let click = RwSignal::new(false);
    let rf: NodeRef<Details> = NodeRef::new();
    let _ = Effect::new(move || {
        let _ = expanded.get();
        if let Some(node) = rf.get() {
            node.set_open(!expanded.get());
        }
    });
    view! {<details node_ref = rf>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            {header.map(|h| h())}
        }</summary>
        <div>{move || if expanded.get() {
          Some(children())
        } else { None }}</div>
    </details>}
}
