pub mod actions;
pub mod css;
pub mod local_cache;

use ftml_ontology::utils::Css;
use ftml_uris::ModuleUri;
use leptos::{
    IntoView,
    html::ElementChild,
    prelude::{
        AnyView, Effect, Get, GetUntracked, IntoAny, RwSignal, StyleAttribute, Suspend, Update,
        With, WithUntracked, expect_context, provide_context, use_context,
    },
};

use crate::utils::css::CssExt;

pub fn math<V: IntoView>(f: impl FnOnce() -> V) -> impl IntoView {
    static CSS: std::sync::LazyLock<Css> = std::sync::LazyLock::new(|| {
        Css::Link(
            "https://fonts.googleapis.com/css2?family=STIX+Two+Math"
                .to_string()
                .into_boxed_str(),
        )
    });
    CSS.clone().inject();
    leptos::math::math()
        .style("font-family:'STIX Two Math'")
        .child(f())
}

pub fn get_true_rect(elem: &leptos::web_sys::Element) -> leptos::web_sys::DomRect {
    let rect = elem.get_bounding_client_rect();
    let Some(window) = leptos::web_sys::window() else {
        return rect;
    };
    if let Ok(s) = window.scroll_x() {
        rect.set_x(rect.x() + s);
    }
    if let Ok(s) = window.scroll_y() {
        rect.set_y(rect.y() + s);
    }
    rect
}

/// ### Panics
pub fn owned<V: leptos::prelude::IntoView>(
    f: impl FnOnce() -> V,
) -> leptos::tachys::reactive_graph::OwnedView<V> {
    let owner = leptos::prelude::Owner::current()
        .expect("no current reactive Owner found")
        .child();
    let children = owner.with(f);
    leptos::tachys::reactive_graph::OwnedView::new_with_owner(children, owner)
}

#[derive(Debug, Clone)]
pub struct ContextChain<T: Send + Sync + Clone + 'static> {
    value: T,
    parent: Option<Box<Self>>,
}
impl<T: Send + Sync + Clone + 'static + std::fmt::Debug> ContextChain<T> {
    pub fn provide(value: T) {
        leptos::prelude::provide_context(Self {
            value,
            parent: leptos::prelude::use_context::<Self>().map(Box::new),
        });
    }

    pub fn get() -> Option<T> {
        leptos::prelude::use_context::<Self>().map(|v| v.value)
    }
    pub fn with<R>(f: impl FnOnce(&T) -> R) -> Option<R> {
        leptos::prelude::with_context::<Self, _>(|v| f(&v.value))
    }
    pub fn iter() -> impl Iterator<Item = T> {
        struct ChainIter<T: Send + Sync + Clone + 'static> {
            current: Option<ContextChain<T>>,
        }
        impl<T: Send + Sync + Clone + 'static> Iterator for ChainIter<T> {
            type Item = T;
            fn next(&mut self) -> Option<Self::Item> {
                if let Some(next) = self.current.take() {
                    self.current = next.parent.map(|v| *v);
                    Some(next.value)
                } else {
                    None
                }
            }
        }
        let slf = leptos::prelude::use_context::<Self>();
        ChainIter { current: slf }
    }
}

pub trait FutureExt {
    type T;
    fn into_view(self, f: impl FnOnce(Self::T) -> AnyView + Clone + Send + 'static) -> AnyView;
}
impl<T, Fut: std::future::Future<Output = T>, F: Fn() -> Fut + Clone + Send + 'static> FutureExt
    for F
{
    type T = T;
    fn into_view(self, f: impl FnOnce(T) -> AnyView + Clone + Send + 'static) -> AnyView {
        use leptos::prelude::{Suspense, view};
        view!(<Suspense fallback = || "…">{move || {
            let s = self.clone();
            let mut f = f.clone();
            let fut = send_wrapper::SendWrapper::new(async move {let ret = s().await;f(ret)});
            Suspend::new(fut)
        }}</Suspense>)
        .into_any()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ModuleContext(RwSignal<rustc_hash::FxHashSet<ModuleUri>>);
impl ModuleContext {
    pub(crate) fn barrier() {
        let map = use_context::<Self>()
            .map_or_else(rustc_hash::FxHashSet::default, |s| s.0.get_untracked());
        provide_context(Self(RwSignal::new(map)));
    }
    pub(crate) fn reset() {
        provide_context(Self(RwSignal::new(rustc_hash::FxHashSet::default())));
    }
    pub(crate) fn add(uri: ModuleUri) {
        if let Some(s) = use_context::<Self>() {
            s.0.update(|v| {
                v.insert(uri);
            });
        }
    }
    pub fn get_context() -> rustc_hash::FxHashSet<ModuleUri> {
        let mut ret = use_context::<Self>()
            .map_or_else(rustc_hash::FxHashSet::default, |s| s.0.get_untracked());
        if let Some(c) = use_context::<ContentModuleContext>() {
            c.0.with_untracked(|s| {
                for uri in s {
                    ret.insert(uri.clone());
                }
            });
        }
        ret
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ContentModuleContext(RwSignal<rustc_hash::FxHashSet<ModuleUri>>);
impl ContentModuleContext {
    pub fn add(uri: ModuleUri) {
        if let Some(s) = use_context::<Self>() {
            s.0.update(|v| {
                v.insert(uri);
            });
        }
    }
    pub fn make_new(new: Option<ModuleUri>) {
        let mut map = rustc_hash::FxHashSet::default();
        if let Some(new) = new {
            map.insert(new);
        }
        let inner = Self(RwSignal::new(map));
        if let Some(parent) = use_context::<Self>() {
            Effect::new(move || {
                let mut news = Vec::new();
                inner.0.with_untracked(|i| {
                    parent.0.with(|hs| {
                        for u in hs {
                            if !i.contains(u) {
                                news.push(u.clone());
                            }
                        }
                    });
                });
                if !news.is_empty() {
                    inner.0.update(|hs| {
                        for n in news {
                            hs.insert(n);
                        }
                    });
                }
            });
        }
        provide_context(inner);
    }
}
