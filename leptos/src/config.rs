use ftml_dom::DocumentState;
use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::{DocumentElementUri, DocumentUri, NarrativeUri};
use leptos::context::Provider;
use leptos::prelude::*;

use crate::callbacks::{OnSectionTitle, SectionWrap};

#[derive(Clone, Default)]
#[cfg_attr(feature = "csr", derive(serde::Serialize, serde::Deserialize))]
//#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FtmlConfig {
    #[cfg_attr(feature = "csr", serde(default))]
    #[cfg_attr(feature = "csr", serde(rename = "allowHovers"))]
    pub allow_hovers: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default))]
    #[cfg_attr(feature = "csr", serde(rename = "documentUri"))]
    pub document_uri: Option<DocumentUri>,

    #[cfg_attr(feature = "csr", serde(default))]
    #[cfg_attr(feature = "csr", serde(rename = "highlightStyle"))]
    pub highlight_style: Option<HighlightStyle>,

    #[cfg_attr(feature = "csr", serde(skip))]
    pub section_wrap: Option<SectionWrap>,

    #[cfg_attr(feature = "csr", serde(skip))]
    pub on_section_title: Option<OnSectionTitle>,
}

#[cfg(feature = "typescript")]
#[derive(thiserror::Error, Debug)]
pub enum FtmlConfigParseError {
    #[error("not a javascript object")]
    NotAnObject(leptos_react::utils::JsDisplay),
    #[error("invalid value for {0}: {1}")]
    InvalidValue(&'static str, leptos_react::utils::JsDisplay),
    #[error("invalid URI in {0}: {1}")]
    InvalidUri(&'static str, ftml_uris::errors::UriParseError),
    #[error("invalid javascript function in {0}: {1}")]
    InvalidFun(&'static str, leptos_react::functions::NotAJsFunction),
}

#[cfg(feature = "typescript")]
impl wasm_bindgen::convert::TryFromJsValue for FtmlConfig {
    type Error = (Self, Vec<FtmlConfigParseError>);
    fn try_from_js_value(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        macro_rules! fields {
            ($($stat:ident = $name:literal),* $(,)?) => {
                std::thread_local! {$(
                    static $stat : std::cell::LazyCell<wasm_bindgen::JsValue> =std::cell::LazyCell::new(|| wasm_bindgen::JsValue::from($name));
                )*}
            }
        }
        if !value.is_object() {
            return Err((
                Self::default(),
                vec![FtmlConfigParseError::NotAnObject(
                    leptos_react::utils::JsDisplay(value),
                )],
            ));
        }
        let mut config = Self::default();
        let mut errors = Vec::new();

        macro_rules! get {
            ($v:ident @ $name:literal = $id:ident $ast:ident $b:block) => {
                get!($v @ $name = $id v => v.$ast(); $b)
            };
            ($v:ident@ $name:literal = $id:ident $i:ident => $f:expr; $b:block) => {
                fields! {
                    $id = $name
                }
                if let Ok($i) = $id.with(|s| leptos::web_sys::js_sys::Reflect::get(&value, s)) {
                    get!(@opt $name $i $f; $v $b)
                }
            };
            ($v:ident@ $name:literal = $id:ident ?F $b:block) => {
                fields! {
                    $id = $name
                }
                if let Ok(v) = $id.with(|s| leptos::web_sys::js_sys::Reflect::get(&value, s)) {
                    match  leptos_react::functions::JsRet::from_js(v) {
                        Ok($v) => $b,
                        Err(e) => errors.push(FtmlConfigParseError::InvalidFun(
                            $name,
                            e,
                        )),
                    }
                }
            };
            ($v:ident@ $name:literal = $id:ident ? $i:ident => $r:expr; $e:ident => $err:expr; $b:block) => {
                fields! {
                    $id = $name
                }
                if let Ok($i) = $id.with(|s| leptos::web_sys::js_sys::Reflect::get(&value, s)) {
                    match $r {
                        Ok($v) => $b,
                        Err($e) => errors.push($err),
                    }
                }
            };
            (@opt $name:literal $e:ident $f:expr; $v:ident $b:block) => {
                match $f {
                    Some($v) => $b,
                    None => errors.push(FtmlConfigParseError::InvalidValue(
                        $name,
                        leptos_react::utils::JsDisplay($e),
                    )),
                }
            };
        }

        get!(v @ "allowHovers" = ALLOW_HOVERS as_bool { config.allow_hovers = Some(v)});
        get!(v @ "documentUri" = DOCUMENT_URI as_string {
            match v.parse() {
                Ok(url) => config.document_uri = Some(url),
                Err(e) => errors.push(FtmlConfigParseError::InvalidUri("documentUri", e))
            }
        });
        get!(v @ "highlightStyle" = HIGHLIGHT_STYLE ?
            j => HighlightStyle::try_from_js_value(j);
            e => FtmlConfigParseError::InvalidValue("highlightStyle", leptos_react::utils::JsDisplay(e));
            {config.highlight_style = Some(v)}
        );
        get!(v @ "sectionWrap" = SECTION_WRAP ?F { config.section_wrap = Some(v) });
        get!(v @ "onSectionTitle" = ON_SECTION_TITLE ?F { config.on_section_title = Some(v) });
        // more

        if errors.is_empty() {
            Ok(config)
        } else {
            Err((config, errors))
        }
    }
}

impl FtmlConfig {
    #[must_use]
    pub fn apply(self) -> Option<DocumentUri> {
        if let Some(b) = self.allow_hovers {
            provide_context(AllowHovers(b));
        }
        if let Some(b) = self.section_wrap {
            provide_context(Some(b));
        }
        if let Some(b) = self.on_section_title {
            provide_context(Some(b));
        }
        if let Some(h) = self.highlight_style {
            tracing::info!("initializing highlight style from config");
            provide_context(RwSignal::new(h));
        }
        self.document_uri
    }
}

#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HighlightStyle {
    Colored,
    Subtle,
    Off,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct AllowHovers(pub bool);

pub struct FtmlConfigState;
impl FtmlConfigState {
    #[inline]
    #[must_use]
    pub fn allow_hovers() -> bool {
        use_context::<AllowHovers>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn highlight_style() -> ReadSignal<HighlightStyle> {
        expect_context::<RwSignal<HighlightStyle>>().read_only()
    }

    #[inline]
    #[must_use]
    pub fn with_allow_hovers<V: IntoView + 'static>(
        value: bool,
        children: TypedChildren<V>,
    ) -> impl IntoView {
        Provider(leptos::context::ProviderProps {
            value: AllowHovers(value),
            children,
        })
    }

    pub fn wrap_section<V: IntoView, F: FnOnce() -> V>(
        uri: &DocumentElementUri,
        children: F,
    ) -> impl IntoView + use<V, F> {
        use leptos::either::Either::{Left, Right};
        if let Some(Some(w)) = use_context::<Option<SectionWrap>>() {
            Left(w.wrap(uri, children))
        } else {
            Right(children())
        }
    }

    pub fn insert_section_title(lvl: SectionLevel) -> impl IntoView + use<> {
        if let Some(Some(w)) = use_context::<Option<OnSectionTitle>>() {
            let NarrativeUri::Element(uri) = DocumentState::current_uri() else {
                tracing::error!("Could not determine URI for current section");
                return None;
            };
            Some(w.insert(&uri, &lvl))
        } else {
            None
        }
    }
}
