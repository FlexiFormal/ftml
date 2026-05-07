#![allow(clippy::must_use_candidate)]

use ftml_component_utils::theming::{Theme, Themer as BaseThemer};
use leptos::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
pub enum ThemeType {
    #[default]
    Light,
    Dark,
}
impl<'a> From<&'a Theme> for ThemeType {
    fn from(theme: &'a Theme) -> Self {
        if theme.name == "dark" {
            Self::Dark
        } else {
            Self::Light
        }
    }
}
impl From<ThemeType> for Theme {
    fn from(tp: ThemeType) -> Self {
        match tp {
            ThemeType::Light => Self::light(),
            ThemeType::Dark => Self::dark(),
        }
    }
}

#[component(transparent)]
pub fn Themer<Ch: IntoView + 'static>(
    children: TypedChildren<Ch>,
    #[prop(optional)] safe: bool,
) -> impl IntoView {
    use ftml_component_utils::toasts::ToasterProvider;
    use leptos::either::EitherOf3::{A, B, C};
    let children = children.into_inner();
    if use_context::<RwSignal<Theme>>().is_some() {
        return A(children());
    }
    let theme = RwSignal::<Theme>::new(Theme::light());
    let theme_set = RwSignal::new(false);
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
        #[allow(unused_variables)]
        theme.with(|theme| {
            #[cfg(any(feature = "csr", feature = "hydrate"))]
            {
                use gloo_storage::Storage;
                let _ = gloo_storage::LocalStorage::set("theme", ThemeType::from(theme));
            }
        });
    });
    let i = view! {
        <BaseThemer theme>
          <ToasterProvider>{children()}</ToasterProvider>
        </BaseThemer>
    };
    if safe {
        B(i.attr(
            "style",
            "\
            font-family:inherit;\
            font-size:inherit;\
            font-weight:inherit;\
            line-height:inherit;\
            background-color:inherit;\
            color:inherit;\
            display:contents;
        ",
        ))
    } else {
        C(i)
    }
}
