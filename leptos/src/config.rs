use ftml_uris::DocumentUri;
use leptos::context::Provider;
use leptos::prelude::*;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "csr", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FtmlConfig {
    #[cfg_attr(feature = "csr", serde(default))]
    #[cfg_attr(feature = "csr", serde(rename = "allowHovers"))]
    allow_hovers: Option<bool>,
    #[cfg_attr(feature = "csr", serde(default))]
    #[cfg_attr(feature = "csr", serde(rename = "documentUri"))]
    document_uri: Option<DocumentUri>,
}
impl FtmlConfig {
    #[must_use]
    pub fn apply(self) -> Option<DocumentUri> {
        if let Some(b) = self.allow_hovers {
            provide_context(AllowHovers(b));
        }
        self.document_uri
    }
}

#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "csr", derive(serde::Serialize, serde::Deserialize))]
pub enum HighlightOption {
    Colored,
    Subtle,
    Off,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct AllowHovers(pub bool);

pub(crate) struct FtmlConfigState;
impl FtmlConfigState {
    #[inline]
    pub fn allow_hovers() -> bool {
        use_context::<AllowHovers>().is_some_and(|b| b.0)
    }
    #[inline]
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
