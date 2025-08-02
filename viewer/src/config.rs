#[cfg(not(feature = "typescript"))]
use ftml_leptos::config::FtmlConfigParseError;
use ftml_uris::DocumentUri;
#[cfg(not(feature = "typescript"))]
use wasm_bindgen::{JsCast, JsValue, convert::TryFromJsValue};

#[cfg(not(feature = "typescript"))]
pub(crate) fn parse_config() -> (FtmlViewerConfig, Vec<FtmlConfigParseError>) {
    let window = leptos::tachys::dom::window();
    window
        .get("FTML_CONFIG")
        .map_or_else(|| (FtmlViewerConfig::default(), Vec::new()), parse)
}

#[cfg(not(feature = "typescript"))]
fn parse(cfg: leptos::web_sys::js_sys::Object) -> (FtmlViewerConfig, Vec<FtmlConfigParseError>) {
    let Ok(cfg): Result<JsValue, _> = cfg.dyn_into() else {
        return (FtmlViewerConfig::default(), Vec::new());
    };
    let (r, v) = match ftml_leptos::config::FtmlConfig::try_from_js_value(cfg.clone()) {
        Ok(r) => (r, Vec::new()),
        Err((r, v)) => (r, v),
    };
    let mut c: FtmlViewerConfig = r.into();
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("backendUrl")) {
        if let Some(s) = v.as_string() {
            c.backend_url = Some(s.into_boxed_str());
        }
    }
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("redirects")) {
        if let Ok(s) = leptos::web_sys::js_sys::JSON::stringify(&v) {
            if let Some(s) = s.as_string() {
                if let Ok(v) = serde_json::from_str(&s) {
                    c.redirects = v;
                }
            }
        }
    }

    (c, v)
}

/*
//#[cfg(not(feature = "typescript"))]
pub(crate) fn parse_config() -> (FtmlViewerConfig, Vec<FtmlConfigParseError>) {
    if let Ok(scripts) = leptos::tachys::dom::document().query_selector_all("head script") {
        let mut i = 0;
        while let Some(node) = scripts.get(i) {
            use leptos::wasm_bindgen::JsCast;
            i += 1;
            let Ok(elem) = node.dyn_into::<leptos::web_sys::Element>() else {
                continue;
            };
            if elem.get_attribute("src").is_none()
                && elem
                    .get_attribute("type")
                    .is_some_and(|s| s == "text/javascript")
                && elem.get_attribute("id").is_some_and(|s| s == "ftml")
            {
                return parse(elem);
            }
        }
    }
    (FtmlViewerConfig::default(), Vec::new())
}

//#[cfg(not(feature = "typescript"))]
fn parse(script_elem: leptos::web_sys::Element) -> (FtmlViewerConfig, Vec<FtmlConfigParseError>) {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let window = leptos::tachys::dom::window();

        leptos::web_sys::js_sys::Reflect::set(
            &JsValue::from(window),
            &JsValue::from("getFtmlConfig"),
            &JsValue::from(leptos::web_sys::js_sys::Function::new_with_args(
                "element",
                include_str!("getFtmlConfig.js"),
            )),
        )
        .expect("error defining js function");
    });
    let func: leptos::web_sys::js_sys::Function = leptos::tachys::dom::window()
        .get("getFtmlConfig")
        .expect("error getting Window property")
        .dyn_into()
        .expect("Window.getFtmlConfig is not a function");
    let cfg = func
        .call1(&JsValue::NULL, &JsValue::from(script_elem))
        .expect("error calling getFtmlConfig");
    let (r, v) = match ftml_leptos::config::FtmlConfig::try_from_js_value(cfg.clone()) {
        Ok(r) => (r, Vec::new()),
        Err((r, v)) => (r, v),
    };
    let mut c: FtmlViewerConfig = r.into();
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("backendUrl")) {
        if let Some(s) = v.as_string() {
            c.backend_url = Some(s.into_boxed_str());
        }
    }
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("redirects")) {
        if let Ok(s) = leptos::web_sys::js_sys::JSON::stringify(&v) {
            if let Some(s) = s.as_string() {
                if let Ok(v) = serde_json::from_str(&s) {
                    c.redirects = v;
                }
            }
        }
    }

    (c, v)
}
 */

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FtmlViewerConfig {
    #[serde(flatten)]
    pub inner: ftml_leptos::config::FtmlConfig,
    #[serde(default)]
    pub redirects: Vec<(DocumentUri, Box<str>)>,
    #[serde(default, rename = "backendUrl")]
    pub backend_url: Option<Box<str>>,
}
impl FtmlViewerConfig {
    #[must_use]
    pub fn apply(self) -> Option<DocumentUri> {
        if !self.redirects.is_empty() {
            let mut rs = super::backend::REDIRECTS.write();
            rs.extend_from_slice(&self.redirects);
        }
        if let Some(url) = self.backend_url {
            let mut be = super::backend::BACKEND_URL.write();
            *be = url;
        }
        self.inner.apply()
    }
}
impl From<ftml_leptos::config::FtmlConfig> for FtmlViewerConfig {
    fn from(value: ftml_leptos::config::FtmlConfig) -> Self {
        Self {
            inner: value,
            redirects: Vec::new(),
            backend_url: None,
        }
    }
}
