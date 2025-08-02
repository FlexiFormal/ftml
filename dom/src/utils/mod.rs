pub mod actions;
pub mod css;
pub mod local_cache;

use leptos::wasm_bindgen::JsValue;

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
