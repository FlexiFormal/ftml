use ftml_dom::{counters::LogicalLevel, toc::TocSource};
use ftml_js_utils::JsDisplay;
use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::{DocumentElementUri, DocumentUri, LeafUri};
use leptos::context::Provider;
use leptos::prelude::*;

use crate::utils::ReactiveStore;

#[cfg(feature = "callbacks")]
use crate::callbacks::{OnSectionTitle, ParagraphWrap, ProblemWrap, SectionWrap, SlideWrap};

#[cfg(feature = "typescript")]
#[leptos::wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const FTML_CONFIG: &str = r#"
export interface FtmlConfig {
    allowHovers?:boolean;
    allowFullscreen?:boolean;
    allowFormalInfo?:boolean;
    allowNotationChanges?:boolean;
    chooseHighlightStyle?:boolean;
    showContent?:boolean;
    pdfLink?:boolean;
    documentUri?:DocumentUri;
    highlightStyle?:HighlightStyle;
    toc?:TocSource;
    autoexpandLimit?:LogicalLevel;
    sectionWrap?:SectionWrap;
    paragraphWrap?:ParagraphWrap;
    slideWrap?:SlideWrap;
    problemWrap?:ProblemWrap;
    onSectionTitle?:OnSectionTitle;
}
"#;

#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "csr", derive(serde::Serialize, serde::Deserialize))]
//#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
//#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FtmlConfig {
    #[cfg_attr(feature = "csr", serde(default, rename = "allowHovers"))]
    pub allow_hovers: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "allowFullscreen"))]
    pub allow_fullscreen: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "allowFormalInfo"))]
    pub allow_formals: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "allowNotationChanges"))]
    pub allow_notation_changes: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "chooseHighlightStyle"))]
    pub choose_highlight_style: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "showContent"))]
    pub show_content: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "pdfLink"))]
    pub pdf_link: Option<bool>,

    #[cfg_attr(feature = "csr", serde(default, rename = "documentUri"))]
    pub document_uri: Option<DocumentUri>,

    #[cfg_attr(feature = "csr", serde(default, rename = "highlightStyle"))]
    pub highlight_style: Option<HighlightStyle>,

    #[cfg_attr(feature = "csr", serde(default))]
    pub toc: Option<TocSource>,

    #[cfg_attr(feature = "csr", serde(default, rename = "autoexpandLimit"))]
    pub autoexpand_limit: Option<LogicalLevel>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub section_wrap: Option<SectionWrap>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub paragraph_wrap: Option<ParagraphWrap>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub slide_wrap: Option<SlideWrap>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub problem_wrap: Option<ProblemWrap>,

    #[cfg(feature = "callbacks")]
    #[serde(skip)]
    pub on_section_title: Option<OnSectionTitle>,
}

#[wasm_bindgen::prelude::wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HighlightStyle {
    Colored,
    Subtle,
    Off,
    None,
}
impl HighlightStyle {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Colored => "colored",
            Self::Subtle => "subtle",
            Self::Off => "off",
            Self::None => "none",
        }
    }

    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "colored" => Some(Self::Colored),
            "subtle" => Some(Self::Subtle),
            "off" => Some(Self::Off),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}
impl ftml_js_utils::conversion::FromWasmBindgen for HighlightStyle {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AllowHovers(pub bool);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AllowFullscreen(pub bool);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AllowFormals(pub bool);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ShowContent(pub bool);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PdfLink(pub bool);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ChooseHighlightStyle(pub bool);

#[derive(Copy, Clone)]
pub struct AutoexpandLimit(pub LogicalLevel);

#[derive(Copy, Clone)]
pub struct AllowNotationChanges(bool);

#[derive(thiserror::Error, Debug)]
pub enum FtmlConfigParseError {
    #[error("not a javascript object")]
    NotAnObject(JsDisplay),
    #[error("invalid value for {0}")]
    InvalidValue(&'static str),
    #[error("invalid URI in {0}: {1}")]
    InvalidUri(&'static str, ftml_uris::errors::UriParseError),
    //#[cfg(feature = "callbacks")]
    //#[error("invalid javascript function in {0}: {1}")]
    //InvalidFun(&'static str, leptos_react::functions::NotAJsFunction),
}

pub struct FtmlConfigParseErrors {
    pub config: FtmlConfig,
    pub errors: Vec<FtmlConfigParseError>,
}
impl std::fmt::Display for FtmlConfigParseErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FtmlConfig")
            .field("cfg", &self.config)
            .field("errors", &self.errors)
            .finish()
    }
}

impl ftml_js_utils::conversion::FromJs for FtmlConfig {
    type Error = FtmlConfigParseErrors;
    fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use ftml_js_utils::conversion::FromJs;
        if !value.is_object() {
            return Err(FtmlConfigParseErrors {
                config: Self::default(),
                errors: vec![FtmlConfigParseError::NotAnObject(JsDisplay(value))],
            });
        }
        let mut config = Self::default();
        let mut errors = Vec::new();
        macro_rules! get {
            ($name:literal+$field:ident:$tp:ty) => {
                match <$tp as FromJs>::from_field(&value, $name) {
                    Err(_) => errors.push(FtmlConfigParseError::InvalidValue($name)),
                    Ok(v) => config.$field = v,
                }
            };
        }
        get!("allowHovers"+allow_hovers:bool);
        get!("showContent"+show_content:bool);
        get!("allowFullscreen"+allow_fullscreen:bool);
        get!("allowFormalInfo"+allow_formals:bool);
        get!("pdfLink"+pdf_link:bool);
        get!("allowNotationChanges"+allow_notation_changes:bool);
        get!("chooseHighlightStyle"+choose_highlight_style:bool);
        get!("documentUri"+document_uri:DocumentUri);
        get!("highlightStyle"+highlight_style:HighlightStyle);
        get!("toc"+toc:TocSource);
        get!("autoexpandLimit"+autoexpand_limit:LogicalLevel);
        #[cfg(feature = "callbacks")]
        get!("sectionWrap"+section_wrap:SectionWrap);
        #[cfg(feature = "callbacks")]
        get!("paragraphWrap"+paragraph_wrap:ParagraphWrap);
        #[cfg(feature = "callbacks")]
        get!("slideWrap"+slide_wrap:SlideWrap);
        #[cfg(feature = "callbacks")]
        get!("problemWrap"+problem_wrap:ProblemWrap);
        #[cfg(feature = "callbacks")]
        get!("onSectionTitle"+on_section_title:OnSectionTitle);

        if errors.is_empty() {
            Ok(config)
        } else {
            Err(FtmlConfigParseErrors { config, errors })
        }
    }
}

/*
#[cfg(feature = "csr")]
impl leptos::wasm_bindgen::convert::TryFromJsValue for FtmlConfig {
    type Error = (Self, Vec<FtmlConfigParseError>);
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::too_many_lines)]
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
                    match  leptos_react::functions::FromJs::from_js(v) {
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
        get!(v @ "showContent" = SHOW_CONTENT as_bool { config.show_content = Some(v)});
        get!(v @ "allowFullscreen" = ALLOW_FULLSCREEN as_bool { config.allow_fullscreen = Some(v)});
        get!(v @ "allowFormalInfo" = ALLOW_FORMAL_INFO as_bool { config.allow_formals = Some(v)});
        get!(v @ "pdfLink" = PDF_LINK as_bool { config.pdf_link = Some(v)});
        get!(v @ "allowNotationChanges" = ALLOW_NOTATION_CHANGES as_bool { config.allow_notation_changes = Some(v)} );
        get!(v @ "chooseHighlightStyle" = CHOOSE_HIGHLIGHT_STYLE as_bool { config.choose_highlight_style = Some(v)} );
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
        get!(v @ "toc" = TOC ?
            j => TocSource::try_from_js_value(j.clone());
            _e => FtmlConfigParseError::InvalidValue("toc", JsDisplay(j));
            {config.toc = Some(v)}
        );
        get!(v @ "autoexpandLimit" = AUTOEXPAND_LIMIT ?
            j => LogicalLevel::try_from_js_value(j.clone());
            _e => FtmlConfigParseError::InvalidValue("autoexpandLimit", JsDisplay(j));
            {config.autoexpand_limit = Some(v)}
        );
        #[cfg(feature = "callbacks")]
        get!(v @ "sectionWrap" = SECTION_WRAP ?F { config.section_wrap = Some(v) });
        #[cfg(feature = "callbacks")]
        get!(v @ "paragraphWrap" = PARAGRAPH_WRAP ?F { config.paragraph_wrap = Some(v) });
        #[cfg(feature = "callbacks")]
        get!(v @ "slideWrap" = SLIDE_WRAP ?F { config.slide_wrap = Some(v) });
        #[cfg(feature = "callbacks")]
        get!(v @ "problemWrap" = PROBLEM_WRAP ?F { config.problem_wrap = Some(v) });
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
*/

impl FtmlConfig {
    #[must_use]
    pub fn apply(self) -> Option<DocumentUri> {
        if let Some(b) = self.allow_hovers {
            provide_context(AllowHovers(b));
        }
        if let Some(b) = self.show_content {
            provide_context(ShowContent(b));
        }
        if let Some(b) = self.allow_formals {
            provide_context(AllowFormals(b));
        }
        if let Some(b) = self.allow_fullscreen {
            provide_context(AllowFullscreen(b));
        }
        if let Some(b) = self.choose_highlight_style {
            provide_context(ChooseHighlightStyle(b));
        }
        if let Some(b) = self.pdf_link {
            provide_context(PdfLink(b));
        }
        if let Some(b) = self.allow_notation_changes {
            provide_context(AllowNotationChanges(b));
        }
        if let Some(h) = self.highlight_style {
            let style = RwSignal::new(h);
            provide_context(style);
            provide_context(style.read_only());
        }
        if let Some(toc) = self.toc {
            //let toc = RwSignal::new(h);
            //provide_context(toc);
            provide_context(toc);
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
        if let Some(b) = self.paragraph_wrap {
            provide_context(Some(b));
        }
        #[cfg(feature = "callbacks")]
        if let Some(b) = self.slide_wrap {
            provide_context(Some(b));
        }
        #[cfg(feature = "callbacks")]
        if let Some(b) = self.problem_wrap {
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

impl FtmlConfig {
    #[inline]
    #[must_use]
    pub fn allow_hovers() -> bool {
        use_context::<AllowHovers>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn show_content() -> bool {
        use_context::<ShowContent>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn allow_fullscreen() -> bool {
        use_context::<AllowFullscreen>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn pdf_link() -> bool {
        use_context::<PdfLink>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn allow_formal_info() -> bool {
        use_context::<AllowFormals>().is_none_or(|b| b.0)
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
    pub fn choose_highlight_style() -> bool {
        use_context::<ChooseHighlightStyle>().is_none_or(|b| b.0)
    }

    #[inline]
    #[must_use]
    pub fn autoexpand_limit() -> ReadSignal<AutoexpandLimit> {
        expect_context()
    }

    #[inline]
    #[must_use]
    pub fn with_toc_source<R>(f: impl FnOnce(&TocSource) -> R) -> Option<R> {
        with_context(f)
    }

    /// ### Panics
    pub fn disable_hovers<V: IntoView, F: FnOnce() -> V>(f: F) -> impl IntoView + use<F, V> {
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
    pub fn with_allow_hovers<V: IntoView + 'static>(
        value: bool,
        children: TypedChildren<V>,
    ) -> impl IntoView {
        Provider(leptos::context::ProviderProps {
            value: AllowHovers(value),
            children,
        })
    }
}
