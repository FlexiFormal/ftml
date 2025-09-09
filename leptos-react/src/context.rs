use ftml_js_utils::conversion::ToJs;
use leptos::prelude::Owner;
use parking_lot::RwLock;
use std::{convert::Infallible, sync::Arc};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct LeptosMountHandle {
    mount: std::cell::Cell<
        Option<leptos::prelude::UnmountHandle<leptos::tachys::view::any_view::AnyViewState>>,
    >,
}

#[wasm_bindgen]
impl LeptosMountHandle {
    /// unmounts the view and cleans up the reactive system.
    /// Not calling this is a memory leak
    pub fn unmount(&self) -> Result<(), wasm_bindgen::JsError> {
        if let Some(mount) = self.mount.take() {
            drop(mount); //try_catch(move || drop(mount))?;
        }
        Ok(())
    }
}

impl LeptosMountHandle {
    pub fn new<V: leptos::prelude::IntoView + 'static>(
        div: leptos::web_sys::HtmlElement,
        f: impl FnOnce() -> V + 'static,
    ) -> Self {
        let handle =
            leptos::prelude::mount_to(div, move || leptos::prelude::IntoAny::into_any(f()));
        Self {
            mount: std::cell::Cell::new(Some(handle)),
        }
    }
}

/// Represents a leptos context; i.e. a node somewhere in the reactive graph
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct LeptosContext {
    inner: Arc<RwLock<Option<Owner>>>,
}
impl LeptosContext {
    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        if let Some(o) = (*self.inner.read()).clone() {
            o.with(f)
        } else {
            tracing::error!("Leptos context already cleaned up!");
            f()
        }
    }
}

#[wasm_bindgen]
impl LeptosContext {
    /// Cleans up the reactive system.
    pub fn cleanup(&self) -> Result<(), wasm_bindgen::JsError> {
        if let Some(mount) = self.inner.write().take() {
            mount.cleanup(); //flams_web_utils::try_catch(move || mount.cleanup())?;
        }
        Ok(())
    }

    pub fn wasm_clone(&self) -> Self {
        self.clone()
    }
}
impl From<Owner> for LeptosContext {
    #[inline]
    fn from(value: Owner) -> Self {
        Self {
            inner: std::sync::Arc::new(RwLock::new(Some(value))),
        }
    }
}
impl ToJs for LeptosContext {
    type Error = Infallible;
    fn to_js(&self) -> Result<wasm_bindgen::JsValue, Self::Error> {
        Ok(self.clone().into())
    }
}
