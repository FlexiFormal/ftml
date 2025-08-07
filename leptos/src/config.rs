use ftml_dom::{DocumentState, counters::LogicalLevel, utils::JsDisplay};
use ftml_ontology::narrative::elements::{SectionLevel, paragraphs::ParagraphKind};
use ftml_uris::{DocumentElementUri, DocumentUri, LeafUri, NarrativeUri};
use leptos::context::Provider;
use leptos::prelude::*;

use crate::utils::ReactiveStore;

#[cfg(feature = "callbacks")]
use crate::callbacks::{OnSectionTitle, SectionWrap};

#[derive(Clone, Default)]
#[cfg_attr(feature = "csr", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FtmlConfig {
    #[cfg_attr(feature = "csr", serde(default, rename = "allowHovers"))]
    pub allow_hovers: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "allowFormalInfo"))]
    pub allow_formals: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "allowNotationChanges"))]
    pub allow_notation_changes: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "documentUri"))]
    pub document_uri: Option<DocumentUri>,

    #[cfg_attr(feature = "csr", serde(default, rename = "highlightStyle"))]
    pub highlight_style: Option<HighlightStyle>,

    #[cfg_attr(feature = "csr", serde(default, rename = "autoexpandLimit"))]
    pub autoexpand_limit: Option<LogicalLevel>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub section_wrap: Option<SectionWrap>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub on_section_title: Option<OnSectionTitle>,
}

#[cfg_attr(feature = "csr", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HighlightStyle {
    Colored,
    Subtle,
    Off,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AllowHovers(pub bool);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AllowFormals(pub bool);

#[derive(Copy, Clone)]
pub struct AutoexpandLimit(pub LogicalLevel);

#[derive(Copy, Clone)]
pub struct AllowNotationChanges(bool);

#[derive(thiserror::Error, Debug)]
pub enum FtmlConfigParseError {
    #[error("not a javascript object")]
    NotAnObject(JsDisplay),
    #[error("invalid value for {0}: {1}")]
    InvalidValue(&'static str, JsDisplay),
    #[error("invalid URI in {0}: {1}")]
    InvalidUri(&'static str, ftml_uris::errors::UriParseError),
    #[cfg(feature = "callbacks")]
    #[error("invalid javascript function in {0}: {1}")]
    InvalidFun(&'static str, leptos_react::functions::NotAJsFunction),
}

#[cfg(feature = "csr")]
impl leptos::wasm_bindgen::convert::TryFromJsValue for FtmlConfig {
    type Error = (Self, Vec<FtmlConfigParseError>);
    fn try_from_js_value(value: leptos::wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        macro_rules! fields {
            ($($stat:ident = $name:literal),* $(,)?) => {
                std::thread_local! {$(
                    static $stat : std::cell::LazyCell<leptos::wasm_bindgen::JsValue> =std::cell::LazyCell::new(|| leptos::wasm_bindgen::JsValue::from($name));
                )*}
            }
        }
        if !value.is_object() {
            return Err((
                Self::default(),
                vec![FtmlConfigParseError::NotAnObject(JsDisplay(value))],
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
                        JsDisplay($e),
                    )),
                }
            };
        }

        get!(v @ "allowHovers" = ALLOW_HOVERS as_bool { config.allow_hovers = Some(v)});
        get!(v @ "allowFormalInfo" = ALLOW_FORMAL_INFO as_bool { config.allow_formals = Some(v)});
        get!(v @ "allowNotationChanges" = ALLOW_NOTATION_CHANGES as_bool { config.allow_notation_changes = Some(v)} );
        get!(v @ "documentUri" = DOCUMENT_URI as_string {
            match v.parse() {
                Ok(url) => config.document_uri = Some(url),
                Err(e) => errors.push(FtmlConfigParseError::InvalidUri("documentUri", e))
            }
        });
        get!(v @ "highlightStyle" = HIGHLIGHT_STYLE ?
            j => HighlightStyle::try_from_js_value(j);
            e => FtmlConfigParseError::InvalidValue("highlightStyle", JsDisplay(e));
            {config.highlight_style = Some(v)}
        );
        get!(v @ "autoexpandLimit" = AUTOEXPAND_LIMIT ?
            j => LogicalLevel::try_from_js_value(j);
            e => FtmlConfigParseError::InvalidValue("autoexpandLimit", JsDisplay(e));
            {config.autoexpand_limit = Some(v)}
        );
        #[cfg(feature = "callbacks")]
        get!(v @ "sectionWrap" = SECTION_WRAP ?F { config.section_wrap = Some(v) });
        #[cfg(feature = "callbacks")]
        get!(v @ "onSectionTitle" = ON_SECTION_TITLE ?F { config.on_section_title = Some(v) });
        /*
        #[cfg_attr(feature = "csr", serde(rename = "autoexpandLimit"))]
        pub autoexpand_limit: Option<LogicalLevel>,

        */
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
        if let Some(b) = self.allow_formals {
            provide_context(AllowFormals(b));
        }
        if let Some(b) = self.allow_notation_changes {
            provide_context(AllowNotationChanges(b));
        }
        if let Some(h) = self.highlight_style {
            let style = RwSignal::new(h);
            provide_context(style);
            provide_context(style.read_only());
        }
        if let Some(limit) = self.autoexpand_limit {
            let sig = RwSignal::new(AutoexpandLimit(limit));
            provide_context(sig);
            provide_context(sig.read_only());
        }
        #[cfg(feature = "callbacks")]
        if let Some(b) = self.section_wrap {
            provide_context(Some(b));
        }
        #[cfg(feature = "callbacks")]
        if let Some(b) = self.on_section_title {
            provide_context(Some(b));
        }
        self.document_uri
    }

    pub fn init() {
        if with_context::<RwSignal<HighlightStyle>, _>(|_| ()).is_none() {
            #[cfg(not(any(feature = "csr", feature = "hydrate")))]
            let style = RwSignal::new(HighlightStyle::Colored);
            #[cfg(any(feature = "csr", feature = "hydrate"))]
            let style = {
                let r =
                    <gloo_storage::LocalStorage as gloo_storage::Storage>::get("highlight_option")
                        .map_or(HighlightStyle::Colored, |e| e);
                let r = RwSignal::new(r);
                Effect::new(move || {
                    let r = r.get();
                    let _ = <gloo_storage::LocalStorage as gloo_storage::Storage>::set(
                        "highlight_option",
                        r,
                    );
                });
                r
            };
            provide_context(style);
            provide_context(style.read_only());
        }
        if with_context::<RwSignal<AutoexpandLimit>, _>(|_| ()).is_none() {
            let sig = RwSignal::new(AutoexpandLimit(LogicalLevel::Section(
                SectionLevel::Section,
            )));
            provide_context(sig);
            provide_context(sig.read_only());
        }
        if with_context::<StoredValue<ReactiveStore>, _>(|_| ()).is_none() {
            provide_context(StoredValue::new(ReactiveStore::new()));
        }
    }
}

pub struct FtmlConfigState;
impl FtmlConfigState {
    #[inline]
    #[must_use]
    pub fn allow_hovers() -> bool {
        use_context::<AllowHovers>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn allow_formal_info() -> bool {
        use_context::<AllowFormals>().is_none_or(|b| b.0)
    }

    /// ### Panics
    pub fn disable_hovers<V: IntoView>(f: impl FnOnce() -> V) -> impl IntoView {
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let children = owner.with(move || {
            provide_context(AllowHovers(false));
            f()
        });
        leptos::tachys::reactive_graph::OwnedView::new_with_owner(children, owner)
    }

    /// ### Panics
    #[must_use]
    pub fn notation_preference(uri: &LeafUri) -> ReadSignal<Option<DocumentElementUri>> {
        let sig = Self::notation_preference_signal(uri);
        with_context::<StoredValue<ReactiveStore>, _>(|s| {
            s.with_value(|s| s.with(|| sig.read_only()))
        })
        .expect("Not in an ftml context")
    }

    pub(crate) fn notation_preference_signal(
        uri: &LeafUri,
    ) -> RwSignal<Option<DocumentElementUri>> {
        with_context::<StoredValue<ReactiveStore>, _>(|s| {
            if let Some(v) = s.with_value(|store| store.notations.get(uri).copied()) {
                return v;
            }
            let value = {
                #[cfg(any(feature = "csr", feature = "hydrate"))]
                {
                    use gloo_storage::Storage;
                    gloo_storage::LocalStorage::get(format!("notation_{uri}")).ok()
                }
                #[cfg(not(any(feature = "csr", feature = "hydrate")))]
                {
                    None
                }
            };
            let ret = s.with_value(move |store| {
                store.with(move || {
                    let r = RwSignal::new(value);
                    #[cfg(any(feature = "csr", feature = "hydrate"))]
                    {
                        let uri = uri.clone();
                        Effect::new(move || {
                            r.with(|s| {
                                use gloo_storage::Storage;
                                if let Some(s) = s.as_ref() {
                                    let _ = gloo_storage::LocalStorage::set(
                                        format!("notation_{uri}"),
                                        s,
                                    );
                                } else {
                                    gloo_storage::LocalStorage::delete(format!("notation_{uri}"));
                                }
                            });
                        });
                    }
                    r
                })
            });
            s.update_value(|s| {
                s.notations.insert(uri.clone(), ret);
            });
            ret
        })
        .expect("Not in an ftml context")
    }

    #[inline]
    #[must_use]
    pub fn allow_notation_changes() -> bool {
        use_context::<AllowNotationChanges>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn highlight_style() -> ReadSignal<HighlightStyle> {
        expect_context()
    }

    #[inline]
    #[must_use]
    pub fn autoexpand_limit() -> ReadSignal<AutoexpandLimit> {
        expect_context()
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
        #[cfg(not(feature = "callbacks"))]
        {
            children()
        }
        #[cfg(feature = "callbacks")]
        {
            if let Some(Some(w)) = use_context::<Option<SectionWrap>>() {
                Left(w.wrap(uri, children))
            } else {
                Right(children())
            }
        }
    }

    pub fn wrap_paragraph<V: IntoView, F: FnOnce() -> V>(
        uri: &DocumentElementUri,
        kind: ParagraphKind,
        children: F,
    ) -> impl IntoView + use<V, F> {
        use leptos::either::Either::{Left, Right};
        #[cfg(not(feature = "callbacks"))]
        {
            children()
        }
        #[cfg(feature = "callbacks")]
        {
            use crate::callbacks::ParagraphWrap;

            if let Some(Some(w)) = use_context::<Option<ParagraphWrap>>() {
                Left(w.wrap(uri, &kind, children))
            } else {
                Right(children())
            }
        }
    }

    pub fn insert_section_title(lvl: SectionLevel) -> impl IntoView + use<> {
        #[cfg(not(feature = "callbacks"))]
        {
            None::<&'static str>
        }
        #[cfg(feature = "callbacks")]
        {
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
}
