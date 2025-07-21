#![allow(clippy::must_use_candidate)]

use leptos::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
pub enum ThemeType {
    #[default]
    Light,
    Dark,
}
impl<'a> From<&'a thaw::Theme> for ThemeType {
    fn from(theme: &'a thaw::Theme) -> Self {
        if theme.name == "dark" {
            Self::Dark
        } else {
            Self::Light
        }
    }
}
impl From<ThemeType> for thaw::Theme {
    fn from(tp: ThemeType) -> Self {
        match tp {
            ThemeType::Light => Self::light(),
            ThemeType::Dark => Self::dark(),
        }
    }
}

#[component(transparent)]
pub fn Themer<Ch: IntoView + 'static>(children: TypedChildren<Ch>) -> impl IntoView {
    use thaw::{ConfigProvider, Theme, ToasterProvider};
    let theme = RwSignal::<thaw::Theme>::new(Theme::light());
    let theme_set = RwSignal::new(false);
    let children = children.into_inner();
    provide_context(theme);
    Effect::new(move || {
        if !theme_set.get() {
            #[cfg(any(feature = "csr", feature = "hydrate"))]
            {
                use gloo_storage::Storage;
                theme_set.update_untracked(|x| *x = true);
                if let Ok(t) = gloo_storage::LocalStorage::get::<ThemeType>("theme") {
                    theme.set(t.into());
                }
            }
        }
    });
    Effect::new(move || {
        theme.with(|theme| {
            #[cfg(any(feature = "csr", feature = "hydrate"))]
            {
                use gloo_storage::Storage;
                let _ = gloo_storage::LocalStorage::set("theme", ThemeType::from(theme));
            }
        });
    });
    view! {
        <ConfigProvider theme>
          <ToasterProvider>{children()}</ToasterProvider>
        </ConfigProvider>
    }
}
