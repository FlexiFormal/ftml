pub mod actions;
pub mod css;
pub mod local_cache;

use leptos::{IntoView, prelude::Suspend, wasm_bindgen::JsValue};

#[derive(Debug)]
pub struct JsDisplay(pub JsValue);
impl std::fmt::Display for JsDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.0.as_string() {
            return f.write_str(&v);
        }
        if let Some(v) = self.0.as_f64() {
            return write!(f, "num {v}");
        }
        if let Some(v) = self.0.as_bool() {
            return write!(f, "boolean {v}");
        }
        if let Ok(js) = leptos::web_sys::js_sys::JSON::stringify(&self.0) {
            let s: String = js.into();
            return f.write_str(&s);
        }
        write!(f, "object {:?}", self.0)
    }
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
    parent: Option<Box<ContextChain<T>>>,
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
    fn into_view<V: IntoView + 'static>(
        self,
        f: impl FnOnce(Self::T) -> V + Clone + Send + 'static,
    ) -> impl IntoView;
}
impl<T, Fut: std::future::Future<Output = T> + Send, F: Fn() -> Fut + Clone + Send + 'static>
    FutureExt for F
{
    type T = T;
    fn into_view<V: IntoView + 'static>(
        self,
        f: impl FnOnce(T) -> V + Clone + Send + 'static,
    ) -> impl IntoView {
        use leptos::prelude::{Suspense, view};
        view!(<Suspense fallback = || "…">{move || {
            let s = self.clone();
            let mut f = f.clone();
            Suspend::new(async move {let ret = s().await;f(ret)})
        }}</Suspense>)
    }
}
