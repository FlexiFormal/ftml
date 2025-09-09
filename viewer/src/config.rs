use ftml_components::config::{FtmlConfigParseError, FtmlConfigParseErrors};
use ftml_uris::DocumentUri;
use wasm_bindgen::JsValue;

pub fn parse_config() -> (FtmlViewerConfig, Vec<FtmlConfigParseError>) {
    let global = leptos::web_sys::js_sys::global();
    leptos::web_sys::js_sys::Reflect::get(&global, &JsValue::from_str("FTML_CONFIG"))
        .map_or_else(|_| (FtmlViewerConfig::default(), Vec::new()), parse)
}

#[allow(clippy::needless_pass_by_value)]
fn parse(cfg: JsValue) -> (FtmlViewerConfig, Vec<FtmlConfigParseError>) {
    use ftml_js_utils::conversion::FromJs;
    let (r, v) = match ftml_components::config::FtmlConfig::from_js(cfg.clone()) {
        Ok(r) => (r, Vec::new()),
        Err(FtmlConfigParseErrors { config, errors }) => (config, errors),
    };
    let mut c: FtmlViewerConfig = r.into();
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("backendUrl"))
        && let Some(s) = v.as_string()
    {
        c.backend_url = Some(s.into_boxed_str());
    }
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("redirects"))
        && let Ok(s) = leptos::web_sys::js_sys::JSON::stringify(&v)
        && let Some(s) = s.as_string()
        && let Ok(v) = serde_json::from_str(&s)
    {
        c.redirects = v;
    }
    if let Ok(v) = leptos::web_sys::js_sys::Reflect::get(&cfg, &JsValue::from_str("logLevel"))
        && let Some(s) = v.as_string()
    {
        c.log_level = match &*s.to_ascii_uppercase() {
            "TRACE" => LogLevel::TRACE,
            "DEBUG" => LogLevel::DEBUG,
            "INFO" => LogLevel::INFO,
            "ERROR" => LogLevel::ERROR,
            _ => LogLevel::WARN,
        };
    }

    (c, v)
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    #[default]
    WARN,
    ERROR,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::TRACE => Self::TRACE,
            LogLevel::DEBUG => Self::DEBUG,
            LogLevel::INFO => Self::INFO,
            LogLevel::WARN => Self::WARN,
            LogLevel::ERROR => Self::ERROR,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FtmlViewerConfig {
    #[serde(flatten)]
    pub inner: ftml_components::config::FtmlConfig,
    #[serde(default)]
    pub redirects: Vec<(DocumentUri, Box<str>)>,
    #[serde(default, rename = "backendUrl")]
    pub backend_url: Option<Box<str>>,
    #[serde(default, rename = "logLevel")]
    pub log_level: LogLevel,
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
impl From<ftml_components::config::FtmlConfig> for FtmlViewerConfig {
    fn from(value: ftml_components::config::FtmlConfig) -> Self {
        Self {
            inner: value,
            redirects: Vec::new(),
            backend_url: None,
            log_level: LogLevel::default(),
        }
    }
}
